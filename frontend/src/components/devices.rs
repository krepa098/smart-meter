use std::time::Duration;
use yew::{function_component, html, use_state};

use crate::{
    req::{self, MeasurementType},
    utils,
};

#[function_component(Devices)]
pub fn device_list() -> yew::Html {
    let devices = use_state(|| None);
    let latest_device_measurements = use_state(|| None);

    // requests
    {
        let devices = devices.clone();

        if devices.is_none() {
            wasm_bindgen_futures::spawn_local(async move {
                devices.set(Some(req::request::device_infos().await));
            });
        }
    }
    {
        let devices = devices.clone();
        let latest_device_measurements = latest_device_measurements.clone();

        if latest_device_measurements.is_none() && devices.is_some() {
            wasm_bindgen_futures::spawn_local(async move {
                let mut measurements = std::collections::HashMap::new();

                for dev in devices.as_ref().unwrap() {
                    let id = dev.device_id;
                    let resp = req::request::measurements(
                        dev.device_id as u32,
                        None,
                        None,
                        req::MeasurementType::All as u32,
                        1,
                    )
                    .await;

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
                let uptime = humantime::format_duration(Duration::from_secs(dev.uptime as u64));
                let report_interval =
                    humantime::format_duration(Duration::from_secs(dev.report_interval as u64));
                let sample_interval =
                    humantime::format_duration(Duration::from_secs(dev.sample_interval as u64));
                let is_online = last_seen < Duration::from_secs(dev.report_interval as u64) + Duration::from_secs(2*60);
                let bat_cap_str = if let Some(measurements) = latest_device_measurements.as_ref() {
                    format!(
                        "{:.0}%",
                        measurements.get(&device_id).unwrap().data[&(MeasurementType::BatCapacity as u32)][0]
                    )
                } else {
                    "N/A".to_owned()
                };
                let wifi = match dev.wifi_ssid.as_ref() {
                    Some(wifi) => wifi.to_string(),
                    None => "N/A".to_string(),
                };

                html! {
                    <div class="col-xs-3">
                        <div class="panel panel-default">
                            <div class="panel-heading"><h4>{"Bedroom"}</h4></div>
                            <div class="panel-body">
                                <img class="center-block" src="media/m1s1.webp"/>
                                <table class="table table-hover">
                                    <tbody>
                                        if is_online {
                                            <tr><td>{"Online"}</td><td>{"🟢"}</td></tr>
                                            <tr><td>{"Uptime"}</td><td>{uptime}</td></tr>
                                        } else {
                                            <tr class="warning"><td>{"Online"}</td><td>{"🔴"}</td></tr>
                                        }
                                        <tr><td>{"Device ID"}</td><td>{dev.device_id}</td></tr>
                                        <tr><td>{"Firmware"}</td><td>{dev.fw_version.to_string()}</td></tr>
                                        <tr><td>{"BSEC"}</td><td>{dev.bsec_version.to_string()}</td></tr>
                                        <tr><td>{"Battery"}</td><td>{bat_cap_str}</td></tr>
                                        <tr><td>{"WiFi"}</td><td>{wifi}</td></tr>
                                        <tr><td>{"Report Interval"}</td><td>{report_interval}</td></tr>
                                        <tr><td>{"Sample Interval"}</td><td>{sample_interval}</td></tr>
                                    </tbody>
                                </table>
                            </div>
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

    html! {
        <div class="row">
            {device_list}
        </div>
    }
}
