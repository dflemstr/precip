use chrono;

#[derive(Debug, Default, Serialize)]
pub struct State {
    pub modules: Vec<Module>,
}

#[derive(Debug, Serialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub running: bool,
    pub force_running: bool,
    pub historical_moisture: Vec<Sample<f64>>,
}

#[derive(Debug, Serialize)]
pub struct Sample<A> {
    pub measurement_start: chrono::DateTime<chrono::Utc>,
    pub min: A,
    pub max: A,
    pub p25: A,
    pub p50: A,
    pub p75: A,
}
