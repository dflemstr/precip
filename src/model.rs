use ads1x15;
use uuid;

pub struct ModuleConfig {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub moisture_i2c_address: u8,
    pub moisture_channel: ads1x15::Channel,
    pub moisture_distance: f64,
    pub pump_channel: u64,
    pub min_moisture: f64,
    pub max_moisture: f64,
}
