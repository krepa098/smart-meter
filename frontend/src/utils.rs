// pub fn duration_to_now(old: &std::time::Duration) -> std::time::Duration {}

use chrono::{DateTime, NaiveDate, Utc};
use log::info;
use std::time::Duration;

pub fn duration_since_epoch(stamp_older: u64) -> Duration {
    let now_since_epoch = Duration::from_secs((Utc::now().timestamp_millis() / 1000) as u64);
    let ts = Duration::from_secs(stamp_older);
    now_since_epoch - ts
}

pub fn utc_from_millis(millis: i64) -> DateTime<Utc> {
    let naive = chrono::NaiveDateTime::from_timestamp_millis(millis).unwrap();
    DateTime::<Utc>::from_utc(naive, Utc)
}

pub fn utc_now() -> DateTime<Utc> {
    chrono::Utc::now()
}

pub fn js_date_ts_to_naive(timestring: &str) -> NaiveDate {
    chrono::NaiveDate::parse_from_str(timestring, "%Y-%m-%d").unwrap()
}

pub fn naive_date_to_js(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn ceil_multiple(val: f32, q: f32) -> f32 {
    (val / q).ceil() * q
}

pub fn floor_multiple(val: f32, q: f32) -> f32 {
    (val / q).floor() * q
}

// pub fn stats

pub struct SeriesStats {
    pub x_min: i64,
    pub x_max: i64,
    pub y_max: f32,
    pub y_min: f32,
}

impl SeriesStats {
    pub fn x_range(&self) -> i64 {
        self.x_max - self.x_min
    }

    pub fn y_range(&self) -> f32 {
        self.y_max - self.y_min
    }
}

pub trait Stats {
    fn stats(&self) -> SeriesStats;
}

impl Stats for Vec<(i64, f32)> {
    fn stats(&self) -> SeriesStats {
        let mut x_min = i64::MAX;
        let mut x_max = i64::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for (x, y) in self.iter() {
            x_min = x_min.min(*x);
            x_max = x_max.max(*x);
            y_min = y_min.min(*y);
            y_max = y_max.max(*y);
        }

        SeriesStats {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}
