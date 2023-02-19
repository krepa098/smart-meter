mod components;
mod req;
mod utils;

use log::info;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::req::{MeasurementMask, MeasurementType};

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

    // series
    #[prop_or_default]
    pub on_mes_type_changed: Callback<(MeasurementType, bool)>,
}

enum Msg {}

struct Model {
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
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
                <div class="col-sm-9 col-sm-offset-3 col-md-10 col-md-offset-2 main">
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
                <div class="col-sm-9 col-sm-offset-3 col-md-10 col-md-offset-2 main">
                    <h1 class="page-header">{"Home"}</h1>
                </div>
            </div>
        </div>
    }
}

#[function_component(PageReadings)]
pub fn page_readings() -> Html {
    let mes_type_handle = use_state_eq(MeasurementMask::default);

    let on_mes_type_changed: Callback<(MeasurementType, bool)> = {
        let handle = mes_type_handle.clone();
        Callback::from(move |(mes_type, active)| {
            let mut ret = *handle;
            ret.set(mes_type, active);
            handle.set(ret);
            info!("Mes type changed {:?}", ret);
        })
    };

    html! {
        <div class="container-fluid">
            <div class="row">
                <Sidebar current_route={Route::Readings} {on_mes_type_changed}/>
                <div class="col-sm-9 col-sm-offset-3 col-md-10 col-md-offset-2 main">
                    <h1 class="page-header">{"Readings"}</h1>
                    <div class="box-center">
                        <components::chart::Model measurement_mask={*mes_type_handle}/>
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
        <div class="col-sm-3 col-md-2 sidebar">
            <ul class="nav nav-sidebar">
                <li class={class_active(Route::Home)}>
                    <Link<Route> to={Route::Home}>{"⌂ Home"}</Link<Route>>
                </li>
                <li class={class_active(Route::Devices)}>
                    <Link<Route> to={Route::Devices}>{"🖴 Devices"}</Link<Route>>
                </li>
                <li class={class_active(Route::Readings)}>
                    <Link<Route> to={Route::Readings}>{"🗠 Readings"}</Link<Route>>
                    <components::chart_menu::Model visible={props.current_route==Route::Readings} on_mes_type_changed={props.on_mes_type_changed.clone()}/>
                </li>


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
