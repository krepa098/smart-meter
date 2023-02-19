use crate::{
    req::{self, MeasurementMask, MeasurementRequestResponse, MeasurementType},
    utils,
};
use chrono::{prelude::*, Duration, DurationRound};
use log::info;
use std::rc::Rc;
use utils::Stats;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_chart::{
    axis::{Axis, Orientation, Scale},
    linear_axis_scale::LinearScale,
    series::{self, Series, Tooltipper, Type},
    time_axis_scale::{Labeller, TimeScale},
};

const WIDTH: f32 = 900.0;
const HEIGHT: f32 = 320.0;
const MARGIN: f32 = 50.0;
const TICK_LENGTH: f32 = 15.0;

pub enum Msg {
    StartDateChanged(DateTime<Utc>),
    EndDateChanged(DateTime<Utc>),
    DeviceIDChanged(u32),
    MeasurementsReceived(MeasurementRequestResponse),
}

pub struct Model {
    measurements: Option<MeasurementRequestResponse>,
    ts_from: DateTime<Utc>,
    ts_to: DateTime<Utc>,
    device_id: u32,
}

#[derive(Properties, PartialEq)]
pub struct ModelProps {
    pub measurement_mask: MeasurementMask,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ModelProps;

    fn create(ctx: &Context<Self>) -> Self {
        let ts_to = Utc::now();
        let ts_from = Utc::now() - Duration::days(1);

        Self {
            measurements: None,
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
            Msg::MeasurementsReceived(dp) => {
                self.measurements = Some(dp);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let local_ts_from: DateTime<Local> = DateTime::from(self.ts_from);
        let local_ts_to: DateTime<Local> = DateTime::from(self.ts_to);
        let ts_from = local_ts_from.format("%Y-%m-%d").to_string();
        let ts_to = local_ts_to.format("%Y-%m-%d").to_string();

        // input callback
        let ts_from_cb = ctx.link().callback(|e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let ts_utc: DateTime<Utc> = utils::js_date_ts_to_utc(&timestring);

            info!("from (utc) {}", ts_utc.to_string());

            Msg::StartDateChanged(ts_utc)
        });

        // input callback
        let ts_to_cb = ctx.link().callback(|e: Event| {
            let target: Option<web_sys::EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let timestring = input.unwrap().value();
            let ts_utc: DateTime<Utc> = utils::js_date_ts_to_utc(&timestring);

            info!("to (utc) {}", ts_utc.to_string());

            Msg::EndDateChanged(ts_utc)
        });

        let chart_types = [
            ("Temperature in °C", MeasurementType::Temperature),
            ("Humidity in %", MeasurementType::Humidity),
            ("Pressure in hPa", MeasurementType::Pressure),
            ("Air Quality", MeasurementType::AirQuality),
            ("Battery Voltage in V", MeasurementType::BatVoltage),
        ];

        let mask = ctx.props().measurement_mask;
        let charts_html: Vec<_> = chart_types.iter().map(|(desc, ty)| {
            if mask.is_set(*ty) {
                html! {
                    <div class="panel panel-default">
                        <div class="panel-heading">
                            <h3 class="panel-title">{desc.to_string()}</h3>
                        </div>
                        <div class="panel-body">
                            <div class="row">
                                if let Some(measurements) = self.measurements.as_ref() {
                                    <div class="col-md-12">
                                        <SimpleChart ylabel={desc.to_string()} datapoints={measurements.timestamps
                                                .iter()
                                                .zip(&measurements.data[&(*ty as u32)])
                                                .map(|(a, b)| (*a, *b))
                                                .collect::<Vec<_>>()}/>
                                    </div>
                                }
                            </div>
                        </div>
                    </div>
                }
            } else {
                html!{}
            }
        }).collect();

        // html
        html! {
            <>
            <div class="panel panel-default">
                <div class="panel-heading">
                    <h3 class="panel-title">{"Date"}</h3>
                </div>
                <div class="panel-body">
                    <div class="row">
                        <div class="col-md-3">
                            <div class="input-group">
                                <span class="input-group-addon" id="basic-addon3">{"From"}</span>
                                <input type="date" onchange={ts_from_cb} class="form-control" value={ts_from}/>
                            </div>
                        </div>
                        <div class="col-md-3">
                            <div class="input-group">
                                <span class="input-group-addon" id="basic-addon3">{"To"}</span>
                                <input type="date" onchange={ts_to_cb} class="form-control" value={ts_to}/>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            {charts_html}
            </>
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
        // let device_id = self.device_id;
        let device_id = 396891554;

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let resp = req::request::measurements(
                device_id,
                Some(ts_from),
                None,
                req::MeasurementMask::ALL,
                10000,
            )
            .await;

            link.send_message(Msg::MeasurementsReceived(resp));
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
        let start_date = utils::utc_from_millis(stats.x_min)
            .duration_trunc(chrono::Duration::days(1))
            .unwrap();
        let end_date = (utils::utc_from_millis(stats.x_max) + chrono::Duration::hours(1))
            .duration_trunc(chrono::Duration::hours(1))
            .unwrap();
        let duration = end_date - start_date;

        info!(
            "{} {} {}",
            start_date.day(),
            start_date.hour(),
            start_date.minute()
        );
        info!(
            "{} {} {}",
            end_date.day(),
            end_date.hour(),
            end_date.minute()
        );

        let max_div = 12;
        let dd = chrono::Duration::hours(1);
        let dd_div = ((duration.num_hours() / dd.num_hours()) / max_div) as i32;
        let timespan = start_date..end_date;
        let h_scale = Rc::new(TimeScale::with_labeller(
            timespan,
            dd * dd_div,
            Some(Rc::from(ts_labeller(duration))),
        )) as Rc<dyn Scale<Scalar = _>>;
        let v_scale = Rc::new(LinearScale::new(
            stats.y_min.floor() - 1.0..stats.y_max.ceil() + 1.0,
            1.0,
        )) as Rc<dyn Scale<Scalar = _>>;
        let tooltip = Rc::from(series::y_tooltip()) as Rc<dyn Tooltipper<_, _>>;

        html! {
            <div>
                <svg class="chart" viewBox={format!("0 0 {} {}", WIDTH, HEIGHT)} preserveAspectRatio="none">
                    <Axis<f32>
                        name={props.ylabel.clone()}
                        orientation={Orientation::Left}
                        scale={Rc::clone(&v_scale)}
                        x1={MARGIN} y1={MARGIN} xy2={HEIGHT - MARGIN} yx2={WIDTH - MARGIN}
                        tick_len={TICK_LENGTH}
                        grid={true}
                        title={props.ylabel.clone()} />

                    <Axis<i64>
                        name="Time"
                        orientation={Orientation::Bottom}
                        scale={Rc::clone(&h_scale)}
                        x1={MARGIN} y1={HEIGHT - MARGIN} xy2={WIDTH - MARGIN} yx2={MARGIN}
                        tick_len={TICK_LENGTH}
                        grid={true}
                        title={"".to_string()} />

                    <Series<i64, f32>
                        series_type={Type::Line}
                        name="some-series"
                        data={&datapoints}
                        horizontal_scale={Rc::clone(&h_scale)}
                        horizontal_scale_step={Duration::days(1).num_milliseconds()}
                        tooltipper={Rc::clone(&tooltip)}
                        vertical_scale={Rc::clone(&v_scale)}
                        x={MARGIN} y={MARGIN} width={WIDTH - (MARGIN * 2.0)} height={HEIGHT - (MARGIN * 2.0)} />


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

fn ts_labeller(total_duration: Duration) -> impl Labeller {
    move |ts| {
        let utc = utils::utc_from_millis(ts);
        let local_date_time: DateTime<Local> = utc.into();

        if total_duration
            <= Duration::from_std(std::time::Duration::from_secs(60 * 60 * 24 * 2)).unwrap()
        {
            if local_date_time.hour() == 0
                && local_date_time.minute() == 0
                && local_date_time.second() == 0
            {
                return local_date_time.format("%H:%M\n%d/%m").to_string();
            }
            return local_date_time.format("%H:%M\n%d/%m").to_string();
        } else {
            return local_date_time.format("%d/%m").to_string();
        }
    }
}
