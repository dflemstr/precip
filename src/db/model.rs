use chrono;
use uuid;

#[derive(Debug)]
pub struct SampleTimeseries {
    pub module_uuid: uuid::Uuid,
    pub slice: chrono::DateTime<chrono::Utc>,
    pub min_raw_voltage: f64,
    pub max_raw_voltage: f64,
    pub p25_raw_voltage: f64,
    pub p50_raw_voltage: f64,
    pub p75_raw_voltage: f64,
}

#[derive(Debug)]
pub struct SampleRange {
    pub module_uuid: uuid::Uuid,
    pub min_raw_voltage: f64,
    pub max_raw_voltage: f64,
}

#[derive(Debug)]
pub struct Stats {
    pub module_uuid: uuid::Uuid,
    pub min_moisture: f64,
    pub max_moisture: f64,
    pub last_moisture: f64,
}

#[derive(Debug)]
pub struct GlobalStats {
    pub temperature: f64,
}

#[derive(Debug)]
pub struct PumpEvent {
    pub created: chrono::DateTime<chrono::Utc>,
    pub module_uuid: uuid::Uuid,
    pub pump_running: bool,
}
