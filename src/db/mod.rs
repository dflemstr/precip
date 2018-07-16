use chrono;
use failure;
use influent;
use serde_json;
use slog;
use uuid;

pub mod model;

pub struct Db<'a> {
    log: slog::Logger,
    client: influent::client::http::HttpClient<'a>,
}

impl<'a> Db<'a> {
    pub fn connect(
        log: slog::Logger,
        credentials: influent::client::Credentials<'a>,
        hosts: Vec<String>,
    ) -> Result<Self, failure::Error> {
        let serializer = influent::serializer::line::LineSerializer::new();
        let mut client = influent::client::http::HttpClient::new(
            credentials,
            Box::new(serializer),
            Box::new(influent::hurl::hyper::HyperHurl::new()),
        );

        for host in hosts {
            client.add_host(leak_static_str(host));
        }

        Ok(Db { log, client })
    }

    pub fn insert_global_measurement(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        temperature: f64,
    ) -> Result<(), failure::Error> {
        use influent::client::Client;

        let mut measurement = influent::measurement::Measurement::new("global");
        measurement.set_timestamp(to_influx_timestamp(now));
        measurement.add_field(
            "temperature",
            influent::measurement::Value::Float(temperature),
        );

        self.client
            .write_one(measurement, Some(influent::client::Precision::Nanoseconds))
            .map_err(from_influent_error)?;

        Ok(())
    }

    pub fn insert_plant_measurement(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        uuid: uuid::Uuid,
        moisture: f64,
    ) -> Result<(), failure::Error> {
        use influent::client::Client;

        let mut measurement = influent::measurement::Measurement::new("plant");
        measurement.set_timestamp(to_influx_timestamp(now));
        measurement.add_tag("uuid", uuid.hyphenated().to_string());
        measurement.add_field("moisture", influent::measurement::Value::Float(moisture));

        self.client
            .write_one(measurement, Some(influent::client::Precision::Nanoseconds))
            .map_err(from_influent_error)?;

        Ok(())
    }

    pub fn insert_pump_measurement(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        uuid: uuid::Uuid,
        running: bool,
    ) -> Result<(), failure::Error> {
        use influent::client::Client;

        let mut measurement = influent::measurement::Measurement::new("pump");
        measurement.set_timestamp(to_influx_timestamp(now));
        measurement.add_tag("uuid", uuid.hyphenated().to_string());
        measurement.add_field("running", influent::measurement::Value::Boolean(running));

        self.client
            .write_one(measurement, Some(influent::client::Precision::Nanoseconds))
            .map_err(from_influent_error)?;

        Ok(())
    }

    pub fn fetch_module_moisture_voltage_range(
        &self,
        m_id: uuid::Uuid,
    ) -> Result<(Option<f64>, Option<f64>), failure::Error> {
        use influent::client::Client;

        let results = self
            .client
            .query(
                format!(
                    "select min(moisture), max(moisture) from plant where uuid = '{}'",
                    m_id
                ),
                Some(influent::client::Precision::Nanoseconds),
            )
            .map_err(from_influent_error)?;

        let results: QueryResults = serde_json::de::from_str(&results)?;
        info!(self.log, "Query results: {:?}", results);

        match results.results.as_slice() {
            [result] => match result.series.as_slice() {
                [series] => match series.values.as_slice() {
                    [value] => match value.as_slice() {
                        [_, min, max] => Ok((Some(*min), Some(*max))),
                        _ => Ok((None, None)),
                    },
                    _ => Ok((None, None)),
                },
                _ => Ok((None, None)),
            },
            _ => Ok((None, None)),
        }
    }

    pub fn collect_samples_range(&self) -> Result<Vec<model::SampleRange>, failure::Error> {
        Ok(Vec::new())
    }

    pub fn collect_samples_timeseries(
        &self,
    ) -> Result<Vec<model::SampleTimeseries>, failure::Error> {
        Ok(Vec::new())
    }

    pub fn collect_pump_events(&self) -> Result<Vec<model::PumpEvent>, failure::Error> {
        Ok(Vec::new())
    }

    pub fn collect_stats(&self) -> Result<Vec<model::Stats>, failure::Error> {
        Ok(Vec::new())
    }

    pub fn collect_global_stats(&self) -> Result<model::GlobalStats, failure::Error> {
        Ok(model::GlobalStats { temperature: 0.0 })
    }
}

fn to_influx_timestamp<Tz>(t: chrono::DateTime<Tz>) -> i64
where
    Tz: chrono::TimeZone,
{
    t.timestamp() * 1_000_000_000 + t.timestamp_subsec_nanos() as i64
}

fn from_influent_error(err: influent::client::ClientError) -> failure::Error {
    match err {
        influent::client::ClientError::CouldNotComplete(m) => {
            failure::err_msg(format!("could not complete: {}", m))
        }
        influent::client::ClientError::Communication(m) => {
            failure::err_msg(format!("communication error: {}", m))
        }
        influent::client::ClientError::Syntax(m) => {
            failure::err_msg(format!("syntax error: {}", m))
        }
        influent::client::ClientError::Unexpected(m) => {
            failure::err_msg(format!("unexpected error: {}", m))
        }
        influent::client::ClientError::Unknown => failure::err_msg("unknown error"),
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

#[derive(Clone, Debug, Deserialize)]
struct QueryResults {
    results: Vec<QueryResult>,
}

#[derive(Clone, Debug, Deserialize)]
struct QueryResult {
    statement_id: u32,
    series: Vec<QuerySeries>,
}

#[derive(Clone, Debug, Deserialize)]
struct QuerySeries {
    name: String,
    columns: Vec<String>,
    values: Vec<Vec<f64>>,
}
