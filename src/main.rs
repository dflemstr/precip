#![feature(proc_macro, generators, trivial_bounds)]

extern crate ads1x15;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate failure;
extern crate futures_await as futures;
extern crate i2cdev;
extern crate i2cdev_bmp280;
extern crate i2csensors;
extern crate itertools;
#[macro_use]
extern crate slog;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate rusoto_core;
extern crate rusoto_s3;
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
extern crate toml;
extern crate uuid;

use std::collections;
use std::env;
use std::ffi;
use std::fs;
use std::sync;
use std::thread;
use std::time;

use futures::prelude::*;

pub mod collect;
pub mod config;
pub mod db;
pub mod model;
pub mod options;
pub mod pumps;
pub mod sensors;
pub mod util;

fn main() -> Result<(), failure::Error> {
    use itertools::Itertools;
    use std::io::Read;
    use structopt::StructOpt;

    let options = options::Options::from_args();
    let log = init_log(&options)?;
    let _log_scope = slog_scope::set_global_logger(log.clone());
    slog_stdlog::init()?;

    if let Err(e) = dotenv::dotenv() {
        warn!(log, "Failed to read .env: {}", e);
    }

    let mut config_string = String::new();
    let config_path =
        env::var_os("PRECIP_CONFIG").unwrap_or_else(|| ffi::OsString::from("config.toml"));
    fs::File::open(config_path)?.read_to_string(&mut config_string)?;
    let config = toml::from_str(&config_string)?;

    let database_url = env::var("DATABASE_URL")?;
    let db = sync::Arc::new(db::Db::connect(log.clone(), &database_url)?);

    let loaded_modules = sync::Arc::new(load_modules(log.clone(), &config, &db)?);

    let dacs = (*loaded_modules)
        .iter()
        .map(|m| m.moisture_i2c_address)
        .unique()
        .map(|addr| {
            let i2c_dev = i2cdev::linux::LinuxI2CDevice::new("/dev/i2c-1", 0x48)?;
            Ok((addr, ads1x15::Ads1x15::new_ads1115(i2c_dev)))
        })
        .collect::<Result<collections::HashMap<_, _>, failure::Error>>()?;

    let bmp280_i2c_dev = i2cdev_bmp280::get_linux_bmp280_i2c_device().unwrap();
    let bmp280 = i2cdev_bmp280::BMP280::new(
        bmp280_i2c_dev,
        i2cdev_bmp280::BMP280Settings {
            compensation: i2cdev_bmp280::BMP280CompensationAlgorithm::Float,
            t_sb: i2cdev_bmp280::BMP280Timing::ms0_5,
            iir_filter_coeff: i2cdev_bmp280::BMP280FilterCoefficient::UltraHigh,
            osrs_t: i2cdev_bmp280::BMP280TemperatureOversampling::x16,
            osrs_p: i2cdev_bmp280::BMP280PressureOversampling::UltraHighResolution,
            power_mode: i2cdev_bmp280::BMP280PowerMode::NormalMode,
        },
    )?;

    let (state_tx, state_rx) = futures::sync::mpsc::channel(0);

    let handle = {
        let log = log.clone();
        thread::Builder::new()
            .name("s3-uploader".to_owned())
            .spawn(|| {
                tokio::executor::current_thread::block_on_all(collect::upload_states_to_s3(
                    log, state_rx,
                ))
            })?
    };

    tokio::run(futures::future::lazy(move || {
        let sampler = sync::Arc::new(sensors::Ads1x15Sampler::start(dacs).unwrap());

        tokio::spawn(
            collect_stats_job(log.clone(), loaded_modules.clone(), db.clone(), state_tx).map_err({
                let log = log.clone();
                move |e| error!(log, "collect job failed: {}", e)
            }),
        );

        tokio::spawn(sample_global_job(log.clone(), bmp280, db.clone()).map_err({
            let log = log.clone();
            move |e| error!(log, "sampling module failed: {}", e)
        }));

        for module in &*loaded_modules {
            tokio::spawn(
                sample_module_job(log.clone(), module.clone(), sampler.clone(), db.clone())
                    .map_err({
                        let log = log.clone();
                        move |e| error!(log, "sampling module failed: {}", e)
                    }),
            );
        }

        info!(log, "started");

        Ok(())
    }));

    handle.join().unwrap()?;

    Ok(())
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
    mut bmp280: i2cdev_bmp280::BMP280<D>,
    db: sync::Arc<db::Db>,
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
        let temperature = i2csensors::Thermometer::temperature_celsius(&mut bmp280)? as f64;
        db.insert_global_sample(now, temperature)?;
    }
    Ok(())
}

