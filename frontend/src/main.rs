mod components;
mod utils;

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use reqwest::header::ACCEPT;
use yew::prelude::*;

enum Msg {
    AddOne,
    Req,
}

struct Model {
    value: i64,
    client: std::sync::Arc<reqwest::Client>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let client = reqwest::Client::new();

        Self {
            value: 0,
            client: std::sync::Arc::new(client),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
            Msg::Req => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <button onclick={link.callback(|_| Msg::Req)}>{ "+1" }</button>
                <p>{ self.value }</p>
                <components::devices::Devices />
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
