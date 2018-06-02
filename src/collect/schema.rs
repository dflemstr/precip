use chrono;

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
