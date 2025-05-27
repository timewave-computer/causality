//! Numeric type definitions for the Causality framework.
//!
//! This module defines numeric types used throughout the system.

use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError, DecodeWithLength};
use dashu::{Integer, Rational};
use dashu::integer::UBig;
use dashu::base::Signed;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Deterministic numeric type that supports arbitrary precision arithmetic
/// Uses rational numbers internally to avoid floating point non-determinism
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Number {
    /// Arbitrary precision integer
    Integer(i64),
    /// Arbitrary precision rational number (for decimal representation)
    Decimal(Rational),
}

impl Number {
    /// Create a new integer number
    pub fn new_integer(value: i64) -> Self {
        Number::Integer(value)
    }
    
    /// Create a new decimal number from integer parts
    /// precision: number of decimal places
    pub fn new_decimal(numerator: i64, denominator: u64) -> Self {
        let rational = Rational::from_parts(
            Integer::from(numerator),
            dashu::integer::UBig::from(denominator)
        );
        Number::Decimal(rational)
    }
    
    /// Create a decimal from a string representation like "123.456"
    pub fn from_decimal_str(s: &str) -> Result<Self, String> {
        if let Some(dot_pos) = s.find('.') {
            let integer_part = &s[..dot_pos];
            let fractional_part = &s[dot_pos + 1..];
            
            let integer_val: i64 = integer_part.parse()
                .map_err(|_| format!("Invalid integer part: {}", integer_part))?;
            
            let fractional_digits = fractional_part.len();
            let fractional_val: u64 = fractional_part.parse()
                .map_err(|_| format!("Invalid fractional part: {}", fractional_part))?;
            
            let denominator = 10_u64.pow(fractional_digits as u32);
            let numerator = integer_val * (denominator as i64) + (fractional_val as i64);
            
            Ok(Number::new_decimal(numerator, denominator))
        } else {
            let integer_val: i64 = s.parse()
                .map_err(|_| format!("Invalid integer: {}", s))?;
            Ok(Number::new_integer(integer_val))
        }
    }
    
    /// Convert to integer, truncating any decimal part
    pub fn to_integer(&self) -> i64 {
        match self {
            Number::Integer(i) => *i,
            Number::Decimal(r) => {
                // Convert rational to integer by truncating
                let integer_part = r.clone().trunc();
                // Convert to i64 safely
                integer_part.to_string().parse::<i64>().unwrap_or(0)
            }
        }
    }
    
    /// Check if this number is zero
    pub fn is_zero(&self) -> bool {
        match self {
            Number::Integer(i) => *i == 0,
            Number::Decimal(r) => r.is_zero(),
        }
    }
    
    /// Check if this number is positive
    pub fn is_positive(&self) -> bool {
        match self {
            Number::Integer(i) => *i > 0,
            Number::Decimal(r) => r.is_positive(),
        }
    }
    
    /// Convert to i64 if possible, returning None for decimals that don't fit
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Number::Integer(i) => Some(*i),
            Number::Decimal(r) => {
                // Only return Some if the decimal is actually an integer
                if r.fract().is_zero() {
                    let integer_part = r.clone().trunc();
                    integer_part.to_string().parse::<i64>().ok()
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(i) => write!(f, "{}", i),
            Number::Decimal(r) => {
                // Format rational as decimal string
                write!(f, "{}", r)
            }
        }
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Number::Integer(i) => {
                0u8.hash(state); // Tag for integer
                i.hash(state);
            },
            Number::Decimal(r) => {
                1u8.hash(state); // Tag for decimal
                // Hash the string representation for deterministic hashing
                r.to_string().hash(state);
            }
        }
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number::Integer(value)
    }
}

// Arithmetic operations
impl std::ops::Add for Number {
    type Output = Number;
    
    fn add(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(a), Number::Integer(b)) => Number::Integer(a + b),
            (Number::Integer(a), Number::Decimal(b)) => {
                let a_rational = Rational::from(Integer::from(a));
                Number::Decimal(a_rational + b)
            },
            (Number::Decimal(a), Number::Integer(b)) => {
                let b_rational = Rational::from(Integer::from(b));
                Number::Decimal(a + b_rational)
            },
            (Number::Decimal(a), Number::Decimal(b)) => Number::Decimal(a + b),
        }
    }
}

