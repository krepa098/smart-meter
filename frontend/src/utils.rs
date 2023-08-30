use chrono::{DateTime, NaiveDate, Utc};
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
