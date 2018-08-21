use ads1x15;
use cron;
use uuid;

pub struct ModuleConfig {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub moisture_i2c_address: u16,
    pub moisture_channel: ads1x15::Channel,
    pub pump_enabled: bool,
    pub pump_schedule: Option<cron::Schedule>,
    pub pump_channel: u64,
    pub min_moisture: f64,
    pub max_moisture: f64,
    pub moisture_voltage_dry: f64,
    pub moisture_voltage_wet: f64,
}
