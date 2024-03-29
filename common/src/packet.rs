const MAGIC: &str = "BRST";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Header {
    magic: String,
    pub device_id: u32,
    pub timestamp: u64,     // ms since boot
    pub rel_timestamp: i64, // ms since packet was sent
}

#[allow(unused)]
impl Header {
    pub fn new(device_id: u32, timestamp: u64) -> Self {
        Self {
            magic: MAGIC.to_string(),
            device_id,
            timestamp,
            rel_timestamp: 0,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Packet {
    pub header: Header,
    pub payload: Payload,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Payload {
    Measurement(Measurement),
    DeviceInfo(DeviceInfo),
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[allow(unused)]
pub struct Measurement {
    pub temperature: Option<f32>,  // °C
    pub pressure: Option<f32>,     // Pa
    pub humidity: Option<f32>,     // percent
    pub air_quality: Option<f32>,  // ohm
    pub bat_voltage: Option<f32>,  // V
    pub bat_capacity: Option<f32>, // percent
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[allow(unused)]
pub struct DeviceInfo {
    pub uptime: u64,                 // seconds
    pub report_interval: u64,        // seconds
    pub sample_interval: u64,        // seconds
    pub firmware_version: [u8; 4],   // major.minor.bugfix.misc
    pub bsec_version: [u8; 4],       // major.minor.bugfix.misc
    pub model: [u8; 16],             // utf8 string
    pub wifi_ssid: Option<[u8; 32]>, // utf8 string (last connected wifi)
}
