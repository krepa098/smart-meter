use reqwest::header::ACCEPT;
use yew::{function_component, html, use_state};

#[function_component(Devices)]
pub fn device_list() -> Html {
    let devices = use_state(|| None);
    let client = reqwest::Client::new();

    // requests
    {
        let devices = devices.clone();

        if devices.is_none() {
            wasm_bindgen_futures::spawn_local(async move {
                let resp = client
                    .get("http://127.0.0.1:8081/api/devices")
                    //.fetch_mode_no_cors()
                    //.header(ACCESS_CONTROL_ALLOW_ORIGIN, "http://127.0.0.1")
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

    let device_list = match devices.as_ref() {
        Some(devices) => devices
            .iter()
            .map(|dev| {
                html! {
                    <>
                    <div>{format!("Device ID: {}", dev.get("device_id").unwrap())}</div>
                    <div>{format!("Firmware: {}", dev.get("fw_version").unwrap())}</div>
                    <div>{format!("BSEC version: {}", dev.get("bsec_version").unwrap())}</div>
                    <div>{format!("Uptime: {}", humantime::format_duration(std::time::Duration::from_millis(dev.get("uptime").unwrap().as_i64().unwrap() as u64)))}</div>
                    </>
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
