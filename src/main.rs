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
#[macro_use]
extern crate slog;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde;
extern crate slog_async;
extern crate slog_journald;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sysfs_gpio;
extern crate tokio;
extern crate toml;
extern crate uuid;

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
pub mod pumps;
pub mod sensors;
pub mod util;

fn main() -> Result<(), failure::Error> {
    use std::io::Read;

    let log = init_log()?;
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

    let i2c_dev = i2cdev::linux::LinuxI2CDevice::new("/dev/i2c-1", 0x48)?;
    let dac = ads1x15::Ads1x15::new_ads1115(i2c_dev);

    let loaded_modules = sync::Arc::new(load_modules(log.clone(), &config, &db)?);

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
        let sampler = sync::Arc::new(sensors::Ads1x15Sampler::start(dac).unwrap());

        tokio::spawn(
            collect_stats_job(log.clone(), loaded_modules.clone(), db.clone(), state_tx).map_err({
                let log = log.clone();
                move |e| error!(log, "collect job failed: {}", e)
            }),
        );

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

fn init_log() -> Result<slog::Logger, failure::Error> {
    use slog::Drain;

    // Work-around 'term' issue; for example lacking 256color support
    if env::var("TERM")
        .map(|s| s.starts_with("xterm"))
        .unwrap_or(false)
    {
        env::set_var("TERM", "xterm");
    }

    let decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::FullFormat::new(decorator).build().fuse();
    let journald_drain = slog::LevelFilter::new(slog_journald::JournaldDrain, slog::Level::Info);

    let drain = slog_async::Async::new(
        slog::Duplicate::new(term_drain, journald_drain).ignore_res(),
    ).build()
        .fuse();

    Ok(slog::Logger::root(drain, o!()))
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
        // TODO(dflemstr): implement proper scale for moisture (maybe in percent)
        let moisture = 3.3 - await!(sampler.sample(module.moisture_channel))? as f64;
        db.insert_sample(module.id, now, moisture)?;

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
        let timeseries_samples = db.collect_timeseries_samples()?;
        let pump_events = db.collect_pump_events()?;
        let stats = db.collect_stats()?;
        let state = collect::State::new(
            log.clone(),
            created,
            &loaded_modules,
            &timeseries_samples,
            &pump_events,
            &stats,
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
                min_moisture: plant.min_moisture,
                max_moisture: plant.max_moisture,
                moisture_channel: match plant.moisture_channel {
                    0 => ads1x15::Channel::A0,
                    1 => ads1x15::Channel::A1,
                    2 => ads1x15::Channel::A2,
                    3 => ads1x15::Channel::A3,
                    x => bail!("No such moisture channel: {}", x),
                },
                pump_channel: plant.pump_channel as u64,
            }))
        })
        .collect()
}
