use std::{time::Duration, collections::HashMap};
use log::info;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent};
use yew::{function_component, html, use_state, Callback, use_mut_ref};

use crate::{
    req_utils,
    utils,
};
use common::req::{MeasurementType, MeasurementMask};

const NOT_AVAILABLE: &str = "N/A";

#[function_component(Devices)]
pub fn device_list() -> yew::Html {
    let devices = use_state(|| None);
    let latest_device_measurements = use_state(|| None);
    let device_measurement_infos = use_state(|| None);
    let device_names = use_mut_ref(HashMap::new);
    let device_edit_names = use_state(HashMap::new);

    // requests
    {
        let devices = devices.clone();

        if devices.is_none() {
            wasm_bindgen_futures::spawn_local(async move {
                devices.set(Some(req_utils::request::device_infos().await));
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
                    let resp = req_utils::request::measurements(
                        dev.device_id as u32,
                        None,
                        None,
                        MeasurementMask::ALL,
                        1,
                    )
                    .await;
                    measurements.insert(id, resp);

                    let resp = req_utils::request::measurement_info(dev.device_id as u32).await;
                    measurement_infos.insert(id, resp);

                    let dev_name_resp = req_utils::request::device_name(dev.device_id as u32).await;
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
                    let device_edit_names = device_edit_names.clone();
                    Callback::from(move |_| {
                        let device_names = device_names.clone();
                        let device_edit_names = device_edit_names.clone();
                        let data = device_names.borrow().get(&(device_id as u32)).cloned();
                        let edit_mode: bool = (*device_edit_names).get(&(device_id as u32)).cloned().unwrap_or_default();
                        info!("{:?}", data);
                        if let Some(name) = data {
                            if !edit_mode {
                                let mut m = (*device_edit_names).clone();
                                m.insert(device_id as u32, true);
                                device_edit_names.set(m);
                             } else {
                                wasm_bindgen_futures::spawn_local(async move {
                                        req_utils::request::set_device_name(device_id as u32, name.clone()).await.unwrap();
                                        device_names.borrow_mut().insert(device_id as u32, name);
                                        let mut m = (*device_edit_names).clone();
                                        m.insert(device_id as u32, false);
                                        device_edit_names.set(m);
                                });
                            }
                        }

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


                let device_name = device_names.borrow().get_key_value(&(device_id as u32)).as_ref().map_or("/".to_string(), |s| s.1.clone());
                let device_edit_name: bool = (*device_edit_names).get(&(device_id as u32)).cloned().unwrap_or_default();
 
                html! {
                    <div class="col-lg-4 col-md-6 col-sm-8 col-xs-12">
                        <div class="panel panel-default">
                            <div class="panel-heading">
                                    <div class="row">
                                        <div class="col-lg-12">
                                            <div class="input-group input-group-lg">
                                                <input type="text" class="form-control" 
                                                    value={device_name} onkeyup={name_change_cb} disabled={!device_edit_name}
                                                />
                                                <span class="input-group-btn">
                                                    <button class="btn btn-default" type="button" onclick={button_click_cb}>
                                                    if device_edit_name {
                                                        <span class="glyphicon glyphicon-floppy-save" aria-hidden="true"></span>                                                        
                                                    } else {
                                                        <span class="glyphicon glyphicon-option-vertical" aria-hidden="true"></span>                                                        
                                                    }
                                                    </button>
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
