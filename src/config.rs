use std::collections;

use uuid;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub plant: collections::HashMap<uuid::Uuid, Plant>,
}

#[derive(Debug, Deserialize)]
pub struct Plant {
    pub name: String,
    pub description: String,
    pub moisture_channel: u8,
    pub pump_channel: u8,
    pub min_moisture: f64,
    pub max_moisture: f64,
}
