use std::{
    ops::{Add, Sub},
    rc::Rc,
};

use chrono::{Duration, Utc};
use reqwest::header::ACCEPT;
use yew::{prelude::*, virtual_dom::VNode};
use yew_chart::{
    axis::{Axis, Orientation, Scale},
    linear_axis_scale::LinearScale,
    series::{self, Labeller, Series, Tooltipper, Type},
    time_axis_scale::TimeScale,
};

const WIDTH: f32 = 533.0;
const HEIGHT: f32 = 300.0;
const MARGIN: f32 = 50.0;
const TICK_LENGTH: f32 = 10.0;

#[function_component(Chart)]
pub fn chart() -> Html {
    let client = reqwest::Client::new();
    let measurements = use_state(|| None);
    let circle_text_labeller = Rc::from(series::circle_label()) as Rc<dyn Labeller>;

    {
        let measurements = measurements.clone();
        let client = client.clone();
        let from_date = chrono::Utc::now()
            .checked_sub_days(chrono::Days::new(1))
            .unwrap()
            .timestamp_millis();
        let limit = 200000;

        if measurements.is_none() {
            wasm_bindgen_futures::spawn_local(async move {
                let device_id = 396891554;
                let resp = client
                    .get("http://127.0.0.1:8081/api/measurements/by_date")
                    .query(&[
                        ("device_id", device_id),
                        ("from_date", from_date),
                        ("limit", limit),
                    ])
                    .header(ACCEPT, "application/json")
                    .send()
                    .await
                    .unwrap()
                    .json::<Vec<serde_json::Map<String, serde_json::Value>>>()
                    .await
                    .unwrap();

                let datapoints: Vec<_> = resp
                    .iter()
                    .map(|m| {
                        (
                            m.get("timestamp").unwrap().as_i64().unwrap(),
                            m.get("temperature").unwrap().as_f64().unwrap() as f32,
                            // Some(circle_text_labeller.clone()),
                            None,
                        )
                    })
                    .collect();

                measurements.set(Some(Rc::new(datapoints)));
            });
        }
    }

    let end_date = Utc::now();
    let start_date = end_date.sub(Duration::hours(12));
    let timespan = start_date..end_date;

    let circle_labeller = Rc::from(series::circle_label()) as Rc<dyn Labeller>;

    // let data_set = Rc::new(vec![
    //     (start_date.timestamp_millis(), 1.0, None),
    //     (
    //         start_date.add(Duration::days(1)).timestamp_millis(),
    //         4.0,
    //         None,
    //     ),
    //     (
    //         start_date.add(Duration::days(2)).timestamp_millis(),
    //         3.0,
    //         None,
    //     ),
    //     (
    //         start_date.add(Duration::days(3)).timestamp_millis(),
    //         2.0,
    //         None,
    //     ),
    //     (
    //         start_date.add(Duration::days(4)).timestamp_millis(),
    //         5.0,
    //         Some(circle_text_labeller.clone()),
    //     ),
    // ]);

    let h_scale =
        Rc::new(TimeScale::new(timespan, Duration::minutes(60))) as Rc<dyn Scale<Scalar = _>>;
    let v_scale = Rc::new(LinearScale::new(0.0..30.0, 2.0)) as Rc<dyn Scale<Scalar = _>>;

    let tooltip = Rc::from(series::y_tooltip()) as Rc<dyn Tooltipper<_, _>>;

    // let mes = match measurements.as_ref() {
    //     Some(mes) => mes,
    //     _ => Rc::from(vec![]),
    // };

    html! {
            <div>
                if let Some(measurements) = measurements.as_ref() {
                    <svg class="chart" viewBox={format!("0 0 {} {}", WIDTH, HEIGHT)} preserveAspectRatio="none">
                    <Series<i64, f32>
                        series_type={Type::Line}
                        name="some-series"
                        data={measurements}
                        horizontal_scale={Rc::clone(&h_scale)}
                        horizontal_scale_step={Duration::hours(2).num_milliseconds()}
                        tooltipper={Rc::clone(&tooltip)}
                        vertical_scale={Rc::clone(&v_scale)}
                        x={MARGIN} y={MARGIN} width={WIDTH - (MARGIN * 2.0)} height={HEIGHT - (MARGIN * 2.0)} />

                    <Axis<f32>
                        name="Temperature °C"
                        orientation={Orientation::Left}
                        scale={Rc::clone(&v_scale)}
                        x1={MARGIN} y1={MARGIN} xy2={HEIGHT - MARGIN}
                        tick_len={TICK_LENGTH}
                        title={"Temperature °C".to_string()} />

                    <Axis<i64>
                        name="Time"
                        orientation={Orientation::Bottom}
                        scale={Rc::clone(&h_scale)}
                        x1={MARGIN} y1={HEIGHT - MARGIN} xy2={WIDTH - MARGIN}
                        tick_len={TICK_LENGTH}
                        title={"Time".to_string()} />

                    </svg>
                }
            </div>
    }
}
