extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;
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
    use rusoto_s3::S3;

    env_logger::init();
    dotenv::dotenv().unwrap();

    let database_url = env::var("DATABASE_URL")?;

    let db = db::Db::connect(&database_url)?;

    let loaded_modules = MODULES
        .iter()
        .map(|config| {
            let db_module = db.ensure_module(uuid::Uuid::parse_str(config.uuid)?, config.name)?;

            Ok(ModuleConfig {
                id: db_module.id,
                ..*config
            })
        })
        .collect::<Result<Vec<_>, failure::Error>>()?;

    let stats = db.collect_stats()?;
    let mut modules = collections::HashMap::new();

    for module in loaded_modules {
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

    let state = schema::State {
        modules: modules.into_iter().map(|(_, v)| v).collect(),
    };
    let state_json = serde_json::to_vec(&state)?;

    let s3_client = rusoto_s3::S3Client::simple(rusoto_core::region::Region::EuWest1);

    let future = s3_client.put_object(&rusoto_s3::PutObjectRequest {
        body: Some(state_json),
        bucket: "precip-stats".to_owned(),
        key: "data.json".to_owned(),
        content_type: Some("application/json".to_owned()),
        cache_control: Some("max-age=300".to_owned()),
        acl: Some("public-read".to_owned()),
        ..rusoto_s3::PutObjectRequest::default()
    });

    tokio::executor::current_thread::block_on_all(future)?;

    Ok(())
}
