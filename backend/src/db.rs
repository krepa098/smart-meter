use crate::{schema::*, utils};
use anyhow::{bail, Result};
use common::req;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;

use self::models::DeviceName;

pub mod models {
    use super::*;

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

    #[derive(Debug, Default, Insertable, Queryable)]
    #[diesel(table_name=device_names)]
    #[allow(unused)]
    pub struct DeviceName {
        pub device_id: i32,
        pub name: String,
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
}

pub struct Db {
    conn: SqliteConnection,
}

impl Db {
    pub fn connect() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)?;

        Ok(Self { conn })
    }

    pub fn insert_measurement(&mut self, mes: &models::NewDeviceMeasurement) -> Result<()> {
        println!("Insert into db!");

        diesel::insert_into(measurements::table)
            .values(mes)
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn update_device_info(&mut self, info: &models::DeviceInfo) -> Result<()> {
        diesel::replace_into(devices::table)
            .values(info)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn update_device_name(&mut self, device_name: &models::DeviceName) -> Result<()> {
        diesel::replace_into(device_names::table)
            .values(device_name)
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn device_name(&mut self, device_id: u32) -> Result<DeviceName> {
        use crate::schema::device_names::dsl;
        let device_name = dsl::device_names
            .filter(dsl::device_id.eq(device_id as i32))
            .first::<models::DeviceName>(&mut self.conn)?;

        Ok(device_name)
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

        let total_entries: i64 = measurements
            .filter(device_id.eq(dev_id as i32))
            .filter(timestamp.ge(from_date.unwrap_or(0) as i64))
            .filter(timestamp.le(to_date.unwrap_or(utils::ms_since_epoch() as u64) as i64))
            .count()
            .get_result(&mut self.conn)?;

        let div = (total_entries as f32 / limit as f32).ceil() as usize;

        let res = measurements
            .filter(device_id.eq(dev_id as i32))
            .filter(timestamp.ge(from_date.unwrap_or(0) as i64))
            .filter(timestamp.le(to_date.unwrap_or(utils::ms_since_epoch() as u64) as i64))
            .order(id.desc())
            .limit(limit as i64)
            .load::<models::DeviceMeasurement>(&mut self.conn)?;

        // TODO: integrate this into the query somehow
        let res: Vec<_> = res.iter().step_by(div).collect();

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

    pub fn all_measurements(&mut self) -> Result<Vec<models::DeviceMeasurement>> {
        use crate::schema::measurements::dsl::*;
        let res = measurements
            .limit(100)
            .order(id.desc())
            .load::<models::DeviceMeasurement>(&mut self.conn)?;

        Ok(res)
    }

    pub fn measurement_info(&mut self, dev_id: u32) -> Result<(i64, i64, i64)> {
        use crate::schema::measurements::dsl::*;
        let oldest_entry = measurements
            .filter(device_id.eq(dev_id as i32))
            .order(id.asc())
            .first::<models::DeviceMeasurement>(&mut self.conn)?;
        let most_recent_entry = measurements
            .filter(device_id.eq(dev_id as i32))
            .order(id.desc())
            .first::<models::DeviceMeasurement>(&mut self.conn)?;

        let count = measurements
            .filter(device_id.eq(dev_id as i32))
            .count()
            .get_result::<i64>(&mut self.conn)?;

        Ok((oldest_entry.timestamp, most_recent_entry.timestamp, count))
    }

    pub fn devices(&mut self) -> Result<Vec<models::DeviceInfo>> {
        use crate::schema::devices::dsl;
        let devices = dsl::devices.load::<models::DeviceInfo>(&mut self.conn)?;

        Ok(devices)
    }
}
