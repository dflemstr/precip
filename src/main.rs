#![feature(proc_macro, generators)]

extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate failure;
extern crate futures_await as futures;
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
extern crate uuid;

use std::collections;
use std::env;
use std::f64;
use std::sync;
use std::thread;
use std::time;

use futures::prelude::*;

pub mod db;
pub mod schema;

const MODULES: &'static [ModuleConfig] = &[
    ModuleConfig {
        id: 0,
        uuid: "19f513bc-9b69-4284-9811-c0c457d21555",
        name: "Plant 1",
    },
    ModuleConfig {
        id: 0,
        uuid: "71f3abbf-eda2-4b1f-a9a2-8af7f290e0a6",
        name: "Plant 2",
    },
    ModuleConfig {
        id: 0,
        uuid: "107ddad5-9274-458c-970e-1f6efefa9148",
        name: "Plant 3",
    },
    ModuleConfig {
        id: 0,
        uuid: "d4b61675-bfb1-4828-8a08-b99d32eb5e51",
        name: "Plant 4",
    },
];

struct ModuleConfig {
    id: i32,
    uuid: &'static str,
    name: &'static str,
}

fn main() -> Result<(), failure::Error> {
    env_logger::init();
    dotenv::dotenv().unwrap();

    let database_url = env::var("DATABASE_URL")?;
    let db = sync::Arc::new(db::Db::connect(&database_url)?);
    let loaded_modules = sync::Arc::new(load_modules(&db)?);

    let (state_tx, state_rx) = futures::sync::mpsc::channel(0);

    let handle = thread::Builder::new()
        .name("s3-uploader".to_owned())
        .spawn(|| tokio::executor::current_thread::block_on_all(upload_states_to_s3(state_rx)))?;

    tokio::run(futures::future::lazy(move || {
        tokio::spawn(
            collect_stats_job(loaded_modules.clone(), db.clone(), state_tx)
                .map_err(|e| error!("collect job failed: {}", e)),
        );

        for module in &*loaded_modules {
            tokio::spawn(
                sample_module_job(module.clone(), db.clone())
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
    module: sync::Arc<ModuleConfig>,
    db: sync::Arc<db::Db>,
) -> Result<(), failure::Error> {
    use chrono::Timelike;

    #[async]
    for _ in every(
        format!("sample {}", module.uuid),
        time::Duration::from_secs(1),
    ) {
        let now = chrono::Utc::now();

        let day_night_variance =
            (now.num_seconds_from_midnight() as f64 / 86400.0 * f64::consts::PI).sin();

        let humidity = 0.5 + 0.2 * day_night_variance + 0.3 * rand::random::<f64>();
        let temperature = 18.0 + 5.0 * day_night_variance + 5.0 * rand::random::<f64>();

        db.insert_sample(module.id, now, humidity, temperature)?;
    }

    Ok(())
}

#[async]
fn collect_stats_job(
    loaded_modules: sync::Arc<Vec<sync::Arc<ModuleConfig>>>,
    db: sync::Arc<db::Db>,
    state_tx: futures::sync::mpsc::Sender<schema::State>,
) -> Result<(), failure::Error> {
    #[async]
    for _ in every(
        "collect stats".to_owned(),
        time::Duration::from_secs(5 * 60),
    ) {
        let state_tx = state_tx.clone();
        let stats = db.collect_stats()?;
        let state = stats_to_state(&loaded_modules, &stats);

        await!(state_tx.send(state))?;
    }

    Ok(())
}

fn load_modules(db: &db::Db) -> Result<Vec<sync::Arc<ModuleConfig>>, failure::Error> {
    MODULES
        .iter()
        .map(|config| {
            let uuid = uuid::Uuid::parse_str(config.uuid)?;
            let db_module = db.ensure_module(uuid, config.name)?;
            debug!("loaded module: {:?}", db_module);

            Ok(sync::Arc::new(ModuleConfig {
                id: db_module.id,
                ..*config
            }))
        })
        .collect()
}

#[async]
fn upload_states_to_s3(
    states: futures::sync::mpsc::Receiver<schema::State>,
) -> Result<(), failure::Error> {
    use rusoto_s3::S3;

    let s3_client = rusoto_s3::S3Client::simple(rusoto_core::region::Region::EuWest1);

    #[async]
    for state in states.map_err(|_| failure::err_msg("state channel poisoned")) {
        let state_json = serde_json::to_vec(&state)?;

        await!(s3_client.put_object(&rusoto_s3::PutObjectRequest {
            body: Some(state_json),
            bucket: "precip-stats".to_owned(),
            key: "data.json".to_owned(),
            content_type: Some("application/json".to_owned()),
            cache_control: Some("max-age=300".to_owned()),
            acl: Some("public-read".to_owned()),
            ..rusoto_s3::PutObjectRequest::default()
        }))?;
        info!("uploaded state to S3");
    }

    Ok(())
}

fn stats_to_state<M>(loaded_modules: &[M], stats: &[db::model::Stats]) -> schema::State
where
    M: AsRef<ModuleConfig>,
{
    let mut modules = collections::HashMap::new();

    for module in loaded_modules {
        let module = module.as_ref();
        modules.insert(
            module.id,
            schema::Module {
                id: module.uuid.to_owned(),
                name: module.name.to_owned(),
                running: false,
                force_running: false,
                historical_humidity: Vec::new(),
                historical_temperature: Vec::new(),
            },
        );
    }

    for stat in stats {
        if let Some(module) = modules.get_mut(&stat.module_id) {
            module.historical_humidity.push(schema::Sample {
                measurement_start: stat.slice,
                min: stat.min_humidity,
                max: stat.max_humidity,
                p25: stat.p25_humidity,
                p50: stat.p50_humidity,
                p75: stat.p75_humidity,
            });
            module.historical_temperature.push(schema::Sample {
                measurement_start: stat.slice,
                min: stat.min_temperature,
                max: stat.max_temperature,
                p25: stat.p25_temperature,
                p50: stat.p50_temperature,
                p75: stat.p75_temperature,
            });
        }
    }
    let modules = modules.into_iter().map(|(_, v)| v).collect();

    schema::State { modules }
}

#[async_stream(item = ())]
fn every(name: String, duration: time::Duration) -> Result<(), failure::Error> {
    debug!("starting timer {:?}", name);

    #[async]
    for _ in tokio::timer::Interval::new(time::Instant::now(), duration) {
        debug!("timer tick {:?}", name);
        stream_yield!(());
    }

    Ok(())
}
