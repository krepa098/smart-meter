mod components;
mod db;
mod utils;

use yew::prelude::*;
use yew_router::prelude::*;

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
                <Switch<Route> render={Switch::render(switch)} />
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
    html! {
        <div class="container-fluid">
            <div class="row">
                <Sidebar current_route={Route::Readings}/>
                <div class="col-sm-9 col-sm-offset-3 col-md-10 col-md-offset-2 main">
                    <h1 class="page-header">{"Readings"}</h1>
                    <div class="box-center">
                        <components::chart::Model/>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[function_component(Sidebar)]
pub fn sidebar(props: &Props) -> Html {
    html! {
        <div class="col-sm-3 col-md-2 sidebar">
            <ul class="nav nav-sidebar">
                <li class={if props.current_route == Route::Home { "active" } else {""}}><a href="/">{"⌂ Home"}</a></li>
                <li class={if props.current_route == Route::Devices { "active" } else {""}}><a href="devices">{"🖴 Devices"}</a></li>
                <li class={if props.current_route == Route::Readings { "active" } else {""}}><a href="readings">{"🗠 Readings"}</a></li>
            </ul>
        </div>
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <PageHome/> },
        Route::Devices => html! { <PageDevices/> },
        Route::Readings => html! { <PageReadings/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

fn main() {
    yew::start_app::<Model>();
}
