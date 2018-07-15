use std::char;
use std::collections;
use std::u8;

use influent;
use serde;
use uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub db: Db,
    pub plant: collections::HashMap<uuid::Uuid, Plant>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Db {
    pub hosts: Vec<String>,
    pub credentials: DbCredentials,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DbCredentials {
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Plant {
    pub name: String,
    pub description: String,
    pub moisture: Moisture,
    pub pump: Pump,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Moisture {
    #[serde(deserialize_with = "deserialize_moisture_channel")]
    pub channel: MoistureChannel,
    #[serde(deserialize_with = "deserialize_distance")]
    pub distance: f64,
    pub voltage_dry: f64,
    pub voltage_wet: f64,
    // in meters
    pub min: f64,
    pub max: f64,
}

#[derive(Clone, Debug)]
pub struct MoistureChannel {
    pub i2c_address: u16,
    pub analog_pin: u8,
}

#[derive(Clone, Debug, Deserialize)]
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
        let i2c_address = u16::from_str_radix(parts[0], 16).map_err(|e| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(parts[0]),
                &format!("a valid hexadecimal integer: {}", e).as_str(),
            )
        })?;
        let analog_pin = u8::from_str_radix(parts[1], 10).map_err(|e| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(parts[1]),
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

fn deserialize_distance<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use std::str::FromStr;

    let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
    if let Some(unit_start) = raw.find(char::is_alphabetic) {
        let (number_str, unit_str) = raw.split_at(unit_start);
        let number = f64::from_str(number_str).map_err(|e| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(number_str),
                &format!("a valid number: {}", e).as_str(),
            )
        })?;
        let result = match unit_str {
            "km" => number * 1000.0,
            "hm" => number * 100.0,
            "dam" => number * 10.0,
            "m" => number,
            "dm" => number / 10.0,
            "cm" => number / 100.0,
            "mm" => number / 1000.0,
            "µm" => number / 1000.0,
            "mum" => number / 1000.0,
            "nm" => number / 1000_000.0,
            "pm" => number / 1000_000_000.0,
            other => {
                return Err(
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(other),
                        &format!("a recognized metric unit (e.g. km, hm, dam, m, dm, cm, mm, µm, mum, nm, pm)").as_str(),
                    ));
            }
        };
        Ok(result)
    } else {
        Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(&raw),
            &"a number with a length unit, such as \"2.3cm\"",
        ))
    }
}

impl From<DbCredentials> for influent::client::Credentials<'static> {
    fn from(credentials: DbCredentials) -> Self {
        influent::client::Credentials {
            username: leak_static_str(credentials.username),
            password: leak_static_str(credentials.password),
            database: leak_static_str(credentials.database),
        }
    }
}

fn leak_static_str(s: String) -> &'static str {
    // TODO: this is a hack due to an annoyance in the influent API.  With async/await in std, this
    // should be avoidable, due to better lifetime support in async code.
    unsafe {
        let ret = ::std::mem::transmute(&s as &str);
        ::std::mem::forget(s);
        ret
    }
}
