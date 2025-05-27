//! Compatibility module for standard library types across std and no_std environments
//!
//! This module re-exports key types from either std or alloc based on environment.

#[cfg(feature = "std")]
pub use std::{io::Error, iter::Peekable, slice::Iter, str::Chars};

#[cfg(not(feature = "std"))]
pub use alloc::{
    fmt::{self, Display, Formatter},
    result::Result,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(feature = "std"))]
pub use core::{
    iter::{Cloned, Enumerate, Map, Peekable},
    slice::Iter,
    str::{CharIndices, Chars, FromStr},
};

// Define an Error type for no_std
#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub struct Error {
    message: String,
}

#[cfg(not(feature = "std"))]
impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            message: message.into(),
        }
    }
}

#[cfg(not(feature = "std"))]
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
