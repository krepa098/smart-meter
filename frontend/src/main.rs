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
        <div class="main">
            <Sidebar current_route={Route::Devices}/>
            <div class="main-content">
                <components::devices::Devices />
            </div>
        </div>
    }
}

#[function_component(PageHome)]
pub fn page_home() -> Html {
    html! {
        <div class="main">
            <Sidebar current_route={Route::Home}/>
            <div class="main-content">

            </div>
        </div>
    }
}

#[function_component(PageReadings)]
pub fn page_readings() -> Html {
    html! {
        <div class="main">
            <Sidebar current_route={Route::Readings}/>
            <div class="main-content">
                <components::chart::Model/>
            </div>
        </div>
    }
}

#[function_component(Sidebar)]
pub fn sidebar(props: &Props) -> Html {
    html! {
        <div class="side-menu">
            <a class={format!("side-menu-item {}", if props.current_route == Route::Home { "side-menu-item-active" } else {""}  )} href="/">{"⌂ Home"}</a>
            <a class={format!("side-menu-item {}", if props.current_route == Route::Devices { "side-menu-item-active" } else {""}  )} href="devices">{"🖴 Devices"}</a>
            <a class={format!("side-menu-item {}", if props.current_route == Route::Readings { "side-menu-item-active" } else {""}  )} href="readings">{"🗠 Readings"}</a>
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
