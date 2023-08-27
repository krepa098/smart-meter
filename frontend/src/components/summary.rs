use std::{time::Duration, collections::HashMap};
use yew::{function_component, html, use_state,  use_mut_ref};

use crate::{
    request,
    utils,
};
use common::req::{MeasurementType, MeasurementMask};

const NOT_AVAILABLE: &str = "N/A";

#[function_component(Summary)]
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
                devices.set(Some(request::device_infos().await));
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
                    let resp = request::measurements(
                        dev.device_id as u32,
                        None,
                        None,
                        MeasurementMask::ALL,
                        1,
                    )
                    .await.unwrap();
                    measurements.insert(id, resp);

                    let resp = request::measurement_info(dev.device_id as u32).await;
                    measurement_infos.insert(id, resp);

                    let dev_name_resp = request::device_name(dev.device_id as u32).await;
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

                let duration_since_last_report = utils::utc_now() - last_seen;
                let is_online = duration_since_last_report < chrono::Duration::from_std(Duration::from_secs(dev.report_interval as u64) - Duration::from_secs(60)).unwrap();
                let bat_cap_str = latest_device_measurements.as_ref().map_or(
                    NOT_AVAILABLE.to_string(),
                    |m| format!(
                        "{:.0}%",
                        m.get(&device_id).unwrap().data[&(MeasurementType::BatCapacity as u32)][0].unwrap_or(std::f32::NAN)
                    )
                );
                let temperature_str = latest_device_measurements.as_ref().map_or(
                    NOT_AVAILABLE.to_string(),
                    |m| format!(
                        "{:.1}°C",
                        m.get(&device_id).unwrap().data[&(MeasurementType::Temperature as u32)][0].unwrap_or(std::f32::NAN)
                    )
                );
                let humidity_str = latest_device_measurements.as_ref().map_or(
                    NOT_AVAILABLE.to_string(),
                    |m| format!(
                        "{:.1}%",
                        m.get(&device_id).unwrap().data[&(MeasurementType::Humidity as u32)][0].unwrap_or(std::f32::NAN)
                    )
                );
                let pressure_str = latest_device_measurements.as_ref().map_or(
                    NOT_AVAILABLE.to_string(),
                    |m| format!(
                        "{:.0}hPa",
                        m.get(&device_id).unwrap().data[&(MeasurementType::Pressure as u32)][0].unwrap_or(std::f32::NAN)/100.0
                    )
                );

                let device_name = device_names.borrow().get_key_value(&(device_id as u32)).as_ref().map_or("/".to_string(), |s| s.1.clone());
 
                html! {
                    <div class="col-lg-4 col-md-6 col-sm-8 col-xs-12">
                        <div class="panel panel-default">
                            <div class="panel-heading">
                                    <div class="row">
                                        <div class="col-lg-12">
                                            <h3>{device_name}</h3>
                                        </div>
                                    </div>
                             </div>
                            <div class="panel-body">
                                <table class="table table-hover">
                                    <tbody>
                                        if is_online {
                                            <tr><td>{"Online"}</td><td>{"🟢"}</td></tr>
                                        } else {
                                            <tr class="warning"><td>{"Online"}</td><td>{"🔴"}</td></tr>
                                        }
                                        <tr><td>{"Temperature"}</td><td>{temperature_str}</td></tr>
                                        <tr><td>{"Humidity"}</td><td>{humidity_str}</td></tr>
                                        <tr><td>{"Pressure"}</td><td>{pressure_str}</td></tr>
                                        <tr><td>{"Battery"}</td><td>{bat_cap_str}</td></tr>
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
