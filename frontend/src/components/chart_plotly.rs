use chrono::{DateTime, Local, Utc};
use log::info;
use plotly::{
    color::NamedColor,
    common::DashType::LongDash,
    layout::{Annotation, Axis, Margin, Shape, ShapeLine},
    Configuration, Layout, Plot, Scatter,
};
use yew::prelude::*;

use crate::utils;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Overlay {
    None,
    IAQ,
    Stats,
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
    pub y_range: Option<(f32, f32)>,
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

            if let Some(range) = props.y_range {
                layout = layout.y_axis(Axis::new().range(vec![range.0, range.1]));
            }

            match props.overlay {
                Overlay::IAQ => add_overlay_iaq(&mut layout, props),
                Overlay::Stats => add_overlay_stats(&mut layout, props),
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

    let has_data = !props.datapoints.is_empty();

    html! {
        <>
            // show a message when we have no data for that interval
            <div class={classes!(has_data.then_some(Some("hidden") ))}><div class="alert alert-warning" role="alert">{"Sorry, no data available for the selected time interval. Please pick a valid interval."}</div></div>
            // otherwise show the graph
            // Note: we always have to keep the chart in the BOM to be able to feed it new data
            <div class={classes!("chart", (!has_data).then_some(Some("hidden")))} id={props.id.clone()}></div>
        </>
    }
}

fn add_overlay_stats(layout: &mut Layout, props: &Props) {
    if props.datapoints.is_empty() {
        return;
    }

    let (p_min, p_max) = {
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;
        let mut x_min = 0;
        let mut x_max = 0;

        for pt in &props.datapoints {
            if y_min > pt.1 {
                y_min = pt.1;
                x_min = pt.0;
            }
            if y_max < pt.1 {
                y_max = pt.1;
                x_max = pt.0;
            }
        }

        ((x_min, y_min), (x_max, y_max))
    };

    layout.add_shape(
        Shape::new()
            .x_ref("paper")
            .y_ref("y")
            .shape_type(plotly::layout::ShapeType::Line)
            .x0(0)
            .x1(1)
            .y0(p_min.1)
            .y1(p_min.1)
            .line(
                ShapeLine::new()
                    .color(NamedColor::Black)
                    .width(1.0)
                    .dash(LongDash),
            )
            .opacity(1.0),
    );
    layout.add_shape(
        Shape::new()
            .x_ref("paper")
            .y_ref("y")
            .shape_type(plotly::layout::ShapeType::Line)
            .x0(0)
            .x1(1)
            .y0(p_max.1)
            .y1(p_max.1)
            .line(
                ShapeLine::new()
                    .color(NamedColor::Black)
                    .width(1.0)
                    .dash(LongDash),
            )
            .opacity(1.0),
    );
    layout.add_annotation(
        Annotation::new()
            .x(p_max.0)
            .y(p_max.1)
            .show_arrow(false)
            .y_shift(10.0)
            .text(format!("max: {:.2}", p_max.1)),
    );
    layout.add_annotation(
        Annotation::new()
            .x(p_min.0)
            .y(p_min.1)
            .show_arrow(false)
            .y_shift(-10.0)
            .text(format!("min: {:.2}", p_min.1)),
    );
}

fn add_overlay_iaq(layout: &mut Layout, _props: &Props) {
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
