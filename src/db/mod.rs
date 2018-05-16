use chrono;
use diesel;
use failure;
use r2d2;
use r2d2_diesel;
use uuid;

use diesel::prelude::*;

pub mod model;
pub mod schema;

pub struct Db(r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>);

impl Db {
    pub fn connect(url: &str) -> Result<Self, failure::Error> {
        debug!("connecting to DB url: {:?}", url);
        let pool = r2d2::Pool::builder().build(r2d2_diesel::ConnectionManager::new(url))?;

        // Check that we can get a connection; fail early
        pool.get()?;
        debug!("connected");

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
        humidity: f64,
        temperature: f64,
    ) -> Result<(), failure::Error> {
        let conn = self.0.get()?;
        let new_sample = model::NewSample {
            module_id,
            created,
            humidity,
            temperature,
        };

        diesel::insert_into(schema::sample::table)
            .values(&new_sample)
            .execute(&*conn)?;

        Ok(())
    }

    pub fn collect_stats(&self) -> Result<Vec<model::Stats>, failure::Error> {
        let conn = self.0.get()?;
        let result = diesel::sql_query(include_str!("stats_query.sql")).load(&*conn)?;
        Ok(result)
    }
}
