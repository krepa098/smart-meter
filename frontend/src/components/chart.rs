use super::chart_plotly::Overlay;
use crate::request;
use chrono::{prelude::*, Days};
use common::req::{self, MeasurementMask, MeasurementRequestResponse, MeasurementType};
use yew::prelude::*;

pub enum Msg {
    MeasurementsReceived(MeasurementRequestResponse),
}

pub struct Model {
    measurements: Option<MeasurementRequestResponse>,
    req_ts: Option<DateTime<Utc>>,
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
            req_ts: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MeasurementsReceived(dp) => {
                self.measurements = Some(dp);
                self.req_ts = Some(Utc::now());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let chart_types = [
            (
                "Temperature",
                "°C",
                MeasurementType::Temperature,
                1.0,
                Overlay::None,
                None,
            ),
            (
                "Humidity",
                "%",
                MeasurementType::Humidity,
                1.0,
                Overlay::None,
                None,
            ),
            (
                "Pressure",
                "hPa",
                MeasurementType::Pressure,
                1.0 / 100.0,
                Overlay::None,
                None,
            ),
            (
                "Air Quality",
                "IAQ",
                MeasurementType::AirQuality,
                1.0,
                Overlay::IAQ,
                None,
            ),
            (
                "Battery Voltage",
                "V",
                MeasurementType::BatVoltage,
                1.0,
                Overlay::None,
                Some((0.0, 6.0)),
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
            .map(|(id, unit, ty, scale, overlay, y_range)| {
                if mask.is_set(*ty) {
                    html! {
                        <div class="panel panel-default">
                            <div class="panel-heading">
                                <h3 class="panel-title">{format!("{} in {}", id, unit)}</h3>
                            </div>
                            <div class="panel-body">
                                <div class="row">
                                    if let Some(measurements) = self.measurements.as_ref() {
                                        <div class="col-md-12">
                                            <crate::components::chart_plotly::ChartPlotly
                                                id={id.to_string()}
                                                unit={unit.to_string()}
                                                {from_ts}
                                                {to_ts}
                                                req_ts={self.req_ts}
                                                overlay={*overlay}
                                                y_range={y_range}
                                                datapoints={measurements.timestamps
                                                    .iter()
                                                    .zip(&measurements.data[&(*ty as u32)])
                                                    .map(|(a, b)| (*a, *b * *scale))
                                                    .collect::<Vec<_>>()}
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
                let resp = request::measurements(
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
