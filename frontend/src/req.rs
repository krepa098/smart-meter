// keep in sync with db.rs of backend
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
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

#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone, Copy)]
#[repr(u32)]
pub enum MeasurementType {
    Temperature = 1 << 0,
    Humidity = 1 << 1,
    Pressure = 1 << 2,
    BatCapacity = 1 << 3,
    BatVoltage = 1 << 4,
    AirQuality = 1 << 5,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[allow(unused)]
pub struct MeasurementMask(u32);

impl MeasurementMask {
    pub fn is_set(&self, other: MeasurementType) -> bool {
        self.0 & other as u32 > 0
    }

    pub fn set(&mut self, other: MeasurementType, active: bool) {
        if active {
            self.0 |= other as u32;
        } else {
            self.0 &= !(other as u32);
        }
    }

    pub const ALL: Self = Self(0xFFFFFFFF);
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct MeasurementRequestResponse {
    pub device_id: i32,
    pub timestamps: Vec<i64>,
    pub data: HashMap<u32, Vec<f32>>,
}

// ===============================================
// helpers
// ===============================================
pub mod request {
    use chrono::{DateTime, Utc};
    use reqwest::header::ACCEPT;

    use super::{DeviceInfo, MeasurementMask, MeasurementRequestResponse};

    pub async fn measurements(
        device_id: u32,
        ts_from: Option<DateTime<Utc>>,
        ts_to: Option<DateTime<Utc>>,
        measurement_mask: MeasurementMask,
        limit: i64,
    ) -> MeasurementRequestResponse {
        let client = reqwest::Client::new();

        let mut query = vec![
            ("device_id", device_id as i64),
            ("limit", limit),
            ("measurement_types", measurement_mask.0 as i64),
        ];
        if let Some(date) = ts_from {
            query.push(("from_date", date.timestamp_millis()))
        }
        if let Some(date) = ts_to {
            query.push(("to_date", date.timestamp_millis()))
        }

        let resp = client
            .get("http://127.0.0.1:8081/api/measurements/by_date")
            .query(&query)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .unwrap()
            .json::<MeasurementRequestResponse>()
            .await
            .unwrap();
        resp
    }

    pub async fn device_infos() -> Vec<DeviceInfo> {
        let client = reqwest::Client::new();

        let resp = client
            .get("http://127.0.0.1:8081/api/devices")
            .header(ACCEPT, "application/json")
            .send()
            .await
            .unwrap()
            .json::<Vec<DeviceInfo>>()
            .await
            .unwrap();

        resp
    }
}