#[async]
fn sample_module_job(
    log: slog::Logger,
    module: sync::Arc<model::ModuleConfig>,
    sampler: sync::Arc<sensors::Ads1x15Sampler>,
    db: sync::Arc<db::Db>,
) -> Result<(), failure::Error> {
    let mut last_report = time::Instant::now();
    let pump = sync::Arc::new(pumps::Pump::new(log.clone(), module.pump_channel)?);

    #[async]
    for _ in util::every(
        log.clone(),
        format!("sample {}", module.uuid),
        time::Duration::from_secs(1),
    ) {
        let now = chrono::Utc::now();

        let moisture_voltage =
            await!(sampler.sample(module.moisture_i2c_address, module.moisture_channel))? as f64;
        let (moisture_min_voltage, moisture_max_voltage) =
            db.fetch_module_moisture_voltage_range(module.id)?;

        let moisture = compute_moisture(
            &*module,
            moisture_voltage,
            moisture_min_voltage,
            moisture_max_voltage,
        );

        db.insert_sample(module.id, now, moisture, moisture_voltage)?;

        if pump.running()? {
            if moisture > module.max_moisture {
                info!(
                    log,
                    "Turning off pump name={:?} channel={} uuid={}",
                    module.name,
                    module.pump_channel,
                    module.uuid
                );
                pump.set_running(false)?;
                db.insert_pump_event(module.id, now, false)?;
            }
        } else {
            if moisture < module.min_moisture {
                info!(
                    log,
                    "Turning on pump name={:?} channel={} uuid={}",
                    module.name,
                    module.pump_channel,
                    module.uuid
                );
                pump.set_running(true)?;
                db.insert_pump_event(module.id, now, true)?;
            }
        }

        if last_report.elapsed() > time::Duration::from_secs(60) {
            info!(
                log,
                "sensor reading name={:?} moisture={} uuid={}", module.name, moisture, module.uuid
            );
            last_report = time::Instant::now();
        }
    }

    Ok(())
}

fn compute_moisture(
    module: &model::ModuleConfig,
    moisture_voltage: f64,
    moisture_min_voltage: Option<f64>,
    moisture_max_voltage: Option<f64>,
) -> f64 {
    let moisture_voltage_wet = moisture_min_voltage
        .map(|v| v.min(module.moisture_voltage_wet))
        .unwrap_or(module.moisture_voltage_wet);
    let moisture_voltage_dry = moisture_max_voltage
        .map(|v| v.max(module.moisture_voltage_dry))
        .unwrap_or(module.moisture_voltage_dry);
    let moisture_voltage_range = moisture_voltage_dry - moisture_voltage_wet;
    1.0 - (moisture_voltage - moisture_voltage_wet) / moisture_voltage_range
}

#[async]
fn collect_stats_job(
    log: slog::Logger,
    loaded_modules: sync::Arc<Vec<sync::Arc<model::ModuleConfig>>>,
    db: sync::Arc<db::Db>,
    state_tx: futures::sync::mpsc::Sender<collect::State>,
) -> Result<(), failure::Error> {
    #[async]
    for _ in util::every(
        log.clone(),
        "collect stats".to_owned(),
        time::Duration::from_secs(60),
    ) {
        let created = chrono::Utc::now();
        let state_tx = state_tx.clone();
        let samples_timeseries = db.collect_samples_timeseries()?;
        let samples_range = db.collect_samples_range()?;
        let pump_events = db.collect_pump_events()?;
        let stats = db.collect_stats()?;
        let global_stats = db.collect_global_stats()?;
        let state = collect::State::new(
            log.clone(),
            created,
            &loaded_modules,
            &samples_timeseries,
            &samples_range,
            &pump_events,
            &stats,
            &global_stats,
        );

        await!(state_tx.send(state))?;
    }

    Ok(())
}

fn load_modules(
    log: slog::Logger,
    config: &config::Config,
    db: &db::Db,
) -> Result<Vec<sync::Arc<model::ModuleConfig>>, failure::Error> {
    config
        .plant
        .iter()
        .map(|(uuid, plant)| {
            let db_module = db.ensure_module(*uuid, &plant.name)?;
            debug!(log, "loaded module: {:?}", db_module);

            Ok(sync::Arc::new(model::ModuleConfig {
                id: db_module.id,
                uuid: *uuid,
                name: plant.name.clone(),
                description: plant.description.clone(),
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
                moisture_distance: plant.moisture.distance,
                pump_channel: plant.pump.channel as u64,
            }))
        })
        .collect()
}
