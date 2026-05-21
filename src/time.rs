use chrono::{DateTime, TimeZone, Utc};

const APPLE_EPOCH_UNIX_SECS: i64 = 978_307_200;

pub fn apple_timestamp_to_utc(raw: i64) -> Option<DateTime<Utc>> {
    let (secs, nsecs) = if raw > 1_000_000_000_000_000_000 {
        (raw / 1_000_000_000, (raw % 1_000_000_000) as u32)
    } else if raw > 1_000_000_000_000_000 {
        (raw / 1_000_000, ((raw % 1_000_000) * 1_000) as u32)
    } else if raw > 1_000_000_000_000 {
        (raw / 1_000, ((raw % 1_000) * 1_000_000) as u32)
    } else {
        (raw, 0)
    };
    Utc.timestamp_opt(APPLE_EPOCH_UNIX_SECS + secs, nsecs)
        .single()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_nanosecond_apple_timestamp() {
        let dt = apple_timestamp_to_utc(738_000_000_000_000_000).expect("valid");
        assert!(dt.timestamp() > 1_600_000_000);
    }
}
