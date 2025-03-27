// Physical time utilities
//
// This file contains utilities for handling physical time, including
// timestamps, durations, and conversions between different time formats.

use crate::error::{Error, Result};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A physical timestamp that represents a point in physical time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalTimestamp {
    /// Milliseconds since UNIX epoch
    millis: u64,
}

impl PhysicalTimestamp {
    /// Create a new physical timestamp from milliseconds since epoch
    pub fn from_millis(millis: u64) -> Self {
        Self { millis }
    }
    
    /// Get the current physical timestamp
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        
        Self { millis }
    }
    
    /// Get the milliseconds since epoch
    pub fn as_millis(&self) -> u64 {
        self.millis
    }
    
    /// Get the seconds since epoch
    pub fn as_secs(&self) -> u64 {
        self.millis / 1000
    }
    
    /// Convert to a SystemTime
    pub fn to_system_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.millis)
    }
    
    /// Create a timestamp from a SystemTime
    pub fn from_system_time(time: SystemTime) -> Result<Self> {
        let duration = time.duration_since(UNIX_EPOCH)
            .map_err(|e| Error::time(format!("Invalid system time: {}", e)))?;
        
        Ok(Self {
            millis: duration.as_millis() as u64,
        })
    }
    
    /// Get the elapsed time since this timestamp
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        Duration::from_millis(now.millis - self.millis)
    }
    
    /// Add a duration to this timestamp
    pub fn add_duration(&self, duration: Duration) -> Self {
        Self {
            millis: self.millis + duration.as_millis() as u64,
        }
    }
    
    /// Subtract a duration from this timestamp
    pub fn sub_duration(&self, duration: Duration) -> Result<Self> {
        if duration.as_millis() as u64 > self.millis {
            return Err(Error::time("Duration exceeds timestamp value"));
        }
        
        Ok(Self {
            millis: self.millis - duration.as_millis() as u64,
        })
    }
}

/// Format for displaying timestamps
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

/// Utilities for working with physical time
pub struct TimeUtils;

impl TimeUtils {
    /// Convert a timestamp to a string in the specified format
    pub fn format_timestamp(ts: &PhysicalTimestamp, format: TimestampFormat) -> String {
        match format {
            TimestampFormat::Millis => ts.as_millis().to_string(),
            TimestampFormat::Seconds => ts.as_secs().to_string(),
            TimestampFormat::Iso8601 => format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                // This is a simplified implementation - in a real system we would
                // use a proper date/time library to handle this correctly
                2023, 1, 1, 0, 0, 0, 0 // Placeholder values
            ),
            TimestampFormat::Rfc3339 => format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                // This is a simplified implementation - in a real system we would
                // use a proper date/time library to handle this correctly
                2023, 1, 1, 0, 0, 0, 0 // Placeholder values
            ),
            TimestampFormat::Human => format!(
                "{} seconds ago",
                (PhysicalTimestamp::now().as_secs() - ts.as_secs())
            ),
        }
    }
    
    /// Parse a timestamp from a string in the specified format
    pub fn parse_timestamp(s: &str, format: TimestampFormat) -> Result<PhysicalTimestamp> {
        match format {
            TimestampFormat::Millis => {
                let millis = s.parse::<u64>()
                    .map_err(|e| Error::time(format!("Invalid milliseconds: {}", e)))?;
                Ok(PhysicalTimestamp::from_millis(millis))
            },
            TimestampFormat::Seconds => {
                let secs = s.parse::<u64>()
                    .map_err(|e| Error::time(format!("Invalid seconds: {}", e)))?;
                Ok(PhysicalTimestamp::from_millis(secs * 1000))
            },
            _ => {
                // This is a simplified implementation - in a real system we would
                // use a proper date/time library to handle this correctly
                Err(Error::time("Complex timestamp parsing not implemented"))
            }
        }
    }
} 