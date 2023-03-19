use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[allow(unused)]
pub struct MeasurementInfo {
    pub device_id: i32,
    pub from_timestamp: i64,
    pub to_timestamp: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub struct MeasurementMask(pub u32);

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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[allow(unused)]
pub struct MeasurementRequestResponse {
    pub device_id: i32,
    pub timestamps: Vec<i64>,
    pub data: HashMap<u32, Vec<f32>>,
}
