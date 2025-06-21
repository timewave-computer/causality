//! Fixed-point arithmetic for deterministic decimal calculations
//!
//! This module provides a FixedPoint type that enables precise decimal arithmetic
//! without the non-determinism of floating point operations.

use serde::{Serialize, Deserialize};
use ssz::{Encode, Decode, DecodeError};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::cmp::Ordering;

/// Fixed-point decimal number with configurable precision
/// 
/// Uses integer arithmetic internally to ensure deterministic results
/// across all platforms and architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixedPoint {
    /// Raw integer value scaled by SCALE_FACTOR
    raw: i64,
}

impl FixedPoint {
    /// Scale factor for fixed-point arithmetic (6 decimal places)
    pub const SCALE_FACTOR: i64 = 1_000_000;
    
    /// Maximum representable value
    pub const MAX: FixedPoint = FixedPoint { raw: i64::MAX };
    
    /// Minimum representable value  
    pub const MIN: FixedPoint = FixedPoint { raw: i64::MIN };
    
    /// Zero value
    pub const ZERO: FixedPoint = FixedPoint { raw: 0 };
    
    /// One value
    pub const ONE: FixedPoint = FixedPoint { raw: Self::SCALE_FACTOR };
    
    /// Create a FixedPoint from an integer
    pub const fn from_int(value: i64) -> Self {
        FixedPoint {
            raw: value.saturating_mul(Self::SCALE_FACTOR),
        }
    }
    
    /// Create a FixedPoint from a rational number (numerator/denominator)
    pub fn from_rational(numerator: i64, denominator: i64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        
        // Use 128-bit arithmetic to avoid overflow during calculation
        let scaled_num = (numerator as i128) * (Self::SCALE_FACTOR as i128);
        let result = scaled_num / (denominator as i128);
        
        if result > i64::MAX as i128 || result < i64::MIN as i128 {
            None
        } else {
            Some(FixedPoint { raw: result as i64 })
        }
    }
    
    /// Create a FixedPoint from raw scaled value
    pub const fn from_raw(raw: i64) -> Self {
        FixedPoint { raw }
    }
    
    /// Get the raw scaled integer value
    pub const fn raw(&self) -> i64 {
        self.raw
    }
    
    /// Convert to integer (truncating fractional part)
    pub const fn to_int(&self) -> i64 {
        self.raw / Self::SCALE_FACTOR
    }
    
    /// Get the fractional part as an integer (0 to SCALE_FACTOR-1)
    pub const fn fractional_part(&self) -> i64 {
        self.raw % Self::SCALE_FACTOR
    }
    
    /// Check if this is zero
    pub const fn is_zero(&self) -> bool {
        self.raw == 0
    }
    
    /// Check if this is positive
    pub const fn is_positive(&self) -> bool {
        self.raw > 0
    }
    
    /// Check if this is negative
    pub const fn is_negative(&self) -> bool {
        self.raw < 0
    }
    
    /// Get absolute value
    pub const fn abs(&self) -> Self {
        FixedPoint {
            raw: if self.raw < 0 { -self.raw } else { self.raw }
        }
    }
    
    /// Saturating addition
    pub const fn saturating_add(&self, other: Self) -> Self {
        FixedPoint {
            raw: self.raw.saturating_add(other.raw)
        }
    }
    
    /// Saturating subtraction
    pub const fn saturating_sub(&self, other: Self) -> Self {
        FixedPoint {
            raw: self.raw.saturating_sub(other.raw)
        }
    }
    
    /// Saturating multiplication
    pub fn saturating_mul(&self, other: Self) -> Self {
        // Use 128-bit arithmetic to avoid overflow
        let result = (self.raw as i128) * (other.raw as i128) / (Self::SCALE_FACTOR as i128);
        
        let clamped = if result > i64::MAX as i128 {
            i64::MAX
        } else if result < i64::MIN as i128 {
            i64::MIN
        } else {
            result as i64
        };
        
        FixedPoint { raw: clamped }
    }
    
    /// Checked division
    pub fn checked_div(&self, other: Self) -> Option<Self> {
        if other.raw == 0 {
            return None;
        }
        
        // Use 128-bit arithmetic to maintain precision
        let scaled_dividend = (self.raw as i128) * (Self::SCALE_FACTOR as i128);
        let result = scaled_dividend / (other.raw as i128);
        
        if result > i64::MAX as i128 || result < i64::MIN as i128 {
            None
        } else {
            Some(FixedPoint { raw: result as i64 })
        }
    }
    
