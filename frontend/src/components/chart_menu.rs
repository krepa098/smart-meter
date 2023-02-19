use crate::req::{MeasurementMask, MeasurementType};
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

    pub on_mes_mask_changed: Callback<(MeasurementType, bool)>,
    pub meas_mask: MeasurementMask,
}

pub struct Model {}

impl Component for Model {
    type Message = ();

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                let cb = ctx.props().on_mes_mask_changed.clone();
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

        html! {
            if  ctx.props().visible {
            <ul class="nav nav-sidebar">
                {checkbox_list}
             </ul>
            }
        }
    }
}
