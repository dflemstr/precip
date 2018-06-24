use chrono;
use diesel;
use failure;
use r2d2;
use r2d2_diesel;
use slog;
use uuid;

use diesel::prelude::*;

pub mod model;
pub mod schema;

pub struct Db(r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>);

impl Db {
    pub fn connect(log: slog::Logger, url: &str) -> Result<Self, failure::Error> {
        debug!(log, "connecting to DB url: {:?}", url);
        let pool = r2d2::Pool::builder().build(r2d2_diesel::ConnectionManager::new(url))?;

        // Check that we can get a connection; fail early
        pool.get()?;
        debug!(log, "connected");

        Ok(Db(pool))
    }

    pub fn ensure_module(
        &self,
        uuid: uuid::Uuid,
        name: &str,
    ) -> Result<model::Module, failure::Error> {
        let conn = self.0.get()?;
        let new_module = model::NewModule {
            uuid,
            name: name.to_owned(),
        };

        let result = diesel::insert_into(schema::module::table)
            .values(&new_module)
            .on_conflict(schema::module::uuid)
            .do_update()
            .set(&new_module)
            .get_result(&*conn)?;

        Ok(result)
    }

    pub fn insert_sample(
        &self,
        module_id: i32,
        created: chrono::DateTime<chrono::Utc>,
        moisture: f64,
        raw_voltage: f64,
    ) -> Result<(), failure::Error> {
        let conn = self.0.get()?;
        let new_sample = model::NewSample {
            module_id,
            created,
            moisture,
            raw_voltage,
        };

        diesel::insert_into(schema::sample::table)
            .values(&new_sample)
            .execute(&*conn)?;

        Ok(())
    }

    pub fn insert_global_sample(
        &self,
        created: chrono::DateTime<chrono::Utc>,
        temperature: f64,
    ) -> Result<(), failure::Error> {
        let conn = self.0.get()?;
        let new_sample = model::NewGlobalSample {
            created,
            temperature,
        };

        diesel::insert_into(schema::global_sample::table)
            .values(&new_sample)
            .execute(&*conn)?;

        Ok(())
    }

    pub fn insert_pump_event(
        &self,
        module_id: i32,
        created: chrono::DateTime<chrono::Utc>,
        pump_running: bool,
    ) -> Result<(), failure::Error> {
        let conn = self.0.get()?;
        let new_pump_event = model::NewPumpEvent {
            module_id,
            created,
            pump_running,
        };

        diesel::insert_into(schema::pump_event::table)
            .values(&new_pump_event)
            .execute(&*conn)?;

        Ok(())
    }

    pub fn fetch_module_moisture_voltage_range(
        &self,
        m_id: i32,
    ) -> Result<(Option<f64>, Option<f64>), failure::Error> {
        use db::schema::sample::dsl::*;
        use diesel::dsl::*;

        let conn = self.0.get()?;

        let min_value = sample
            .select(min(raw_voltage))
            .filter(module_id.eq(m_id))
            .first(&*conn)?;
        let max_value = sample
            .select(max(raw_voltage))
            .filter(module_id.eq(m_id))
            .first(&*conn)?;

        Ok((min_value, max_value))
    }

    pub fn collect_samples_range(&self) -> Result<Vec<model::SampleRange>, failure::Error> {
        let conn = self.0.get()?;
        let result = diesel::sql_query(include_str!("sample_range_query.sql")).load(&*conn)?;
        Ok(result)
    }

    pub fn collect_samples_timeseries(
        &self,
    ) -> Result<Vec<model::SampleTimeseries>, failure::Error> {
        let conn = self.0.get()?;
        let result = diesel::sql_query(include_str!("samples_timeseries_query.sql")).load(&*conn)?;
        Ok(result)
    }

    pub fn collect_pump_events(&self) -> Result<Vec<model::PumpEvent>, failure::Error> {
        let conn = self.0.get()?;
        let result = diesel::sql_query(include_str!("pump_event_query.sql")).load(&*conn)?;
        Ok(result)
    }

    pub fn collect_stats(&self) -> Result<Vec<model::Stats>, failure::Error> {
        let conn = self.0.get()?;
        let result = diesel::sql_query(include_str!("stats_query.sql")).load(&*conn)?;
        Ok(result)
    }

    pub fn collect_global_stats(&self) -> Result<model::GlobalStats, failure::Error> {
        let conn = self.0.get()?;
        let result = diesel::sql_query(include_str!("global_stats_query.sql")).get_result(&*conn)?;
        Ok(result)
    }
}
