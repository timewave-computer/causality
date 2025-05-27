// Time utilities for Causality Engine
//
// This module provides functions for converting between different time formats
// used in the Causality system.

use causality_types::Timestamp;
use chrono::{DateTime, TimeZone, Utc};

/// Convert a Timestamp to DateTime<Utc>
pub fn timestamp_to_datetime(timestamp: Timestamp) -> DateTime<Utc> {
    // Timestamp is in seconds, convert to DateTime
    match Utc.timestamp_opt(timestamp.0 as i64, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => Utc::now() // Fallback to current time if conversion fails
    }
}

/// Convert a DateTime<Utc> to Timestamp
pub fn datetime_to_timestamp(datetime: DateTime<Utc>) -> Timestamp {
    // Extract seconds from DateTime and convert to Timestamp
    Timestamp(datetime.timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timestamp_datetime_conversion() {
        // Test conversion from Timestamp to DateTime
        let ts = Timestamp(1625097600); // 2021-07-01 00:00:00 UTC
        let dt = timestamp_to_datetime(ts);
        assert_eq!(dt.timestamp(), 1625097600);
        
        // Test conversion from DateTime to Timestamp
        let dt = Utc.ymd(2021, 7, 1).and_hms(0, 0, 0);
        let ts = datetime_to_timestamp(dt);
        assert_eq!(ts.0, 1625097600);
        
        // Test roundtrip conversion
        let original_ts = Timestamp(1625097600);
        let dt = timestamp_to_datetime(original_ts);
        let round_trip_ts = datetime_to_timestamp(dt);
        assert_eq!(original_ts, round_trip_ts);
    }
} 