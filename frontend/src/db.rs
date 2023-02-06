// keep in sync with db.rs of backend

#[derive(Debug, serde::Deserialize)]
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
