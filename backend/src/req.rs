use std::collections::HashMap;

#[allow(unused)]
#[derive(Debug, serde::Serialize)]
#[repr(u32)]
pub enum MeasurementType {
    Temperature = 1 << 0,
    Humidity = 1 << 1,
    Pressure = 1 << 2,
    BatCapacity = 1 << 3,
    BatVoltage = 1 << 4,
    AirQuality = 1 << 5,
}

#[derive(Debug, serde::Serialize)]
#[allow(unused)]
pub struct MeasurementRequestResponse {
    pub device_id: i32,
    pub timestamps: Vec<i64>,
    pub data: HashMap<u32, Vec<f32>>,
}
