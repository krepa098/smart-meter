// keep in sync with db.rs of backend
use anyhow::Result;
use chrono::{DateTime, Utc};
use common::req::*;
use reqwest::header::ACCEPT;

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
) -> Result<MeasurementRequestResponse> {
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

    Ok(client
        .get(api_url("api/measurements/by_date"))
        .query(&query)
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<MeasurementRequestResponse>()
        .await?)
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

    client
        .get(api_url("api/measurements/info"))
        .query(&[("device_id", device_id as i64)])
        .header(ACCEPT, "application/json")
        .send()
        .await
        .unwrap()
        .json::<MeasurementInfo>()
        .await
        .unwrap()
}

pub async fn device_name(device_id: u32) -> Result<String> {
    let client = reqwest::Client::new();

    let res = client
        .get(api_url("api/device_name"))
        .query(&[("device_id", device_id as i64)])
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<String>()
        .await?;

    Ok(res)
}

pub async fn set_device_name(device_id: u32, name: String) -> Result<()> {
    let client = reqwest::Client::new();

    let res = client
        .put(api_url("api/device_name"))
        .query(&[("device_id", device_id.to_string()), ("name", name)])
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .status();

    Ok(())
}
