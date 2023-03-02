use chrono::{DateTime, Local, Utc};
use log::info;
use plotly::{
    color::NamedColor,
    layout::{Axis, Margin, Shape},
    Configuration, Layout, Plot, Scatter,
};
use yew::prelude::*;

use crate::utils;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Overlay {
    None,
    IAQ,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
    pub unit: String,
    pub datapoints: Vec<(i64, f32)>,
    pub from_ts: DateTime<Utc>,
    pub to_ts: DateTime<Utc>,
    pub req_ts: Option<DateTime<Utc>>,
    pub overlay: Overlay,
}

#[function_component(ChartPlotly)]
pub fn chart_plotly(props: &Props) -> Html {
    let id = props.id.clone();
    let p = yew_hooks::use_async::<_, _, ()>({
        let mut plot = Plot::new();
        let trace = Scatter::new(
            props
                .datapoints
                .iter()
                .map(|v| DateTime::<Local>::from(utils::utc_from_millis(v.0)))
                .collect(),
            props.datapoints.iter().map(|v| v.1).collect(),
        )
        .text(&props.unit);
        plot.add_trace(trace);
        plot.set_configuration(
            Configuration::default()
                .display_logo(false)
                .editable(false)
                .display_mode_bar(plotly::configuration::DisplayModeBar::Hover),
        );

        let from_ts_str = DateTime::<Local>::from(props.from_ts)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let to_ts_str = DateTime::<Local>::from(props.to_ts)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        plot.set_layout({
            let mut layout = Layout::default()
                .hover_mode(plotly::layout::HoverMode::XUnified)
                .auto_size(true)
                .margin(Margin::default().top(20).bottom(40).left(40).right(20))
                .x_axis(Axis::new().range(vec![from_ts_str, to_ts_str]));

            match props.overlay {
                Overlay::IAQ => {
                    layout.add_shape(
                        Shape::new()
                            .x_ref("paper")
                            .y_ref("y")
                            .shape_type(plotly::layout::ShapeType::Rect)
                            .x0(0)
                            .x1(1)
                            .y0(0.0)
                            .y1(100.0)
                            .fill_color(NamedColor::Green)
                            .opacity(0.05),
                    );
                    layout.add_shape(
                        Shape::new()
                            .x_ref("paper")
                            .y_ref("y")
                            .shape_type(plotly::layout::ShapeType::Rect)
                            .x0(0)
                            .x1(1)
                            .y0(100.0)
                            .y1(200.0)
                            .fill_color(NamedColor::Yellow)
                            .opacity(0.05),
                    );
                    layout.add_shape(
                        Shape::new()
                            .x_ref("paper")
                            .y_ref("y")
                            .shape_type(plotly::layout::ShapeType::Rect)
                            .x0(0)
                            .x1(1)
                            .y0(200.0)
                            .y1(500.0)
                            .fill_color(NamedColor::Red)
                            .opacity(0.05),
                    );
                }
                Overlay::None => (),
            }

            layout
        });

        async move {
            plotly::bindings::new_plot(&id, &plot).await;
            Ok(())
        }
    });

    use_effect_with_deps(
        move |_| {
            p.run();
            || ()
        },
        // replot whenever these change
        props.req_ts,
    );

    html! {
        <div class="chart" id={props.id.clone()}></div>
    }
}
