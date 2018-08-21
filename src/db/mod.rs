use chrono;
use failure;
use influent;
use serde_json;
use slog;
use uuid;

pub mod model;

pub struct Db<'a> {
    client: influent::client::http::HttpClient<'a>,
}

impl<'a> Db<'a> {
    pub fn connect(
        _log: slog::Logger,
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

        Ok(Db { client })
    }

    pub fn insert_global_measurement(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        temperature: f64,
        pressure: f64,
    ) -> Result<(), failure::Error> {
        use influent::client::Client;

        let mut measurement = influent::measurement::Measurement::new("global");
        measurement.set_timestamp(to_influx_timestamp(now));
        measurement.add_field(
            "temperature",
            influent::measurement::Value::Float(temperature),
        );
        measurement.add_field(
            "pressure",
            influent::measurement::Value::Float(pressure),
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

    pub fn update_plant_indices(&self) -> Result<(), failure::Error> {
        use influent::client::Client;

        self.client
            .query(
                "select \
                 percentile(moisture, 5) as moisture_p05, \
                 percentile(moisture, 95) as moisture_p95 \
                 into plant_index from plant where time > now() - 1w group by uuid"
                    .to_owned(),
                Some(influent::client::Precision::Nanoseconds),
            ).map_err(from_influent_error)?;

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
                    "select moisture_p05 as lo, moisture_p95 as hi from plant_index where uuid = '{}'",
                    m_id
                ),
                Some(influent::client::Precision::Nanoseconds),
            )
            .map_err(from_influent_error)?;

        let results: QueryResults = serde_json::de::from_str(&results)?;

        Ok(results
            .results
            .into_iter()
            .find(|r| r.statement_id == Some(0))
            .and_then(|result| result.series.into_iter().find(|s| s.name == "plant"))
            .map_or_else(
                || (None, None),
                |series| {
                    (
                        series
                            .columns
                            .iter()
                            .position(|c| c == "lo")
                            .map(|i| series.values[0][i]),
                        series
                            .columns
                            .iter()
                            .position(|c| c == "hi")
                            .map(|i| series.values[0][i]),
                    )
                },
            ))
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
    statement_id: Option<u32>,
    #[serde(default)]
    series: Vec<QuerySeries>,
}

#[derive(Clone, Debug, Deserialize)]
struct QuerySeries {
    name: String,
    columns: Vec<String>,
    values: Vec<Vec<f64>>,
}
