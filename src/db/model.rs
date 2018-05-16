use chrono;
use uuid;

use diesel::sql_types::Float8;
use diesel::sql_types::Integer;
use diesel::sql_types::Timestamptz;

use super::schema::module;
use super::schema::sample;

#[derive(Debug, Identifiable, Associations, Queryable)]
#[table_name = "module"]
pub struct Module {
    pub id: i32,
    pub uuid: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "module"]
pub struct NewModule {
    pub uuid: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Identifiable, Associations, Queryable)]
#[table_name = "sample"]
pub struct Sample {
    pub id: i32,
    pub created: chrono::DateTime<chrono::Utc>,
    pub module_id: i32,
    pub humidity: f64,
    pub temperature: f64,
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "sample"]
pub struct NewSample {
    pub created: chrono::DateTime<chrono::Utc>,
    pub module_id: i32,
    pub humidity: f64,
    pub temperature: f64,
}

#[derive(Debug, QueryableByName)]
pub struct Stats {
    #[sql_type = "Integer"]
    pub module_id: i32,
    #[sql_type = "Timestamptz"]
    pub slice: chrono::DateTime<chrono::Utc>,
    #[sql_type = "Float8"]
    pub min_humidity: f64,
    #[sql_type = "Float8"]
    pub max_humidity: f64,
    #[sql_type = "Float8"]
    pub p25_humidity: f64,
    #[sql_type = "Float8"]
    pub p50_humidity: f64,
    #[sql_type = "Float8"]
    pub p75_humidity: f64,
    #[sql_type = "Float8"]
    pub min_temperature: f64,
    #[sql_type = "Float8"]
    pub max_temperature: f64,
    #[sql_type = "Float8"]
    pub p25_temperature: f64,
    #[sql_type = "Float8"]
    pub p50_temperature: f64,
    #[sql_type = "Float8"]
    pub p75_temperature: f64,
}
