use ads1x15;
use uuid;

pub struct ModuleConfig {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub name: String,
    pub moisture_channel: ads1x15::Channel,
    pub pump_channel: u64,
    pub min_moisture: f64,
}
