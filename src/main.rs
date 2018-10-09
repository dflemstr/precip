#![feature(generators)]

extern crate ads1x15;
extern crate chrono;
extern crate config as config_rs;
extern crate cron;
#[macro_use]
extern crate failure;
extern crate futures_await as futures;
extern crate i2cdev;
extern crate i2cdev_bmp280;
extern crate i2csensors;
extern crate influent;
extern crate itertools;
#[macro_use]
extern crate slog;
extern crate serde;
extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_journald;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate sysfs_gpio;
extern crate tokio;
extern crate uuid;

use std::collections;
use std::env;
use std::sync;
use std::time;

use futures::prelude::async;
use futures::prelude::await;

pub mod config;
pub mod db;
pub mod model;
pub mod options;
pub mod pumps;
pub mod sensors;
pub mod util;

fn main() -> Result<(), failure::Error> {
    use itertools::Itertools;
    use structopt::StructOpt;

    let options = options::Options::from_args();
    let log = init_log(&options)?;
    let _log_scope = slog_scope::set_global_logger(log.clone());
    slog_stdlog::init()?;

    let config = config::Config::load()?;

    let db = sync::Arc::new(db::Db::connect(
        log.clone(),
        config.db.credentials.into(),
        config.db.hosts,
    )?);

    let loaded_modules = sync::Arc::new(load_modules(config.plant)?);

    let dacs = (*loaded_modules)
        .iter()
        .map(|m| m.moisture_i2c_address)
        .unique()
        .map(|addr| {
            let i2c_dev = i2cdev::linux::LinuxI2CDevice::new("/dev/i2c-1", addr)?;
            Ok((addr, sync::Arc::new(ads1x15::Ads1x15::new_ads1115(i2c_dev))))
        }).collect::<Result<collections::HashMap<_, _>, failure::Error>>()?;

    let bmp280_i2c_dev = i2cdev_bmp280::get_linux_bmp280_i2c_device().unwrap();
    let bmp280 = sync::Arc::new(sync::Mutex::new(i2cdev_bmp280::BMP280::new(
        bmp280_i2c_dev,
        i2cdev_bmp280::BMP280Settings {
            compensation: i2cdev_bmp280::BMP280CompensationAlgorithm::Float,
            t_sb: i2cdev_bmp280::BMP280Timing::ms0_5,
            iir_filter_coeff: i2cdev_bmp280::BMP280FilterCoefficient::UltraHigh,
            osrs_t: i2cdev_bmp280::BMP280TemperatureOversampling::x16,
            osrs_p: i2cdev_bmp280::BMP280PressureOversampling::UltraHighResolution,
            power_mode: i2cdev_bmp280::BMP280PowerMode::NormalMode,
        },
    )?));

    let sampler = sync::Arc::new(sensors::Ads1x15Sampler::start(dacs)?);

    let sample_global_future: Box<futures::Future<Item = _, Error = _> + Send> =
        Box::new(sample_global_job(log.clone(), bmp280, db.clone()));
    let update_indices_future: Box<futures::Future<Item = _, Error = _> + Send> =
        Box::new(update_indices_job(log.clone(), db.clone()));
    let sample_futures = loaded_modules.iter().map(|module| {
        Box::new(sample_module_job(
            log.clone(),
            module.clone(),
            sampler.clone(),
            db.clone(),
        )) as Box<futures::Future<Item = _, Error = _> + Send>
    });
    let run_pump_futures = loaded_modules.iter().map(|module| {
        Box::new(run_pump_job(log.clone(), module.clone(), db.clone()))
            as Box<futures::Future<Item = _, Error = _> + Send>
    });

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime
        .block_on(futures::future::select_all(
            vec![sample_global_future, update_indices_future]
                .into_iter()
                .chain(run_pump_futures)
                .chain(sample_futures),
        )).map(|r| r.0)
        .map_err(|r| r.0)
}

fn init_log(options: &options::Options) -> Result<slog::Logger, failure::Error> {
    use slog::Drain;

    // Work-around 'term' issue; for example lacking 256color support
    if env::var("TERM")
        .map(|s| s.starts_with("xterm"))
        .unwrap_or(false)
    {
        env::set_var("TERM", "xterm");
    }

    let decorator = slog_term::TermDecorator::new().build();
    let term_drain: Box<slog::Drain<Ok = (), Err = slog::Never> + Send> = if options.silent {
        Box::new(slog::Discard)
    } else {
        let drain = slog_term::FullFormat::new(decorator).build().ignore_res();
        if options.debug {
            Box::new(drain)
        } else {
            let total = options.verbose as i32 - options.quiet as i32;

            if total < -3 {
                Box::new(slog::Discard)
            } else {
                let level = match total {
                    -3 => slog::Level::Critical,
                    -2 => slog::Level::Error,
                    -1 => slog::Level::Warning,
                    0 => slog::Level::Info,
                    1 => slog::Level::Debug,
                    _ => slog::Level::Trace,
                };

                Box::new(slog::LevelFilter::new(drain, level).fuse())
            }
        }
    };

    let journald_drain = slog_journald::JournaldDrain;

    let drain = slog_async::Async::new(slog_envlogger::new(
        slog::Duplicate::new(term_drain, journald_drain).ignore_res(),
    )).build()
    .fuse();

    Ok(slog::Logger::root(drain, o!()))
}

