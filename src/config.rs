use std::collections;

use uuid;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub plant: collections::HashMap<uuid::Uuid, Plant>,
}

#[derive(Debug, Deserialize)]
pub struct Plant {
    pub name: String,
    pub moisture: u8,
    pub pump: u8,
}
