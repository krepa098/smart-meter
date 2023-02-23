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

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[allow(unused)]
pub struct MeasurementInfo {
    pub device_id: i32,
    pub from_timestamp: i64,
    pub to_timestamp: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl Default for MeasurementMask {
    fn default() -> Self {
        Self(1)
    }
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

    use super::*;

    fn api_url(endpoint: &str) -> String {
        let host_url = host_url();
        format!("{host_url}/{endpoint}")
    }

    fn host_url() -> String {
        let location = web_sys::window().unwrap().location();
        format!(
            "{}//{}:8081",
            location.protocol().unwrap(),
            location.hostname().unwrap()
        )
    }

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
            .get(api_url("api/measurements/by_date"))
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
            .get(api_url("api/devices"))
            .header(ACCEPT, "application/json")
            .send()
            .await
            .unwrap()
            .json::<Vec<DeviceInfo>>()
            .await
            .unwrap();

        resp
    }

    pub async fn measurement_info(device_id: u32) -> MeasurementInfo {
        let client = reqwest::Client::new();

        let resp = client
            .get(api_url("api/measurements/info"))
            .query(&[("device_id", device_id as i64)])
            .header(ACCEPT, "application/json")
            .send()
            .await
            .unwrap()
            .json::<MeasurementInfo>()
            .await
            .unwrap();

        resp
    }
}
