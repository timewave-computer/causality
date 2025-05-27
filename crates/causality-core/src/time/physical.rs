// Physical Time Utilities
//
// This module provides time formatting utilities that complement
// the core Clock implementations from the clock module.

use std::time::{SystemTime, Duration, UNIX_EPOCH};
use chrono::{DateTime, Utc, TimeZone};

use crate::time::error::TimeError;
use crate::time::Timestamp;

/// Timestamp format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampFormat {
    /// Milliseconds since epoch
    Millis,
    /// Seconds since epoch
    Seconds,
    /// ISO 8601 format (YYYY-MM-DDTHH:MM:SS.sssZ)
    Iso8601,
    /// RFC 3339 format (YYYY-MM-DD HH:MM:SS.sss)
    Rfc3339,
    /// Human-readable format
    Human,
}

/// Timestamp utilities for formatting, parsing, and conversion between 
/// different time representations
pub struct TimeUtils;

impl TimeUtils {
    /// Convert system time to DateTime
    pub fn system_time_to_date_time(system_time: SystemTime) -> Result<DateTime<Utc>, TimeError> {
        let duration_since_epoch = system_time
            .duration_since(UNIX_EPOCH)
            .map_err(|e| TimeError::Other(format!("System time error: {}", e)))?;
        
        let secs = duration_since_epoch.as_secs();
        let nanos = duration_since_epoch.subsec_nanos();
        
        if secs > i64::MAX as u64 {
            return Err(TimeError::OutOfBounds("Timestamp too large for DateTime".into()));
        }
        
        Ok(Utc.timestamp_opt(secs as i64, nanos).single()
            .ok_or_else(|| TimeError::InvalidFormat("Invalid timestamp for DateTime".into()))?)
    }
    
    /// Convert DateTime to system time
    pub fn date_time_to_system_time(dt: DateTime<Utc>) -> SystemTime {
        let duration = Duration::new(dt.timestamp() as u64, dt.timestamp_subsec_nanos());
        UNIX_EPOCH + duration
    }

    /// Convert Timestamp to DateTime
    pub fn timestamp_to_date_time(ts: &Timestamp) -> DateTime<Utc> {
        let secs = ts.as_secs() as i64;
        let nanos = (ts.as_nanos() % 1_000_000_000) as u32;
        Utc.timestamp_opt(secs, nanos).single().unwrap_or_default()
    }

    /// Convert DateTime to Timestamp
    pub fn date_time_to_timestamp(dt: &DateTime<Utc>) -> Timestamp {
        let secs = dt.timestamp() as u64;
        let nanos = dt.timestamp_subsec_nanos() as u64;
        Timestamp::from_nanos(secs * 1_000_000_000 + nanos)
    }
    
    /// Format a timestamp according to the specified format
    pub fn format_timestamp(ts: &Timestamp, format: TimestampFormat) -> String {
        match format {
            TimestampFormat::Millis => format!("{}", ts.as_millis()),
            TimestampFormat::Seconds => format!("{}", ts.as_secs()),
            TimestampFormat::Iso8601 => {
                let dt = Self::timestamp_to_date_time(ts);
                dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            },
            TimestampFormat::Rfc3339 => {
                let dt = Self::timestamp_to_date_time(ts);
                dt.to_rfc3339()
            },
            TimestampFormat::Human => {
                let dt = Self::timestamp_to_date_time(ts);
                dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string()
            },
        }
    }
    
    /// Parse a timestamp from a string
    pub fn parse_timestamp(s: &str, format: TimestampFormat) -> Result<Timestamp, TimeError> {
        match format {
            TimestampFormat::Millis => {
                s.parse::<u64>()
                    .map(Timestamp::from_millis)
                    .map_err(|e| TimeError::InvalidFormat(format!("Invalid milliseconds: {}", e)))
            },
            TimestampFormat::Seconds => {
                s.parse::<u64>()
                    .map(Timestamp::from_secs)
                    .map_err(|e| TimeError::InvalidFormat(format!("Invalid seconds: {}", e)))
            },
            TimestampFormat::Iso8601 | TimestampFormat::Rfc3339 => {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| {
                        let utc_dt = dt.with_timezone(&Utc);
                        Self::date_time_to_timestamp(&utc_dt)
                    })
                    .map_err(|e| TimeError::InvalidFormat(format!("Invalid timestamp format: {}", e)))
            },
            TimestampFormat::Human => {
                Err(TimeError::InvalidFormat("Parsing human-readable format not supported".into()))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::clock;
    
    #[test]
    fn test_timestamp_datetime_conversion() {
        let now = Timestamp::from_secs(1600000000);
        let dt = TimeUtils::timestamp_to_date_time(&now);
        let converted_back = TimeUtils::date_time_to_timestamp(&dt);
        
        assert_eq!(now, converted_back, "Timestamp to DateTime and back should be lossless");
    }
    
    #[test]
    fn test_format_timestamp() {
        let ts = Timestamp::from_secs(1600000000);
        
        // Test different formats
        let millis = TimeUtils::format_timestamp(&ts, TimestampFormat::Millis);
        let seconds = TimeUtils::format_timestamp(&ts, TimestampFormat::Seconds);
        let iso = TimeUtils::format_timestamp(&ts, TimestampFormat::Iso8601);
        
        assert_eq!(millis, "1600000000000");
        assert_eq!(seconds, "1600000000");
        assert!(iso.contains("2020-09-13T12:26:40"), "ISO format should contain the correct date and time");
    }
    
    #[test]
    fn test_parse_timestamp() {
        // Test parsing seconds
        let secs_result = TimeUtils::parse_timestamp("1600000000", TimestampFormat::Seconds);
        assert!(secs_result.is_ok());
        assert_eq!(secs_result.unwrap(), Timestamp::from_secs(1600000000));
        
        // Test parsing milliseconds
        let millis_result = TimeUtils::parse_timestamp("1600000000000", TimestampFormat::Millis);
        assert!(millis_result.is_ok());
        assert_eq!(millis_result.unwrap(), Timestamp::from_millis(1600000000000));
        
        // Test parsing ISO format
        let iso_result = TimeUtils::parse_timestamp("2020-09-13T12:26:40Z", TimestampFormat::Iso8601);
        assert!(iso_result.is_ok());
        let expected = Timestamp::from_millis(1600000000000);
        assert_eq!(iso_result.unwrap().as_secs(), expected.as_secs());
    }
} 