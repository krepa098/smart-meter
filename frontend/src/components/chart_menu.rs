use crate::{
    req::{self, MeasurementInfo, MeasurementMask, MeasurementType},
    utils,
};
use chrono::{DateTime, Local, Utc};
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
    pub on_from_date_changed: Callback<DateTime<Utc>>,
    pub on_to_date_changed: Callback<DateTime<Utc>>,
    pub from_date: DateTime<Utc>,
    pub to_date: DateTime<Utc>,
}

pub enum Msg {
    MeasurementInfoReceived(MeasurementInfo),
}

pub struct Model {
    pub device_id: u32,
    pub ts_max: Option<DateTime<Utc>>,
    pub ts_min: Option<DateTime<Utc>>,
}

impl Component for Model {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            device_id: 396891554,
            ts_max: None,
            ts_min: None,
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
            let ts_utc: DateTime<Utc> = utils::js_date_ts_to_utc(&timestring);
            cb.emit(ts_utc);
        });

        let cb = ctx.props().on_to_date_changed.clone();
        let ts_to_cb = Callback::from(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let ts_utc: DateTime<Utc> = utils::js_date_ts_to_utc(&timestring);
            cb.emit(ts_utc);
        });

        let local_ts_from: DateTime<Local> = DateTime::from(ctx.props().from_date);
        let local_ts_to: DateTime<Local> = DateTime::from(ctx.props().to_date);
        let ts_from = local_ts_from.format("%Y-%m-%d").to_string();
        let ts_to = local_ts_to.format("%Y-%m-%d").to_string();

        // menu
        html! {
            if  ctx.props().visible {
            <ul class="nav nav-sidebar">
                // date and time
                <li>
                    <div class="submenuitem">
                        <div class="input-group col-md-12">
                            <span class="input-group-addon width-70" id="basic-addon3">{"From"}</span>
                            <input type="date" class="form-control" onchange={ts_from_cb} value={ts_from}
                                min={match self.ts_min {
                                    Some(ts) => utils::utc_to_js(&ts),
                                    None => "".to_string(),
                                }}
                                max={match self.ts_max {
                                    Some(ts) => utils::utc_to_js(&ts),
                                    None => "".to_string(),
                                }}
                            />
                        </div>
                    </div>
                </li>
                <li>
                    <div class="submenuitem">
                        <div class="input-group col-md-12">
                            <span class="input-group-addon width-70" id="basic-addon3">{"To"}</span>
                            <input type="date" class="form-control" onchange={ts_to_cb} value={ts_to}
                                min={match self.ts_min {
                                    Some(ts) => utils::utc_to_js(&ts),
                                    None => "".to_string(),
                                }}
                                max={match self.ts_max {
                                    Some(ts) => utils::utc_to_js(&ts),
                                    None => "".to_string(),
                                }}
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
                self.ts_max = Some(utils::utc_from_millis(m.to_timestamp));
                self.ts_min = Some(utils::utc_from_millis(m.from_timestamp));
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
            let device_id = self.device_id;
            wasm_bindgen_futures::spawn_local(async move {
                let resp = req::request::measurement_info(device_id).await;
                link.send_message(Msg::MeasurementInfoReceived(resp));
            });
        }
    }
}
