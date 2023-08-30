mod components;
mod dataset;
mod request;
mod utils;

use chrono::{Days, Local, NaiveDate};
use yew::prelude::*;
use yew_router::prelude::*;

use common::req::{MeasurementMask, MeasurementType};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/devices")]
    Devices,
    #[at("/readings")]
    Readings,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub current_route: Route,

    // measurement mask
    #[prop_or_default]
    pub on_meas_mask_changed: Callback<(MeasurementType, bool)>,

    #[prop_or_default]
    pub meas_mask: MeasurementMask,

    // dates
    #[prop_or_default]
    pub on_from_date_changed: Callback<NaiveDate>,
    #[prop_or_default]
    pub on_to_date_changed: Callback<NaiveDate>,
    #[prop_or_default]
    pub from_date: NaiveDate,
    #[prop_or_default]
    pub to_date: NaiveDate,

    // device
    #[prop_or_default]
    pub on_device_id_changed: Callback<u32>,
    #[prop_or_default]
    pub device_id: Option<u32>,
}

enum Msg {}

struct Model {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        }
    }
}

#[function_component(PageDevices)]
pub fn page_devices() -> Html {
    html! {
        <div class="container-fluid">
            <div class="row">
                <Sidebar current_route={Route::Devices}/>
                <div class="main-content">
                    <h1 class="page-header">{"Devices"}</h1>
                    <components::devices::Devices />
                </div>
            </div>
        </div>
    }
}

#[function_component(PageHome)]
pub fn page_home() -> Html {
    html! {
        <div class="container-fluid">
            <div class="row">
                <Sidebar current_route={Route::Home}/>
                <div class="main-content">
                    <h1 class="page-header">{"Home"}</h1>
                    <components::summary::Summary />
                </div>
            </div>
        </div>
    }
}

#[function_component(PageReadings)]
pub fn page_readings() -> Html {
    // measurement mask
    let meas_mask_handle = use_state_eq(MeasurementMask::default);
    let on_meas_mask_changed: Callback<(MeasurementType, bool)> = {
        let handle = meas_mask_handle.clone();
        Callback::from(move |(meas_type, active)| {
            let mut ret = *handle;
            ret.set(meas_type, active);
            handle.set(ret);
        })
    };

    // datetime
    let to_date_handle = use_state_eq(|| Local::now().date_naive());
    let from_date_handle = use_state_eq(|| Local::now().date_naive() - Days::new(1));

    let on_from_date_changed: Callback<NaiveDate> = {
        let handle = from_date_handle.clone();
        Callback::from(move |date| {
            handle.set(date);
        })
    };

    let on_to_date_changed: Callback<NaiveDate> = {
        let handle = to_date_handle.clone();
        Callback::from(move |date| {
            handle.set(date);
        })
    };

    let device_id_handle = use_state_eq(|| None);
    let on_device_id_changed: Callback<u32> = {
        let handle = device_id_handle.clone();
        Callback::from(move |id| {
            handle.set(Some(id));
        })
    };

    html! {
        <div class="container-fluid">
            <div class="row">
                <Sidebar current_route={Route::Readings}
                    {on_meas_mask_changed} meas_mask={*meas_mask_handle}
                    {on_to_date_changed} {on_from_date_changed}
                    to_date={*to_date_handle} from_date={*from_date_handle}
                    {on_device_id_changed} device_id={*device_id_handle}
                />
                <div class="main-content">
                    <h1 class="page-header">{"Readings"}</h1>
                    <div class="box-center">
                        <components::chart::Model
                            device_id={*device_id_handle}
                            measurement_mask={*meas_mask_handle}
                            to_date={*to_date_handle} from_date={*from_date_handle}
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[function_component(Sidebar)]
pub fn sidebar(props: &Props) -> Html {
    let cr = &props.current_route;
    let class_active = move |r| {
        if *cr == r {
            "active"
        } else {
            ""
        }
    };

    html! {
        <div class="sidebar">
            <ul class="nav nav-sidebar">
                <img class="logo-img center-block" src="media/logo.webp"/>
                <li class={class_active(Route::Home)}>
                    <Link<Route> to={Route::Home}><span class="glyphicon glyphicon-home" aria-hidden="true"/>{" Home"}</Link<Route>>
                </li>
                <li class={class_active(Route::Devices)}>
                    <Link<Route> to={Route::Devices}><span class="glyphicon glyphicon-modal-window" aria-hidden="true"/> {" Devices"}</Link<Route>>
                </li>
                <li class={class_active(Route::Readings)}>
                    <Link<Route> to={Route::Readings}><span class="glyphicon glyphicon-scale" aria-hidden="true"/> {" Readings"}</Link<Route>>
                </li>
                <components::chart_menu::Model visible={props.current_route==Route::Readings}
                    on_meas_mask_changed={props.on_meas_mask_changed.clone()}
                    meas_mask={props.meas_mask}
                    on_to_date_changed={props.on_to_date_changed.clone()}
                    on_from_date_changed={props.on_from_date_changed.clone()}
                    to_date={props.to_date}
                    from_date={props.from_date}
                    on_device_id_changed={props.on_device_id_changed.clone()}
                    device_id={props.device_id}
                />
                <li/>
            </ul>

            <ul class="nav nav-sidebar fix-bottom">
            {format!("v{}.{}.{}", env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0), env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0), env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0))}
            </ul>

        </div>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <PageHome/> },
        Route::Devices => html! { <PageDevices/> },
        Route::Readings => html! { <PageReadings/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Model>::new().render();
}
