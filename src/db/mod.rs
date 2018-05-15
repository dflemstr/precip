use chrono;
use diesel;
use failure;
use uuid;

use diesel::prelude::*;

pub mod model;
pub mod schema;

pub struct Db(diesel::pg::PgConnection);

impl Db {
    pub fn connect(url: &str) -> Result<Self, failure::Error> {
        info!("connecting to DB url: {:?}", url);
        let connection = diesel::pg::PgConnection::establish(url)?;
        Ok(Db(connection))
    }

    pub fn ensure_module(
        &self,
        uuid: uuid::Uuid,
        name: &str,
    ) -> Result<model::Module, failure::Error> {
        let new_module = model::NewModule {
            uuid,
            name: name.to_owned(),
        };

        let result = diesel::insert_into(schema::module::table)
            .values(&new_module)
            .on_conflict(schema::module::uuid)
            .do_update()
            .set(&new_module)
            .get_result(&self.0)?;

        Ok(result)
    }

    pub fn insert_sample(
        &self,
        module_id: i32,
        created: chrono::DateTime<chrono::Utc>,
        humidity: f64,
        temperature: f64,
    ) -> Result<(), failure::Error> {
        let new_sample = model::NewSample {
            module_id,
            created,
            humidity,
            temperature,
        };

        diesel::insert_into(schema::sample::table)
            .values(&new_sample)
            .execute(&self.0)?;

        Ok(())
    }

    pub fn collect_stats(&self) -> Result<Vec<model::Stats>, failure::Error> {
        let result = diesel::sql_query(include_str!("stats_query.sql")).load(&self.0)?;
        Ok(result)
    }
}
