use crate::{db, utils};
use chrono::{prelude::*, Duration};
use gloo_console::log;
use reqwest::header::ACCEPT;
use std::rc::Rc;
use utils::Stats;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_chart::{
    axis::{Axis, Orientation, Scale},
    linear_axis_scale::LinearScale,
    series::{self, Series, Tooltipper, Type},
    time_axis_scale::TimeScale,
};

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 400.0;
const MARGIN: f32 = 50.0;
const TICK_LENGTH: f32 = 15.0;

pub enum Msg {
    StartDateChanged(DateTime<Utc>),
    EndDateChanged(DateTime<Utc>),
    DeviceIDChanged(u32),
    DatapointsReceived(Vec<(i64, f32)>),
}

pub struct Model {
    datapoints: Vec<(i64, f32)>,
    ts_from: DateTime<Utc>,
    ts_to: DateTime<Utc>,
    device_id: u32,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let ts_to = Utc::now();
        let ts_from = Utc::now() - Duration::days(1);

        Self {
            datapoints: vec![],
            ts_from,
            ts_to,
            device_id: 396891554,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartDateChanged(ts) => {
                self.ts_from = ts;
                self.request_datapoints(ctx);
                false
            }
            Msg::EndDateChanged(ts) => {
                self.ts_to = ts;
                self.request_datapoints(ctx);
                false
            }
            Msg::DeviceIDChanged(id) => {
                self.device_id = id;
                self.request_datapoints(ctx);
                false
            }
            Msg::DatapointsReceived(dp) => {
                self.datapoints = dp;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let local_ts_from: DateTime<Local> = DateTime::from(self.ts_from);
        let local_ts_to: DateTime<Local> = DateTime::from(self.ts_to);
        let ts_from = local_ts_from.format("%Y-%m-%dT%H:%M").to_string();
        let ts_to = local_ts_to.format("%Y-%m-%dT%H:%M").to_string();

        // input callback
        let ts_from_cb = ctx.link().callback(|e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let ts_utc: DateTime<Utc> = utils::js_ts_to_utc(&timestring);

            log!("from (utc)", ts_utc.to_string());

            Msg::StartDateChanged(ts_utc)
        });

        // input callback
        let ts_to_cb = ctx.link().callback(|e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let ts_utc: DateTime<Utc> = utils::js_ts_to_utc(&timestring);

            log!("to (utc)", ts_utc.to_string());

            Msg::EndDateChanged(ts_utc)
        });

        // // stats
        // let stats = self.datapoints.stats();

        // // assembly measurements
        // let datapoints: Vec<_> = self
        //     .datapoints
        //     .iter()
        //     .map(|(x, y)| (*x, *y, None))
        //     .collect();
        // let datapoints = Rc::new(datapoints);

        // // axis setup
        // let end_date = self.ts_to;
        // let start_date = self.ts_from;
        // let timespan = start_date..end_date;
        // let h_scale =
        //     Rc::new(TimeScale::new(timespan, Duration::minutes(60))) as Rc<dyn Scale<Scalar = _>>;
        // let v_scale =
        //     Rc::new(LinearScale::new(stats.y_min..stats.y_max, 1.0)) as Rc<dyn Scale<Scalar = _>>;
        // let tooltip = Rc::from(series::y_tooltip()) as Rc<dyn Tooltipper<_, _>>;

        // html
        html! {
            <div class="panel panel-default">
                <div class="panel-heading">
                    <h3 class="panel-title">{"Temperature"}</h3>
                </div>
                <div class="panel-body">
                    <div class="row">
                        <div class="col-md-3">
                            <div class="input-group">
                                <span class="input-group-addon" id="basic-addon3">{"From"}</span>
                                <input type="datetime-local" onchange={ts_from_cb} class="form-control" value={ts_from}/>
                            </div>
                        </div>
                        <div class="col-md-3">
                            <div class="input-group">
                                <span class="input-group-addon" id="basic-addon3">{"To"}</span>
                                <input type="datetime-local" onchange={ts_to_cb} class="form-control" value={ts_to}/>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-md-8">
                            <SimpleChart ylabel={"Temperature °C"} datapoints={self.datapoints.clone()}/>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.request_datapoints(ctx);
        }
    }
}

