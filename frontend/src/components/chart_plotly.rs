use std::rc::Rc;

use chrono::{DateTime, Local, Utc};
use common::req::MeasurementType;
use log::info;
use plotly::{
    color::NamedColor,
    common::DashType::LongDash,
    layout::{Annotation, Axis, Legend, Margin, Shape, ShapeLine},
    Configuration, Layout, Plot, Scatter,
};
use yew::prelude::*;

use crate::{
    dataset::{Dataset, Stats},
    utils,
};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Overlay {
    None,
    IAQ,
    DewPoint,
    Stats,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
    pub dataset: Rc<Dataset>,
    pub kind: MeasurementType,
    pub from_ts: DateTime<Utc>,
    pub to_ts: DateTime<Utc>,
    pub req_ts: Option<DateTime<Utc>>,
    pub overlays: Vec<Overlay>,
    pub y_range: Option<(f32, f32)>,
}

#[function_component(ChartPlotly)]
pub fn chart_plotly(props: &Props) -> Html {
    let id = props.id.clone();
    let series = props.dataset.get(&props.kind).unwrap();
    let p = yew_hooks::use_async::<_, _, ()>({
        let mut plot = Plot::new();
        let trace = Scatter::new(
            series
                .data
                .iter()
                .map(|(t, _)| DateTime::<Local>::from(utils::utc_from_millis(*t)))
                .collect(),
            series.data.iter().map(|(_, y)| *y).collect(),
        )
        .text(&series.unit)
        .name(&series.name)
        .connect_gaps(false);
        plot.add_trace(trace);
        plot.set_configuration(
            Configuration::default()
                .display_logo(false)
                .editable(false)
                .display_mode_bar(plotly::configuration::DisplayModeBar::Hover)
                .autosizable(true)
                .responsive(true),
        );

        let from_ts_str = DateTime::<Local>::from(props.from_ts)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let to_ts_str = DateTime::<Local>::from(props.to_ts)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // create layout
        let mut layout = Layout::default()
            .hover_mode(plotly::layout::HoverMode::XUnified)
            .auto_size(true)
            .margin(Margin::default().top(20).bottom(40).left(40).right(20))
            .x_axis(Axis::new().range(vec![from_ts_str, to_ts_str]))
            .legend(
                Legend::new()
                    .y_anchor(plotly::common::Anchor::Bottom)
                    .x_anchor(plotly::common::Anchor::Right)
                    .orientation(plotly::common::Orientation::Horizontal)
                    .y(1.02)
                    .x(1.0),
            );

        if let Some(range) = props.y_range {
            layout = layout.y_axis(Axis::new().range(vec![range.0, range.1]));
        }

        // add overlays
        for overlay in &props.overlays {
            match overlay {
                Overlay::IAQ => add_overlay_iaq(&mut layout, props),
                Overlay::Stats => add_overlay_stats(&mut layout, props),
                Overlay::DewPoint => add_overlay_humidity(&mut plot, &mut layout, props),
                Overlay::None => (),
            }
        }

        plot.set_layout(layout);

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

    let has_data = !series.data.is_empty();

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
    let series = props.dataset.get(&props.kind).unwrap();

    if series.data.is_empty() {
        return;
    }

    let stats = series.stats();

    layout.add_shape(
        Shape::new()
            .x_ref("paper")
            .y_ref("y")
            .shape_type(plotly::layout::ShapeType::Line)
            .x0(0)
            .x1(1)
            .y0(stats.y_min)
            .y1(stats.y_min)
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
            .y0(stats.y_max)
            .y1(stats.y_max)
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
            .x(stats.x_max)
            .y(stats.y_max)
            .show_arrow(false)
            .y_shift(10.0)
            .text(format!("max: {:.2}", stats.y_max)),
    );
    layout.add_annotation(
        Annotation::new()
            .x(stats.x_min)
            .y(stats.y_min)
            .show_arrow(false)
            .y_shift(-10.0)
            .text(format!("min: {:.2}", stats.y_min)),
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

fn add_overlay_humidity(plot: &mut Plot, _layout: &mut Layout, props: &Props) {
    let series = props.dataset.get(&MeasurementType::DewTemperature).unwrap();

    let trace = Scatter::new(
        series
            .data
            .iter()
            .map(|(t, _)| DateTime::<Local>::from(utils::utc_from_millis(*t)))
            .collect(),
        series.data.iter().map(|(_, y)| *y).collect(),
    )
    .text(&series.unit)
    .name(&series.name);
    plot.add_trace(trace);
}
