use chrono::{DateTime, Utc};

pub fn ms_since_epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn utc_with_offset(offset_ms: i64) -> DateTime<Utc> {
    let now = Utc::now();
    now.checked_add_signed(chrono::Duration::milliseconds(offset_ms))
        .unwrap()
}
