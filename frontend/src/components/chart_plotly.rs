use chrono::{DateTime, Local, Utc};
use log::info;
use plotly::{layout::Margin, Configuration, Layout, Plot, Scatter};
use yew::prelude::*;

use crate::utils;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
    pub unit: String,
    pub datapoints: Vec<(i64, f32)>,
    pub from_ts: DateTime<Utc>,
    pub to_ts: DateTime<Utc>,
    pub req_ts: Option<DateTime<Utc>>,
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
        plot.set_layout(
            Layout::default()
                .hover_mode(plotly::layout::HoverMode::XUnified)
                .auto_size(true)
                .margin(Margin::default().top(20).bottom(40).left(40).right(20)),
        );

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
