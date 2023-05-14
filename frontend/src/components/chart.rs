use super::chart_plotly::Overlay;
use crate::request;
use chrono::{prelude::*, Days};
use common::req::{self, MeasurementMask, MeasurementRequestResponse, MeasurementType};
use yew::prelude::*;

pub enum Msg {
    MeasurementsReceived(MeasurementRequestResponse),
}

struct SeriesProps {
    id: String,
    unit: String,
    ty: MeasurementType,
    scale: f32,
    overlay: Overlay,
    y_range: Option<(f32, f32)>,
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

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            measurements: None,
            req_ts: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MeasurementsReceived(dp) => {
                self.measurements = Some(dp);
                self.req_ts = Some(Utc::now());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let series_props = [
            SeriesProps {
                id: "Temperature".to_owned(),
                unit: "°C".to_owned(),
                ty: MeasurementType::Temperature,
                scale: 1.0,
                overlay: Overlay::None,
                y_range: None,
            },
            SeriesProps {
                id: "Humidity".to_owned(),
                unit: "%".to_owned(),
                ty: MeasurementType::Humidity,
                scale: 1.0,
                overlay: Overlay::None,
                y_range: None,
            },
            SeriesProps {
                id: "Pressure".to_owned(),
                unit: "hPa".to_owned(),
                ty: MeasurementType::Pressure,
                scale: 1.0 / 100.0,
                overlay: Overlay::None,
                y_range: None,
            },
            SeriesProps {
                id: "Air Quality".to_owned(),
                unit: "IAQ".to_owned(),
                ty: MeasurementType::AirQuality,
                scale: 1.0,
                overlay: Overlay::IAQ,
                y_range: None,
            },
            SeriesProps {
                id: "Battery Voltage".to_owned(),
                unit: "V".to_owned(),
                ty: MeasurementType::BatVoltage,
                scale: 1.0,
                overlay: Overlay::None,
                y_range: Some((4.0, 6.0)),
            },
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
        let charts_html: Vec<_> = series_props
            .iter()
            .map(|prop| {
                if mask.is_set(prop.ty) {
                    html! {
                        <div class="panel panel-default">
                            <div class="panel-heading">
                                <h3 class="panel-title">{format!("{} in {}", prop.id, prop.unit)}</h3>
                            </div>
                            <div class="panel-body">
                                <div class="row">
                                    if let Some(measurements) = self.measurements.as_ref() {
                                        <div class="col-md-12">
                                            <crate::components::chart_plotly::ChartPlotly
                                                id={prop.id.to_string()}
                                                unit={prop.unit.to_string()}
                                                {from_ts}
                                                {to_ts}
                                                req_ts={self.req_ts}
                                                overlay={prop.overlay}
                                                y_range={prop.y_range}
                                                datapoints={measurements.timestamps
                                                    .iter()
                                                    .zip(&measurements.data[&(prop.ty as u32)])
                                                    .map(|(a, b)| (*a, *b * prop.scale))
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