    /// Calculate percentage: self * (percent / 100)
    pub fn percentage(&self, percent: Self) -> Self {
        let hundred = FixedPoint::from_int(100);
        self.saturating_mul(percent).checked_div(hundred).unwrap_or(FixedPoint::ZERO)
    }
    
    /// Create a percentage from integer (e.g., 50 -> 0.5)
    pub fn from_percentage(percent: i64) -> Self {
        FixedPoint::from_rational(percent, 100).unwrap_or(FixedPoint::ZERO)
    }
}

impl Add for FixedPoint {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        FixedPoint {
            raw: self.raw.saturating_add(other.raw)
        }
    }
}

impl Sub for FixedPoint {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        FixedPoint {
            raw: self.raw.saturating_sub(other.raw)
        }
    }
}

impl Mul for FixedPoint {
    type Output = Self;
    
    fn mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl Div for FixedPoint {
    type Output = Self;
    
    fn div(self, other: Self) -> Self {
        self.checked_div(other).unwrap_or(FixedPoint::ZERO)
    }
}

impl Neg for FixedPoint {
    type Output = Self;
    
    fn neg(self) -> Self {
        FixedPoint {
            raw: -self.raw
        }
    }
}

impl PartialOrd for FixedPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FixedPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl fmt::Display for FixedPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let integer_part = self.to_int();
        let fractional_part = self.fractional_part().abs();
        
        if self.is_negative() && integer_part == 0 {
            write!(f, "-{}.{:06}", integer_part, fractional_part)
        } else {
            write!(f, "{}.{:06}", integer_part, fractional_part)
        }
    }
}

impl From<i64> for FixedPoint {
    fn from(value: i64) -> Self {
        FixedPoint::from_int(value)
    }
}

impl From<i32> for FixedPoint {
    fn from(value: i32) -> Self {
        FixedPoint::from_int(value as i64)
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for FixedPoint {
    fn is_ssz_fixed_len() -> bool {
        true
    }
    
    fn ssz_fixed_len() -> usize {
        8 // i64 size
    }
    
    fn ssz_bytes_len(&self) -> usize {
        8
    }
    
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.raw.to_le_bytes());
    }
}

impl Decode for FixedPoint {
    fn is_ssz_fixed_len() -> bool {
        true
    }
    
    fn ssz_fixed_len() -> usize {
        8
    }
    
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 8 {
            return Err(DecodeError::InvalidByteLength { 
                len: bytes.len(), 
                expected: 8 
            });
        }
        
        let mut raw_bytes = [0u8; 8];
        raw_bytes.copy_from_slice(bytes);
        let raw = i64::from_le_bytes(raw_bytes);
        Ok(FixedPoint { raw })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_arithmetic() {
        let a = FixedPoint::from_int(5);
        let b = FixedPoint::from_int(3);
        
        assert_eq!(a + b, FixedPoint::from_int(8));
        assert_eq!(a - b, FixedPoint::from_int(2));
        assert_eq!(a * b, FixedPoint::from_int(15));
        assert_eq!(a / b, FixedPoint::from_rational(5, 3).unwrap());
    }
    
    #[test]
    fn test_rational_creation() {
        let half = FixedPoint::from_rational(1, 2).unwrap();
        let quarter = FixedPoint::from_rational(1, 4).unwrap();
        
        assert_eq!(half + quarter, FixedPoint::from_rational(3, 4).unwrap());
        assert_eq!(half * quarter, FixedPoint::from_rational(1, 8).unwrap());
    }
    
    #[test]
    fn test_percentage() {
        let value = FixedPoint::from_int(100);
        let fifty_percent = FixedPoint::from_percentage(50);
        
        assert_eq!(value.percentage(fifty_percent), FixedPoint::from_int(50));
    }
    
    #[test]
    fn test_display() {
        let value = FixedPoint::from_rational(355, 113).unwrap(); // Approximation of π
        let display = format!("{}", value);
        assert!(display.starts_with("3.141592"));
    }
    
    #[test]
    fn test_zero_division() {
        let a = FixedPoint::from_int(5);
        let zero = FixedPoint::ZERO;
        
        assert_eq!(a.checked_div(zero), None);
        assert_eq!(a / zero, FixedPoint::ZERO); // Saturating behavior
    }
    
    #[test]
    fn test_ssz_serialization() {
        let value = FixedPoint::from_rational(22, 7).unwrap();
        let bytes = value.as_ssz_bytes();
        let decoded = FixedPoint::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(value, decoded);
    }
} 