#[async]
fn sample_global_job<D>(
    log: slog::Logger,
    bmp280: sync::Arc<sync::Mutex<i2cdev_bmp280::BMP280<D>>>,
    db: sync::Arc<db::Db<'static>>,
) -> Result<(), failure::Error>
where
    D: i2cdev::core::I2CDevice + Sized + 'static,
    D::Error: Send + Sync + 'static,
{
    #[async]
    for _ in util::every(
        log.clone(),
        "measure temperature".to_owned(),
        time::Duration::from_secs(1),
    ) {
        let now = chrono::Utc::now();
        let temperature =
            i2csensors::Thermometer::temperature_celsius(&mut *bmp280.lock().unwrap())? as f64;
        let pressure = i2csensors::Barometer::pressure_kpa(&mut *bmp280.lock().unwrap())? as f64;

        if let Err(e) = db.insert_global_measurement(now, temperature, pressure) {
            warn!(log, "failed to insert plant measurement: {}", e);
        }
    }
    Ok(())
}

#[async]
fn update_indices_job(
    log: slog::Logger,
    db: sync::Arc<db::Db<'static>>,
) -> Result<(), failure::Error> {
    #[async]
    for _ in util::every(
        log.clone(),
        "update indices".to_owned(),
        time::Duration::from_secs(60),
    ) {
        if let Err(e) = db.update_plant_indices() {
            warn!(log, "failed to update plant indices: {}", e);
        }
    }
    Ok(())
}

#[async]
fn sample_module_job<D>(
    log: slog::Logger,
    module: sync::Arc<model::ModuleConfig>,
    sampler: sync::Arc<sensors::Ads1x15Sampler<D>>,
    db: sync::Arc<db::Db<'static>>,
) -> Result<(), failure::Error>
where
    D: i2cdev::core::I2CDevice + Send + 'static,
    <D as i2cdev::core::I2CDevice>::Error: Send + Sync + 'static,
{
    let mut last_report = time::Instant::now();

    #[async]
    for _ in util::every(
        log.clone(),
        format!("sample {}", module.uuid),
        time::Duration::from_secs(1),
    ) {
        let now = chrono::Utc::now();

        let moisture_voltage =
            await!(sampler.sample(module.moisture_i2c_address, module.moisture_channel))? as f64;

        if let Err(e) = db.insert_plant_measurement(now, module.uuid, moisture_voltage) {
            warn!(log, "failed to insert plant measurement: {}", e);
        }

        if last_report.elapsed() > time::Duration::from_secs(60) {
            info!(
                log,
                "sensor reading name={:?} moisture={}V uuid={}",
                module.name,
                moisture_voltage,
                module.uuid
            );
            last_report = time::Instant::now();
        }
    }

    Ok(())
}

#[async]
fn run_pump_job(
    log: slog::Logger,
    module: sync::Arc<model::ModuleConfig>,
    db: sync::Arc<db::Db<'static>>,
) -> Result<(), failure::Error> {
    if module.pump_enabled {
        let pump = pumps::Pump::new(log.clone(), module.pump_channel)?;
        while let Some(now) = module
            .pump_schedule
            .as_ref()
            .and_then(|schedule| schedule.upcoming(chrono::Local).next())
        {
            await!(tokio::timer::Delay::new(
                time::Instant::now() + (now - chrono::Local::now()).to_std()?
            ));

            info!(
                log,
                "running turning pump on name={:?} uuid={}", module.name, module.uuid
            );
            pump.set_running(true)?;
            db.insert_pump_measurement(chrono::Utc::now(), module.uuid, true)?;

            await!(tokio::timer::Delay::new(
                time::Instant::now() + module.pump_duration.unwrap_or(time::Duration::new(0, 0))
            ));

            info!(
                log,
                "running turning pump off name={:?} uuid={}", module.name, module.uuid
            );
            pump.set_running(false)?;
            db.insert_pump_measurement(chrono::Utc::now(), module.uuid, false)?;
        }
    }

    Ok(())
}

fn load_modules(
    plant: collections::HashMap<uuid::Uuid, config::Plant>,
) -> Result<Vec<sync::Arc<model::ModuleConfig>>, failure::Error> {
    use std::str::FromStr;

    plant
        .into_iter()
        .map(|(uuid, plant)| {
            Ok(sync::Arc::new(model::ModuleConfig {
                uuid,
                name: plant.name,
                description: plant.description,
                min_moisture: plant.moisture.min,
                max_moisture: plant.moisture.max,
                moisture_voltage_dry: plant.moisture.voltage_dry,
                moisture_voltage_wet: plant.moisture.voltage_wet,
                moisture_i2c_address: plant.moisture.channel.i2c_address,
                moisture_channel: match plant.moisture.channel.analog_pin {
                    0 => ads1x15::Channel::A0,
                    1 => ads1x15::Channel::A1,
                    2 => ads1x15::Channel::A2,
                    3 => ads1x15::Channel::A3,
                    x => bail!("No such moisture channel: {}", x),
                },
                pump_enabled: plant.pump.enabled,
                pump_schedule: plant
                    .pump
                    .schedule
                    .as_ref()
                    .map(|schedule| cron::Schedule::from_str(&schedule.start).unwrap()),
                pump_duration: plant
                    .pump
                    .schedule
                    .as_ref()
                    .map(|schedule| time::Duration::from_secs(schedule.duration_seconds)),
                pump_channel: plant.pump.channel as u64,
            }))
        }).collect()
}