impl Model {
    pub fn request_datapoints(&self, ctx: &Context<Self>) {
        let ts_from = self.ts_from;
        let ts_to = self.ts_to;

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let client = reqwest::Client::new();
            let from_date = ts_from;
            let limit = 200000;

            let device_id = 396891554;
            let resp = client
                .get("http://127.0.0.1:8081/api/measurements/by_date")
                .query(&[
                    ("device_id", device_id),
                    ("from_date", from_date.timestamp_millis()),
                    ("limit", limit),
                ])
                .header(ACCEPT, "application/json")
                .send()
                .await
                .unwrap()
                .json::<Vec<db::DeviceMeasurement>>()
                .await
                .unwrap();

            let datapoints: Vec<_> = resp
                .iter()
                .map(|m| (m.timestamp, m.temperature.unwrap_or(f32::NAN)))
                .collect();

            link.send_message(Msg::DatapointsReceived(datapoints));
        });
    }
}

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    ylabel: String,
    datapoints: Vec<(i64, f32)>,
}

#[function_component(SimpleChart)]
fn simple_chart(props: &ChartProps) -> Html {
    if !props.datapoints.is_empty() {
        // stats
        let stats = props.datapoints.stats();

        // assembly measurements
        let datapoints: Vec<_> = props
            .datapoints
            .iter()
            .map(|(x, y)| (*x, *y, None))
            .collect();
        let datapoints = Rc::new(datapoints);

        // axis setup
        let start_date = utils::utc_from_millis(stats.x_min);
        let end_date = utils::utc_from_millis(stats.x_max);
        let duration = end_date - start_date;

        let timespan = start_date..end_date;
        let h_scale = Rc::new(TimeScale::new(timespan, duration / 5)) as Rc<dyn Scale<Scalar = _>>;
        let v_scale =
            Rc::new(LinearScale::new(stats.y_min..stats.y_max, 1.0)) as Rc<dyn Scale<Scalar = _>>;
        let tooltip = Rc::from(series::y_tooltip()) as Rc<dyn Tooltipper<_, _>>;

        html! {
            <div>
                <svg class="chart" viewBox={format!("0 0 {} {}", WIDTH, HEIGHT)} preserveAspectRatio="none">
                    <Series<i64, f32>
                        series_type={Type::Line}
                        name="some-series"
                        data={&datapoints}
                        horizontal_scale={Rc::clone(&h_scale)}
                        horizontal_scale_step={Duration::days(1).num_milliseconds()}
                        tooltipper={Rc::clone(&tooltip)}
                        vertical_scale={Rc::clone(&v_scale)}
                        x={MARGIN} y={MARGIN} width={WIDTH - (MARGIN * 2.0)} height={HEIGHT - (MARGIN * 2.0)} />

                    <Axis<f32>
                        name={props.ylabel.clone()}
                        orientation={Orientation::Left}
                        scale={Rc::clone(&v_scale)}
                        x1={MARGIN} y1={MARGIN} xy2={HEIGHT - MARGIN}
                        tick_len={TICK_LENGTH}
                        title={props.ylabel.clone()} />

                    <Axis<i64>
                        name="Time"
                        orientation={Orientation::Bottom}
                        scale={Rc::clone(&h_scale)}
                        x1={MARGIN} y1={HEIGHT - MARGIN} xy2={WIDTH - MARGIN}
                        tick_len={TICK_LENGTH}
                        title={"Time".to_string()} />
                </svg>
            </div>
        }
    } else {
        html! {
            <div class="chart">
                <label>{"no data"}</label>
            </div>
        }
    }
}
