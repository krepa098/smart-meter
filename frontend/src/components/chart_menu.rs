use crate::{
    req::{MeasurementMask, MeasurementType},
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

pub struct Model {}

impl Component for Model {
    type Message = ();

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
        let ts_from_cb = ctx.link().callback(move |e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let ts_utc: DateTime<Utc> = utils::js_date_ts_to_utc(&timestring);
            cb.emit(ts_utc);
        });

        let cb = ctx.props().on_to_date_changed.clone();
        let ts_to_cb = ctx.link().callback(move |e: Event| {
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
                <li>
                    <div class="submenuitem">
                            <div class="input-group col-md-12">
                                <span class="input-group-addon width-70" id="basic-addon3">{"From"}</span>
                                <input type="date" class="form-control" onchange={ts_from_cb} value={ts_from}/>
                            </div>
                    </div>
                </li>
                <li>
                    <div class="submenuitem">
                            <div class="input-group col-md-12">
                                <span class="input-group-addon width-70" id="basic-addon3">{"To"}</span>
                                <input type="date" class="form-control" onchange={ts_to_cb} value={ts_to}/>
                            </div>
                    </div>
                </li>
                {checkbox_list}
             </ul>
            }
        }
    }
}
