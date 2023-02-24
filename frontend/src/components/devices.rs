use std::{time::Duration, collections::HashMap};
use log::info;
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};
use yew::{function_component, html, use_state, Callback, use_mut_ref};

use crate::{
    req::{self, MeasurementType},
    utils,
};

const NOT_AVAILABLE: &str = "N/A";

#[function_component(Devices)]
pub fn device_list() -> yew::Html {
    let devices = use_state(|| None);
    let latest_device_measurements = use_state(|| None);
    let device_measurement_infos = use_state(|| None);
    let device_names = use_mut_ref(HashMap::new);

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
        let device_measurement_infos = device_measurement_infos.clone();
        let device_names = device_names.clone();

        if latest_device_measurements.is_none() && devices.is_some() {
            wasm_bindgen_futures::spawn_local(async move {
                let mut measurements = HashMap::new();
                let mut measurement_infos = HashMap::new();
                let mut names = HashMap::new();

                for dev in devices.as_ref().unwrap() {
                    let id = dev.device_id;
                    let resp = req::request::measurements(
                        dev.device_id as u32,
                        None,
                        None,
                        req::MeasurementMask::ALL,
                        1,
                    )
                    .await;
                    measurements.insert(id, resp);

                    let resp = req::request::measurement_info(dev.device_id as u32).await;
                    measurement_infos.insert(id, resp);

                    let dev_name_resp = req::request::device_name(dev.device_id as u32).await;
                    let dev_name = dev_name_resp.map_or(format!("{} (unnamed)", dev.device_id as u32), |v| v);
                    names.insert(dev.device_id as u32, dev_name);
                }

                latest_device_measurements.set(Some(measurements));
                device_measurement_infos.set(Some(measurement_infos));
                device_names.replace(names);
            });
        }
    }

    let device_list = match devices.as_ref() {
        Some(devices) => devices
            .iter()
            .map(|dev| {
                let device_id = dev.device_id;
                let last_seen = utils::utc_from_millis(dev.last_seen * 1000);
                let uptime = humantime::format_duration(Duration::from_secs(dev.uptime as u64));
                let report_interval =
                    humantime::format_duration(Duration::from_secs(dev.report_interval as u64));
                let sample_interval =
                    humantime::format_duration(Duration::from_secs(dev.sample_interval as u64));

                let duration_since_last_report = utils::utc_now() - last_seen;
                let is_online = duration_since_last_report < chrono::Duration::from_std(Duration::from_secs(dev.report_interval as u64) - Duration::from_secs(60)).unwrap();
                let bat_cap_str = latest_device_measurements.as_ref().map_or(
                    NOT_AVAILABLE.to_string(),
                    |m| format!(
                        "{:.0}%",
                        m.get(&device_id).unwrap().data[&(MeasurementType::BatCapacity as u32)][0]
                    )
                );
                let wifi = dev.wifi_ssid.as_ref().map_or(
                    NOT_AVAILABLE.to_string(), 
                    |w| w.to_string(),
                );
                let sample_count = device_measurement_infos.as_ref().map_or(
                    NOT_AVAILABLE.to_string(), 
                    |info| format!("{}", info.get(&device_id).unwrap().count)
                );

                let button_click_cb = {
                    let device_names = device_names.clone();
                    Callback::from(move |_| {
                        let device_names = device_names.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let name = device_names.borrow().get(&(device_id as u32)).cloned();
                            if let Some(name) = name {
                                req::request::set_device_name(device_id as u32, name).await.unwrap();
                            }
                        });
                    })
                };

                let name_change_cb = {
                    let device_names = device_names.clone();
                    Callback::from(move |e: KeyboardEvent| {
                        let target: Option<web_sys::EventTarget> = e.target();
                        let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                        let value = input.unwrap().value();
                        device_names.borrow_mut().insert(device_id as u32, value);                 
                    })
                };
  
                html! {
                    <div class="col-xs-3">
                        <div class="panel panel-default">
                            <div class="panel-heading">
                                <div class="row">
                                    <div class="col-lg-12">
                                        <div class="input-group input-group-lg">
                                            <input type="text" class="form-control" 
                                                value={device_names.borrow().get_key_value(&(device_id as u32)).as_ref().map_or("/".to_string(), |s| s.1.clone())}
                                                    onkeyup={name_change_cb}
                                                />
                                            <span class="input-group-btn">
                                                <button class="btn btn-default" type="button" onclick={button_click_cb}>{"✎"}</button>
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="panel-body">
                                <img class="device-img center-block" src="media/m1s1.webp"/>
                                <table class="table table-hover">
                                    <tbody>
                                        if is_online {
                                            <tr><td>{"Online"}</td><td>{"🟢"}</td></tr>
                                            <tr><td>{"Uptime"}</td><td>{uptime}</td></tr>
                                        } else {
                                            <tr class="warning"><td>{"Online"}</td><td>{"🔴"}</td></tr>
                                        }
                                        <tr><td>{"Device ID"}</td><td>{dev.device_id}</td></tr>
                                        <tr><td>{"Firmware"}</td><td>{dev.fw_version.clone()}</td></tr>
                                        <tr><td>{"BSEC"}</td><td>{dev.bsec_version.clone()}</td></tr>
                                        <tr><td>{"Battery"}</td><td>{bat_cap_str}</td></tr>
                                        <tr><td>{"WiFi"}</td><td>{wifi}</td></tr>
                                        <tr><td>{"Report Interval"}</td><td>{report_interval}</td></tr>
                                        <tr><td>{"Sample Interval"}</td><td>{sample_interval}</td></tr>
                                        <tr><td>{"Samples"}</td><td>{sample_count}</td></tr>
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
