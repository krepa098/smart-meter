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