impl std::ops::Sub for Number {
    type Output = Number;
    
    fn sub(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(a), Number::Integer(b)) => Number::Integer(a - b),
            (Number::Integer(a), Number::Decimal(b)) => {
                let a_rational = Rational::from(Integer::from(a));
                Number::Decimal(a_rational - b)
            },
            (Number::Decimal(a), Number::Integer(b)) => {
                let b_rational = Rational::from(Integer::from(b));
                Number::Decimal(a - b_rational)
            },
            (Number::Decimal(a), Number::Decimal(b)) => Number::Decimal(a - b),
        }
    }
}

impl std::ops::Mul for Number {
    type Output = Number;
    
    fn mul(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(a), Number::Integer(b)) => Number::Integer(a * b),
            (Number::Integer(a), Number::Decimal(b)) => {
                let a_rational = Rational::from(Integer::from(a));
                Number::Decimal(a_rational * b)
            },
            (Number::Decimal(a), Number::Integer(b)) => {
                let b_rational = Rational::from(Integer::from(b));
                Number::Decimal(a * b_rational)
            },
            (Number::Decimal(a), Number::Decimal(b)) => Number::Decimal(a * b),
        }
    }
}

impl std::ops::Div for Number {
    type Output = Number;
    
    fn div(self, other: Number) -> Number {
        match (self, other) {
            (Number::Integer(a), Number::Integer(b)) => {
                if b == 0 {
                    panic!("Division by zero");
                }
                // Convert to rational for exact division
                let a_rational = Rational::from(Integer::from(a));
                let b_rational = Rational::from(Integer::from(b));
                Number::Decimal(a_rational / b_rational)
            },
            (Number::Integer(a), Number::Decimal(b)) => {
                if b.is_zero() {
                    panic!("Division by zero");
                }
                let a_rational = Rational::from(Integer::from(a));
                Number::Decimal(a_rational / b)
            },
            (Number::Decimal(a), Number::Integer(b)) => {
                if b == 0 {
                    panic!("Division by zero");
                }
                let b_rational = Rational::from(Integer::from(b));
                Number::Decimal(a / b_rational)
            },
            (Number::Decimal(a), Number::Decimal(b)) => {
                if b.is_zero() {
                    panic!("Division by zero");
                }
                Number::Decimal(a / b)
            },
        }
    }
}

