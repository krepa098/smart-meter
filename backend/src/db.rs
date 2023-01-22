use crate::schema::*;
use anyhow::Result;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Default, Insertable)]
#[diesel(table_name=measurements)]
#[allow(unused)]
pub struct NewDeviceMeasurement {
    pub device_id: i32,
    pub timestamp: i64,
    pub temperature: Option<f32>, // °C
    pub pressure: Option<f32>,    // hPa
    pub humidity: Option<f32>,    // percent
    pub air_quality: Option<f32>, // ohm
    pub v_bat: Option<f32>,       // V
}

#[derive(Debug, Queryable, serde::Serialize)]
#[allow(unused)]
pub struct DeviceMeasurement {
    pub id: i32,
    pub device_id: i32,
    pub timestamp: i64,
    pub temperature: Option<f32>, // °C
    pub pressure: Option<f32>,    // hPa
    pub humidity: Option<f32>,    // percent
    pub air_quality: Option<f32>, // ohm
    pub v_bat: Option<f32>,       // V
}

pub struct Db {
    conn: SqliteConnection,
}

impl Db {
    pub fn connect() -> Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)?;

        Ok(Self { conn })
    }

    pub fn insert_measurement(&mut self, mes: &NewDeviceMeasurement) -> Result<()> {
        println!("Insert into db!");

        diesel::insert_into(measurements::table)
            .values(mes)
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn all_measurements(&mut self) -> Result<Vec<DeviceMeasurement>> {
        use crate::schema::measurements::dsl::*;
        let res = measurements
            .limit(10)
            .order(id.desc())
            .load::<DeviceMeasurement>(&mut self.conn)?;

        Ok(res)
    }
}
