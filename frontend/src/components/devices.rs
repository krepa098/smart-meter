use std::time::Duration;

use reqwest::header::ACCEPT;
use yew::{function_component, html, use_state};

use crate::{db, utils};

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
                    .json::<Vec<db::DeviceInfo>>()
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
                    let id = dev.device_id;
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
                        .json::<Vec<db::DeviceMeasurement>>()
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
                let device_id = dev.device_id;
                let last_seen = utils::duration_since_epoch(dev.last_seen as u64);
                let is_online = last_seen.as_secs() < 60 *15;
                let uptime = humantime::format_duration(Duration::from_secs(dev.uptime as u64));
                let report_interval = humantime::format_duration(Duration::from_secs(dev.report_interval as u64));
                let sample_interval = humantime::format_duration(Duration::from_secs(dev.sample_interval as u64));
                let bat_cap_str = if let Some(measurements) = latest_device_measurements.as_ref() {
                    format!("{:.0}%", measurements.get(&device_id).unwrap()[0].bat_cap.unwrap_or(f32::NAN))  
                } else { "N/A".to_owned() };
                let wifi = match  dev.wifi_ssid.as_ref() {
                    Some(wifi) => wifi.to_string(),
                    None => "N/A".to_string(),
                };

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
                            <div class="card-item">{"Device ID"}</div><div>{dev.device_id}</div>
                            <div class="card-item">{"Firmware"}</div><div>{dev.fw_version.to_string()}</div>
                            <div class="card-item">{"BSEC"}</div><div>{dev.bsec_version.to_string()}</div>
                            <div class="card-item">{"Battery"}</div><div>{bat_cap_str}</div>
                            <div class="card-item">{"WiFi"}</div><div>{wifi}</div>
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
