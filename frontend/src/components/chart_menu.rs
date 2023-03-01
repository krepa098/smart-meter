use std::collections::HashMap;

use crate::{
    req::{self, DeviceInfo, MeasurementInfo, MeasurementMask, MeasurementType},
    utils,
};
use chrono::NaiveDate;
use log::info;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Settings {}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub visible: bool,

    // measurement mask
    pub on_meas_mask_changed: Callback<(MeasurementType, bool)>,
    pub meas_mask: MeasurementMask,

    // dates
    pub on_from_date_changed: Callback<NaiveDate>,
    pub on_to_date_changed: Callback<NaiveDate>,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,

    // device
    pub on_device_id_changed: Callback<u32>,
    pub device_id: Option<u32>,
}

pub enum Msg {
    MeasurementInfoReceived(MeasurementInfo),
    DeviceInfoReceived(Vec<DeviceInfo>),
    DeviceNameReceived((u32, String)),
}

pub struct Model {
    pub ts_max: Option<NaiveDate>,
    pub ts_min: Option<NaiveDate>,
    pub devices: Option<Vec<DeviceInfo>>,
    pub device_names: HashMap<u32, String>,
}

impl Component for Model {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            ts_max: None,
            ts_min: None,
            devices: None,
            device_names: HashMap::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        // create measurement mask checkboxes
        let mes_types = [
            (" Temperature", MeasurementType::Temperature),
            (" Humidity", MeasurementType::Humidity),
            (" Pressure", MeasurementType::Pressure),
            (" Air Quality", MeasurementType::AirQuality),
            (" Battery Voltage", MeasurementType::BatVoltage),
        ];

        let meas_mask = ctx.props().meas_mask;

        let checkbox_list: Vec<_> = mes_types
            .iter()
            .map(|(desc, ty)| {
                let cb = ctx.props().on_meas_mask_changed.clone();
                let mes_type = *ty;
                let cbe = Callback::from(move |e: Event| {
                    let target: EventTarget = e.target().unwrap();
                    cb.emit((
                        mes_type,
                        target.unchecked_into::<HtmlInputElement>().checked(),
                    ));
                });

                html! {
                    <li>
                        <div class="submenuitem">
                            <input type="checkbox" onchange={cbe} id={desc.to_string()} checked={meas_mask.is_set(*ty)}/><span><label for={desc.to_string()} class="submenulabel"><a>{desc}</a></label></span>
                        </div>
                    </li>
                }
            })
            .collect();

        // date callbacks
        let cb = ctx.props().on_from_date_changed.clone();
        let ts_from_cb = Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let date = utils::js_date_ts_to_naive(&timestring);
            cb.emit(date);
        });

        let cb = ctx.props().on_to_date_changed.clone();
        let ts_to_cb = Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let date = utils::js_date_ts_to_naive(&timestring);
            cb.emit(date);
        });

        let cb = ctx.props().on_device_id_changed.clone();
        let device_cb = Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let value = input.unwrap().value_as_number() as u32;
            info!("{}", value);
            cb.emit(value);
        });

        let ts_from = ctx.props().from_date.format("%Y-%m-%d").to_string();
        let ts_to = ctx.props().to_date.format("%Y-%m-%d").to_string();

        let device_ids = self.device_names.keys();
        let device_list: Vec<_> = device_ids
            .into_iter()
            .map(|k| {
                let device_id = *k;
                let device_name = self.device_names[&device_id].clone();
                html! {
                    <option>{device_name}</option>
                }
            })
            .collect();

        // menu
        html! {
            if  ctx.props().visible {
            <ul class="nav nav-sidebar">
                // device
                <li>
                    <div class="submenuitem">
                        <div class="input-group col-md-12">
                            <span class="input-group-addon width-70" id="basic-addon3">{"Device"}</span>
                            <select id="company" class="form-control" onchange={device_cb}>
                                {device_list}
                            </select>
                        </div>
                    </div>
                </li>
                // date and time
                <li>
                    <div class="submenuitem">
                        <div class="input-group col-md-12">
                            <span class="input-group-addon width-70" id="basic-addon3">{"From"}</span>
                            <input type="date" class="form-control" onchange={ts_from_cb} value={ts_from}
                                min={self.ts_min.as_ref().map(utils::naive_date_to_js)}
                                max={self.ts_max.as_ref().map(utils::naive_date_to_js)}
                            />
                        </div>
                    </div>
                </li>
                <li>
                    <div class="submenuitem">
                        <div class="input-group col-md-12">
                            <span class="input-group-addon width-70" id="basic-addon3">{"To"}</span>
                            <input type="date" class="form-control" onchange={ts_to_cb} value={ts_to}
                                min={self.ts_min.as_ref().map(utils::naive_date_to_js)}
                                max={self.ts_max.as_ref().map(utils::naive_date_to_js)}
                            />
                        </div>
                    </div>
                </li>
                // checkboxes
                {checkbox_list}
             </ul>
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MeasurementInfoReceived(m) => {
                self.ts_max = Some(utils::utc_from_millis(m.to_timestamp).date_naive());
                self.ts_min = Some(utils::utc_from_millis(m.from_timestamp).date_naive());
                true
            }
            Msg::DeviceInfoReceived(m) => {
                if ctx.props().device_id.is_none() && !m.is_empty() {
                    ctx.props()
                        .on_device_id_changed
                        .emit(m.first().unwrap().device_id as u32);
                }

                self.devices = Some(m);

                true
            }
            Msg::DeviceNameReceived((id, mut name)) => {
                if name.is_empty() {
                    name = id.to_string();
                }
                self.device_names.insert(id, name);
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
            let device_id = ctx.props().device_id;
            let on_device_changed = ctx.props().on_device_id_changed.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let devices_resp = req::request::device_infos().await;

                // resolve device names
                for dev in &devices_resp {
                    let name_req = req::request::device_name(dev.device_id as u32).await;
                    match name_req {
                        Ok(name) => {
                            link.send_message(Msg::DeviceNameReceived((dev.device_id as u32, name)))
                        }
                        Err(_) => link.send_message(Msg::DeviceNameReceived((
                            dev.device_id as u32,
                            format!("{} (unnamed)", dev.device_id),
                        ))),
                    }
                }

                // request measurement info for selected device
                let device_id = device_id.unwrap_or(devices_resp.first().unwrap().device_id as u32);
                let resp = req::request::measurement_info(device_id).await;
                link.send_message(Msg::MeasurementInfoReceived(resp));
                on_device_changed.emit(device_id);

                // devices
                link.send_message(Msg::DeviceInfoReceived(devices_resp));
            });
        }
    }
}
