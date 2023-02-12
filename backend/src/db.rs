use crate::{req, schema::*, utils};
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
    pub timestamp: i64,           // ms since epoch
    pub temperature: Option<f32>, // °C
    pub humidity: Option<f32>,    // percent
    pub pressure: Option<f32>,    // hPa
    pub air_quality: Option<f32>, // ohm
    pub bat_v: Option<f32>,       // V
    pub bat_cap: Option<f32>,     // percent
}

#[derive(Debug, Queryable, serde::Serialize)]
#[allow(unused)]
pub struct DeviceMeasurement {
    pub id: i32,
    pub device_id: i32,
    pub timestamp: i64,           // ms since epoch
    pub temperature: Option<f32>, // °C
    pub humidity: Option<f32>,    // percent
    pub pressure: Option<f32>,    // hPa
    pub air_quality: Option<f32>, // ohm
    pub bat_v: Option<f32>,       // V
    pub bat_cap: Option<f32>,     // percent
}

#[derive(Debug, Default, Insertable, Queryable, serde::Serialize)]
#[diesel(table_name=devices)]
#[allow(unused)]
pub struct DeviceInfo {
    pub device_id: i32, // unique, key
    pub fw_version: String,
    pub bsec_version: String,
    pub wifi_ssid: Option<String>,
    pub uptime: i32,          // s
    pub report_interval: i32, // s
    pub sample_interval: i32, // s
    pub last_seen: i64,       // s
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

    pub fn update_device_info(&mut self, info: &DeviceInfo) -> Result<()> {
        diesel::replace_into(devices::table)
            .values(info)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn measurements_by_date(
        &mut self,
        dev_id: u32,
        from_date: Option<u64>,
        to_date: Option<u64>,
        measurement_type: u32,
        limit: u32,
    ) -> Result<req::MeasurementRequestResponse> {
        use crate::schema::measurements::dsl::*;
        let res = measurements
            .filter(device_id.eq(dev_id as i32))
            .filter(timestamp.ge(from_date.unwrap_or(0) as i64))
            .filter(timestamp.le(to_date.unwrap_or(utils::ms_since_epoch() as u64) as i64))
            .order(id.desc())
            .limit(limit as i64)
            .load::<DeviceMeasurement>(&mut self.conn)?;

        // filter requested measurements
        let mut data = std::collections::HashMap::new();
        if measurement_type & req::MeasurementType::Temperature as u32 > 0 {
            data.insert(
                req::MeasurementType::Temperature as u32,
                res.iter()
                    .map(|p| p.temperature.unwrap_or(f32::NAN))
                    .collect(),
            );
        }
        if measurement_type & req::MeasurementType::Pressure as u32 > 0 {
            data.insert(
                req::MeasurementType::Pressure as u32,
                res.iter().map(|p| p.pressure.unwrap_or(f32::NAN)).collect(),
            );
        }
        if measurement_type & req::MeasurementType::Humidity as u32 > 0 {
            data.insert(
                req::MeasurementType::Humidity as u32,
                res.iter().map(|p| p.humidity.unwrap_or(f32::NAN)).collect(),
            );
        }
        if measurement_type & req::MeasurementType::BatCapacity as u32 > 0 {
            data.insert(
                req::MeasurementType::BatCapacity as u32,
                res.iter().map(|p| p.bat_cap.unwrap_or(f32::NAN)).collect(),
            );
        }
        if measurement_type & req::MeasurementType::BatVoltage as u32 > 0 {
            data.insert(
                req::MeasurementType::BatVoltage as u32,
                res.iter().map(|p| p.bat_v.unwrap_or(f32::NAN)).collect(),
            );
        }
        if measurement_type & req::MeasurementType::AirQuality as u32 > 0 {
            data.insert(
                req::MeasurementType::AirQuality as u32,
                res.iter()
                    .map(|p| p.air_quality.unwrap_or(f32::NAN))
                    .collect(),
            );
        }

        let resp = req::MeasurementRequestResponse {
            device_id: dev_id as i32,
            timestamps: res.iter().map(|p| p.timestamp).collect(),
            data,
        };

        Ok(resp)
    }

    pub fn all_measurements(&mut self) -> Result<Vec<DeviceMeasurement>> {
        use crate::schema::measurements::dsl::*;
        let res = measurements
            .limit(100)
            .order(id.desc())
            .load::<DeviceMeasurement>(&mut self.conn)?;

        Ok(res)
    }

    pub fn devices(&mut self) -> Result<Vec<DeviceInfo>> {
        use crate::schema::devices::dsl;
        let devices = dsl::devices.load::<DeviceInfo>(&mut self.conn)?;

        Ok(devices)
    }
}
