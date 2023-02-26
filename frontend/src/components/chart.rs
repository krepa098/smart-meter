use crate::{
    req::{self, MeasurementMask, MeasurementRequestResponse, MeasurementType},
    utils,
};
use chrono::{prelude::*, Days, Duration, DurationRound};
use log::info;
use std::rc::Rc;
use utils::Stats;
use yew::prelude::*;
use yew_chart::{
    axis::{Axis, Orientation, Scale},
    linear_axis_scale::LinearScale,
    series::{self, Series, Tooltipper, Type},
    time_axis_scale::TimeScale,
};

const WIDTH: f32 = 900.0;
const HEIGHT: f32 = 320.0;
const MARGIN: f32 = 50.0;
const TICK_LENGTH: f32 = 15.0;

pub enum Msg {
    MeasurementsReceived(MeasurementRequestResponse),
}

pub struct Model {
    measurements: Option<MeasurementRequestResponse>,
    measurements_from_ts: Option<DateTime<Utc>>, // oldest available measurement
}

#[derive(Properties, PartialEq)]
pub struct ModelProps {
    pub device_id: Option<u32>,
    pub measurement_mask: MeasurementMask,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ModelProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            measurements: None,
            measurements_from_ts: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MeasurementsReceived(dp) => {
                self.measurements = Some(dp);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let chart_types = [
            ("Temperature in °C", MeasurementType::Temperature, 1.0, 0.5),
            ("Humidity in %", MeasurementType::Humidity, 1.0, 5.0),
            (
                "Pressure in hPa",
                MeasurementType::Pressure,
                1.0 / 100.0,
                10.0,
            ),
            ("Air Quality", MeasurementType::AirQuality, 1.0, 25.0),
            (
                "Battery Voltage in mV",
                MeasurementType::BatVoltage,
                1000.0,
                100.0,
            ),
        ];

        let from_ts: DateTime<Utc> = DateTime::from(
            ctx.props()
                .from_date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
        );
        let to_ts: DateTime<Utc> = DateTime::from(
            (ctx.props().to_date + Days::new(1))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
        );

        let mask = ctx.props().measurement_mask;
        let charts_html: Vec<_> = chart_types
            .iter()
            .map(|(desc, ty, scale, optimal_div)| {
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
                                            <SimpleChart ylabel={desc.to_string()}
                                                datapoints={measurements.timestamps
                                                    .iter()
                                                    .zip(&measurements.data[&(*ty as u32)])
                                                    .map(|(a, b)| (*a, *b * *scale))
                                                    .collect::<Vec<_>>()}
                                                {from_ts}
                                                {to_ts}
                                                {optimal_div}
                                                max_div={8}
                                            />
                                        </div>
                                    }
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            })
            .collect();

        // html
        html! {
            <>
            {charts_html}
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.request_datapoints(ctx);
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.request_datapoints(ctx);
        true
    }
}

impl Model {
    pub fn request_datapoints(&self, ctx: &Context<Self>) {
        let from_ts: DateTime<Utc> = DateTime::from(
            ctx.props()
                .from_date
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
        );

        let to_ts: DateTime<Utc> = DateTime::from(
            (ctx.props().to_date + Days::new(1))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
        );

        if let Some(device_id) = ctx.props().device_id {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                let resp = req::request::measurements(
                    device_id,
                    Some(from_ts),
                    Some(to_ts),
                    req::MeasurementMask::ALL,
                    10000,
                )
                .await;

                link.send_message(Msg::MeasurementsReceived(resp));
            });
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ChartProps {
    ylabel: String,
    datapoints: Vec<(i64, f32)>,
    from_ts: DateTime<Utc>,
    to_ts: DateTime<Utc>,
    max_div: u32,
    optimal_div: f32,
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
        let duration = props.to_ts - props.from_ts;

        // horizontal scale (time)
        let max_div = 12;
        let dd = chrono::Duration::hours(1);
        let dd_div = ((duration.num_hours() / dd.num_hours()) / max_div) as i32;
        let timespan = props.from_ts..props.to_ts;
        let h_scale = Rc::new(TimeScale::with_labeller(
            timespan,
            dd * dd_div,
            Some(Rc::from(ts_labeller(duration))),
        )) as Rc<dyn Scale<Scalar = _>>;

        // vertical scale (measurements)
        let optimal_div = props.optimal_div;
        let max_div = props.max_div;
        let y_max = utils::ceil_multiple(stats.y_max, optimal_div);
        let y_min = utils::floor_multiple(stats.y_min, optimal_div);

        let range = y_max - y_min;
        let q = ((range / optimal_div) / (max_div as f32)).ceil();
        let steps = optimal_div * q;
        let y_max = utils::ceil_multiple(stats.y_max, optimal_div * q);
        let y_min = utils::floor_multiple(stats.y_min, optimal_div * q);

        let v_scale = Rc::new(LinearScale::with_labeller(
            y_min..y_max,
            steps,
            Some(Rc::from(linear_scale_labeller())),
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
                        title="" />

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

fn ts_labeller(total_duration: Duration) -> impl yew_chart::time_axis_scale::Labeller {
    move |ts| {
        let utc = utils::utc_from_millis(ts);
        let local_date_time: DateTime<Local> = utc.into();

        if total_duration
            <= Duration::from_std(std::time::Duration::from_secs(60 * 60 * 24 * 3)).unwrap()
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

fn linear_scale_labeller() -> impl yew_chart::linear_axis_scale::Labeller {
    move |y: f32| {
        let decimal = y - y.floor();
        if decimal == 0.0 {
            format!("{:.0}", y)
        } else {
            format!("{:.1}", y)
        }
    }
}
