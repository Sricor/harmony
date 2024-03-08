use chrono::{Duration, Utc};

pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn timestamp_millis_after_days(days: i64) -> i64 {
    (Utc::now() + Duration::days(days)).timestamp_millis()
}

pub fn is_timestamp_millis_expired(timestamp: i64) -> bool {
    timestamp < timestamp_millis()
}
