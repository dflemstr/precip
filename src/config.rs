use std::collections;
use std::u8;

use serde;
use uuid;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub plant: collections::HashMap<uuid::Uuid, Plant>,
}

#[derive(Debug, Deserialize)]
pub struct Plant {
    pub name: String,
    pub description: String,
    pub moisture: Moisture,
    pub pump: Pump,
}

#[derive(Debug, Deserialize)]
pub struct Moisture {
    #[serde(deserialize_with = "deserialize_moisture_channel")]
    pub channel: MoistureChannel,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug)]
pub struct MoistureChannel {
    pub i2c_address: u8,
    pub analog_pin: u8,
}

#[derive(Debug, Deserialize)]
pub struct Pump {
    pub channel: u8,
}

fn deserialize_moisture_channel<'de, D>(deserializer: D) -> Result<MoistureChannel, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
    let parts = raw.split('-').collect::<Vec<_>>();

    if parts.len() == 2 {
        let i2c_address = u8::from_str_radix(parts[0], 16).map_err(|e| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&raw),
                &format!("a valid hexadecimal integer: {}", e).as_str(),
            )
        })?;
        let analog_pin = u8::from_str_radix(parts[1], 10).map_err(|e| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&raw),
                &format!("a valid decimal integer: {}", e).as_str(),
            )
        })?;
        Ok(MoistureChannel {
            i2c_address,
            analog_pin,
        })
    } else {
        Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(&raw),
            &"a hexadecimal integer, a dash '-', and a decimal integer, like \"e8-3\"",
        ))
    }
}
