use std::time::Duration;

use reqwest::header::ACCEPT;
use yew::{function_component, html, use_state};

use crate::utils;

#[function_component(Devices)]
pub fn device_list() -> Html {
    let devices = use_state(|| None);
    let latest_device_measurements = use_state(|| None);
    let client = reqwest::Client::new();

    // requests
    {
        let devices = devices.clone();
        let client = client.clone();

        if devices.is_none() {
            wasm_bindgen_futures::spawn_local(async move {
                let resp = client
                    .get("http://127.0.0.1:8081/api/devices")
                    .header(ACCEPT, "application/json")
                    .send()
                    .await
                    .unwrap()
                    .json::<Vec<serde_json::Map<String, serde_json::Value>>>()
                    .await
                    .unwrap();
                devices.set(Some(resp));
            });
        }
    }
    {
        let devices = devices.clone();
        let latest_device_measurements = latest_device_measurements.clone();
        let client = client.clone();

        if latest_device_measurements.is_none() && devices.is_some() {
            wasm_bindgen_futures::spawn_local(async move {
                let mut measurements = std::collections::HashMap::new();

                for dev in devices.as_ref().unwrap() {
                    let id = dev.get("device_id").unwrap().as_u64().unwrap();
                    let resp = client
                        .get("http://127.0.0.1:8081/api/measurements/by_date")
                        .query(&[
                            ("device_id", id.to_string().to_owned()),
                            ("limit", "1".to_owned()),
                        ])
                        .header(ACCEPT, "application/json")
                        .send()
                        .await
                        .unwrap()
                        .json::<Vec<serde_json::Map<String, serde_json::Value>>>()
                        .await
                        .unwrap();

                    measurements.insert(id, resp);
                }

                latest_device_measurements.set(Some(measurements));
            });
        }
    }

    let device_list = match devices.as_ref() {
        Some(devices) => devices
            .iter()
            .map(|dev| {
                let device_id = dev.get("device_id").unwrap().as_u64().unwrap();
                let last_seen = utils::duration_since_epoch(dev.get("last_seen").unwrap().as_u64().unwrap());
                let is_online = last_seen.as_secs() < 60 *15;
                let uptime = humantime::format_duration(Duration::from_secs(dev.get("uptime").unwrap().as_i64().unwrap() as u64));
                let report_interval = humantime::format_duration(Duration::from_secs(dev.get("report_interval").unwrap().as_i64().unwrap() as u64));
                let sample_interval = humantime::format_duration(Duration::from_secs(dev.get("sample_interval").unwrap().as_i64().unwrap() as u64));
                let bat_cap_str = if let Some(measurements) = latest_device_measurements.as_ref() {
                    format!("{:.0}%", measurements.get(&device_id).unwrap()[0].get("bat_cap").unwrap().as_f64().unwrap())  
                } else { "N/A".to_owned() };

                html! {
                    <div class="border-rounded card">
                        <div class="card-header">
                            <div class="card-item">{"<DeviceName>"}</div>
                            <img src="media/m1s1.webp"/>
                            <hr/>
                        </div>
                        <div class="card-content">
                            if is_online {
                                <div class="card-item">{"Online"}</div><div>{"🟢"}</div>
                                <div class="card-item">{"Uptime"}</div><div>{uptime}</div>
                            } else {
                                <div class="card-item">{"Online"}</div><div>{"🔴"}</div>
                            }
                            <div class="card-item">{"Device ID"}</div><div>{dev.get("device_id").unwrap()}</div>
                            <div class="card-item">{"Firmware"}</div><div>{dev.get("fw_version").unwrap().as_str().unwrap()}</div>
                            <div class="card-item">{"BSEC"}</div><div>{dev.get("bsec_version").unwrap().as_str().unwrap()}</div>
                            <div class="card-item">{"Battery"}</div><div>{bat_cap_str}</div>
                            <div class="card-item">{"WiFi"}</div><div>{dev.get("wifi_ssid").unwrap().as_str().unwrap()}</div>
                            <div class="card-item">{"Report Interval"}</div><div>{report_interval}</div>
                            <div class="card-item">{"Sample Interval"}</div><div>{sample_interval}</div>

                        </div>
                    </div>
                }
            })
            .collect(),
        None => {
            html! {
                <div>{"Cannot get device list"}</div>
            }
        }
    };

    html! { <>{device_list}</> }
}
