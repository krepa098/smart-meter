// pub fn duration_to_now(old: &std::time::Duration) -> std::time::Duration {}

use chrono::{DateTime, Local, Utc};
use std::time::Duration;

pub fn duration_since_epoch(stamp_older: u64) -> Duration {
    let now_since_epoch = Duration::from_secs((Utc::now().timestamp_millis() / 1000) as u64);
    let ts = Duration::from_secs(stamp_older);
    now_since_epoch - ts
}

// offset in hours
pub fn timezone_offset() -> i32 {
    let offset_in_sec = Local::now().offset().local_minus_utc();
    offset_in_sec / 60 / 60
}

pub fn js_ts_to_utc(timestring: &str) -> DateTime<Utc> {
    let ts = chrono::NaiveDateTime::parse_from_str(&timestring, "%Y-%m-%dT%H:%M")
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    let ts_utc: DateTime<Utc> = DateTime::from(ts);
    ts_utc
}