impl Encode for Number {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            Number::Integer(i) => {
                let mut bytes = vec![0u8]; // Tag for integer
                bytes.extend_from_slice(&i.to_le_bytes());
                bytes
            },
            Number::Decimal(r) => {
                let mut bytes = vec![1u8]; // Tag for decimal
                // Serialize rational as numerator/denominator pair
                let num_bytes = r.numerator().to_string().as_bytes().to_vec();
                let den_bytes = r.denominator().to_string().as_bytes().to_vec();
                
                bytes.extend_from_slice(&(num_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(&num_bytes);
                bytes.extend_from_slice(&(den_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(&den_bytes);
                bytes
            }
        }
    }
}

impl Decode for Number {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Invalid Number: empty bytes".to_string(),
            });
        }
        
        match bytes[0] {
            0 => {
                // Integer
                if bytes.len() != 9 {
                    return Err(DecodeError {
                        message: format!("Invalid Integer length {}, expected 9", bytes.len()),
                    });
                }
                let mut int_bytes = [0u8; 8];
                int_bytes.copy_from_slice(&bytes[1..9]);
                let value = i64::from_le_bytes(int_bytes);
                Ok(Number::Integer(value))
            },
            1 => {
                // Decimal (rational)
                if bytes.len() < 9 {
                    return Err(DecodeError {
                        message: "Invalid Decimal: insufficient length".to_string(),
                    });
                }
                
                let mut pos = 1;
                let num_len = u32::from_le_bytes([
                    bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]
                ]) as usize;
                pos += 4;
                
                if pos + num_len > bytes.len() {
                    return Err(DecodeError {
                        message: "Invalid Decimal: numerator length exceeds data".to_string(),
                    });
                }
                
                let num_str = String::from_utf8(bytes[pos..pos+num_len].to_vec())
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: numerator not valid UTF-8".to_string(),
                    })?;
                pos += num_len;
                
                if pos + 4 > bytes.len() {
                    return Err(DecodeError {
                        message: "Invalid Decimal: insufficient data for denominator length".to_string(),
                    });
                }
                
                let den_len = u32::from_le_bytes([
                    bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]
                ]) as usize;
                pos += 4;
                
                if pos + den_len != bytes.len() {
                    return Err(DecodeError {
                        message: "Invalid Decimal: denominator length mismatch".to_string(),
                    });
                }
                
                let den_str = String::from_utf8(bytes[pos..pos+den_len].to_vec())
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: denominator not valid UTF-8".to_string(),
                    })?;
                
                let numerator = Integer::from_str_radix(&num_str, 10)
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: numerator not valid integer".to_string(),
                    })?;
                let denominator = UBig::from_str_radix(&den_str, 10)
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: denominator not valid integer".to_string(),
                    })?;
                
                let rational = Rational::from_parts(numerator, denominator);
                Ok(Number::Decimal(rational))
            },
            _ => Err(DecodeError {
                message: format!("Invalid Number tag: {}", bytes[0]),
            }),
        }
    }
}

impl SimpleSerialize for Number {}

impl DecodeWithLength for Number {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Invalid Number: empty bytes".to_string(),
            });
        }
        
        match bytes[0] {
            0 => {
                // Integer
                if bytes.len() < 9 {
                    return Err(DecodeError {
                        message: format!("Invalid Integer length {}, expected at least 9", bytes.len()),
                    });
                }
                let mut int_bytes = [0u8; 8];
                int_bytes.copy_from_slice(&bytes[1..9]);
                let value = i64::from_le_bytes(int_bytes);
                Ok((Number::Integer(value), 9))
            },
            1 => {
                // Decimal (rational)
                if bytes.len() < 9 {
                    return Err(DecodeError {
                        message: "Invalid Decimal: insufficient length".to_string(),
                    });
                }
                
                let mut pos = 1;
                let num_len = u32::from_le_bytes([
                    bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]
                ]) as usize;
                pos += 4;
                
                if pos + num_len > bytes.len() {
                    return Err(DecodeError {
                        message: "Invalid Decimal: numerator length exceeds data".to_string(),
                    });
                }
                
                let num_str = String::from_utf8(bytes[pos..pos+num_len].to_vec())
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: numerator not valid UTF-8".to_string(),
                    })?;
                pos += num_len;
                
                if pos + 4 > bytes.len() {
                    return Err(DecodeError {
                        message: "Invalid Decimal: insufficient data for denominator length".to_string(),
                    });
                }
                
                let den_len = u32::from_le_bytes([
                    bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]
                ]) as usize;
                pos += 4;
                
                if pos + den_len > bytes.len() {
                    return Err(DecodeError {
                        message: "Invalid Decimal: denominator length exceeds data".to_string(),
                    });
                }
                
                let den_str = String::from_utf8(bytes[pos..pos+den_len].to_vec())
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: denominator not valid UTF-8".to_string(),
                    })?;
                
                let numerator = Integer::from_str_radix(&num_str, 10)
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: numerator not valid integer".to_string(),
                    })?;
                let denominator = UBig::from_str_radix(&den_str, 10)
                    .map_err(|_| DecodeError {
                        message: "Invalid Decimal: denominator not valid integer".to_string(),
                    })?;
                
                let rational = Rational::from_parts(numerator, denominator);
                let total_consumed = pos + den_len;
                Ok((Number::Decimal(rational), total_consumed))
            },
            _ => Err(DecodeError {
                message: format!("Invalid Number tag: {}", bytes[0]),
            }),
        }
    }
}
