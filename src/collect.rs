use failure;
use futures;
use rusoto_core;
use rusoto_s3;
use serde_json;

use futures::prelude::*;

use std::borrow;
use std::collections;

use chrono;

use db;
use model;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub modules: Vec<Module>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: String,
    pub name: String,
    pub running: bool,
    pub force_running: bool,
    pub min_moisture: f64,
    pub max_moisture: f64,
    pub last_moisture: f64,
    pub moisture_timeseries: Timeseries<f64>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeseries<A> {
    pub measurement_start: Vec<chrono::DateTime<chrono::Utc>>,
    pub min: Vec<A>,
    pub max: Vec<A>,
    pub p25: Vec<A>,
    pub p50: Vec<A>,
    pub p75: Vec<A>,
}

impl State {
    pub fn new<M, TS, S>(loaded_modules: &[M], timeseries_samples: &[TS], stats: &[S]) -> State
    where
        M: borrow::Borrow<model::ModuleConfig>,
        TS: borrow::Borrow<db::model::TimeseriesSample>,
        S: borrow::Borrow<db::model::Stats>,
    {
        let mut modules = collections::HashMap::new();

        for module in loaded_modules {
            let module = module.borrow();
            modules.insert(
                module.id,
                Module {
                    id: module.uuid.to_string(),
                    name: module.name.to_owned(),
                    running: false,
                    force_running: false,
                    min_moisture: 0.0,
                    max_moisture: 0.0,
                    last_moisture: 0.0,
                    moisture_timeseries: Timeseries::default(),
                },
            );
        }

        for sample in timeseries_samples {
            let sample = sample.borrow();
            if let Some(module) = modules.get_mut(&sample.module_id) {
                module
                    .moisture_timeseries
                    .measurement_start
                    .push(sample.slice);
                module.moisture_timeseries.min.push(sample.min_moisture);
                module.moisture_timeseries.max.push(sample.max_moisture);
                module.moisture_timeseries.p25.push(sample.p25_moisture);
                module.moisture_timeseries.p50.push(sample.p50_moisture);
                module.moisture_timeseries.p75.push(sample.p75_moisture);
            }
        }

        for stat in stats {
            let stat = stat.borrow();
            if let Some(module) = modules.get_mut(&stat.module_id) {
                module.min_moisture = stat.min_moisture;
                module.max_moisture = stat.max_moisture;
                module.last_moisture = stat.last_moisture;
            }
        }
        let modules = modules.into_iter().map(|(_, v)| v).collect();

        State { modules }
    }
}

#[async]
pub fn upload_states_to_s3(
    states: futures::sync::mpsc::Receiver<State>,
) -> Result<(), failure::Error> {
    use rusoto_s3::S3;

    let s3_client = rusoto_s3::S3Client::simple(rusoto_core::region::Region::EuWest1);

    #[async]
    for state in states.map_err(|_| failure::err_msg("state channel poisoned")) {
        let state_json = serde_json::to_vec(&state)?;

        if let Err(e) = await!(s3_client.put_object(&rusoto_s3::PutObjectRequest {
            body: Some(state_json),
            bucket: "precip-stats".to_owned(),
            key: "data.json".to_owned(),
            content_type: Some("application/json".to_owned()),
            cache_control: Some("max-age=300".to_owned()),
            acl: Some("public-read".to_owned()),
            ..rusoto_s3::PutObjectRequest::default()
        })) {
            warn!("upload to S3 failed, will retry later: {}", e);
        } else {
            info!("uploaded state to S3");
        }
    }

    Ok(())
}
