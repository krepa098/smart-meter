const MAGIC: &str = "M1S1";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Header {
    magic: String,
    pub device_id: u32,
}

impl Header {
    pub fn with_device_id(device_id: u32) -> Self {
        Self {
            magic: MAGIC.to_string(),
            device_id,
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
    Measurements(Measurement),
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[allow(unused)]
pub struct Measurement {
    pub timestamp: u64,
    pub temperature: Option<f32>, // °C
    pub pressure: Option<f32>,    // hPa
    pub humidity: Option<f32>,    // percent
    pub air_quality: Option<f32>, // ohm
    pub v_bat: Option<f32>,       // V
}
