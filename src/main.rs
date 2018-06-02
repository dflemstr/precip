#![feature(proc_macro, generators, trivial_bounds)]

extern crate ads1x15;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate futures_await as futures;
extern crate i2cdev;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate toml;
extern crate uuid;

use std::env;
use std::fs;
use std::sync;
use std::thread;
use std::time;

use futures::prelude::*;

pub mod collect;
pub mod config;
pub mod db;
pub mod model;
pub mod sensors;
pub mod util;

fn main() -> Result<(), failure::Error> {
    use std::io::Read;

    env_logger::init();
    dotenv::dotenv().unwrap();

    let mut config_string = String::new();
    fs::File::open("config.toml")?.read_to_string(&mut config_string)?;
    let config = toml::from_str(&config_string)?;

    let database_url = env::var("DATABASE_URL")?;
    let db = sync::Arc::new(db::Db::connect(&database_url)?);

    let i2c_dev = i2cdev::linux::LinuxI2CDevice::new("/dev/i2c-1", 0x48)?;
    let dac = ads1x15::Ads1x15::new_ads1115(i2c_dev);

    let loaded_modules = sync::Arc::new(load_modules(&config, &db)?);

    let (state_tx, state_rx) = futures::sync::mpsc::channel(0);

    let handle = thread::Builder::new()
        .name("s3-uploader".to_owned())
        .spawn(|| {
            tokio::executor::current_thread::block_on_all(collect::upload_states_to_s3(state_rx))
        })?;

    tokio::run(futures::future::lazy(move || {
        let sampler = sync::Arc::new(sensors::Ads1x15Sampler::start(dac).unwrap());

        tokio::spawn(
            collect_stats_job(loaded_modules.clone(), db.clone(), state_tx)
                .map_err(|e| error!("collect job failed: {}", e)),
        );

        for module in &*loaded_modules {
            tokio::spawn(
                sample_module_job(module.clone(), sampler.clone(), db.clone())
                    .map_err(|e| error!("sampling module failed: {}", e)),
            );
        }

        info!("started");

        Ok(())
    }));

    handle.join().unwrap()?;

    Ok(())
}

#[async]
fn sample_module_job(
    module: sync::Arc<model::ModuleConfig>,
    sampler: sync::Arc<sensors::Ads1x15Sampler>,
    db: sync::Arc<db::Db>,
) -> Result<(), failure::Error> {
    #[async]
    for _ in util::every(
        format!("sample {}", module.uuid),
        time::Duration::from_secs(1),
    ) {
        let now = chrono::Utc::now();
        // TODO(dflemstr): implement proper scale for moisture (maybe in percent)
        let moisture = 3.3 - await!(sampler.sample(module.moisture_channel))? as f64;
        db.insert_sample(module.id, now, moisture)?;
    }

    Ok(())
}

#[async]
fn collect_stats_job(
    loaded_modules: sync::Arc<Vec<sync::Arc<model::ModuleConfig>>>,
    db: sync::Arc<db::Db>,
    state_tx: futures::sync::mpsc::Sender<collect::State>,
) -> Result<(), failure::Error> {
    #[async]
    for _ in util::every("collect stats".to_owned(), time::Duration::from_secs(60)) {
        let state_tx = state_tx.clone();
        let timeseries_samples = db.collect_timeseries_samples()?;
        let stats = db.collect_stats()?;
        let state = collect::State::new(&loaded_modules, &timeseries_samples, &stats);

        await!(state_tx.send(state))?;
    }

    Ok(())
}

fn load_modules(
    config: &config::Config,
    db: &db::Db,
) -> Result<Vec<sync::Arc<model::ModuleConfig>>, failure::Error> {
    config
        .plant
        .iter()
        .map(|(uuid, plant)| {
            let db_module = db.ensure_module(*uuid, &plant.name)?;
            debug!("loaded module: {:?}", db_module);

            Ok(sync::Arc::new(model::ModuleConfig {
                id: db_module.id,
                uuid: *uuid,
                name: plant.name.clone(),
                moisture_channel: match plant.moisture {
                    0 => ads1x15::Channel::A0,
                    1 => ads1x15::Channel::A1,
                    2 => ads1x15::Channel::A2,
                    3 => ads1x15::Channel::A3,
                    x => bail!("No such moisture channel: {}", x),
                },
            }))
        })
        .collect()
}
