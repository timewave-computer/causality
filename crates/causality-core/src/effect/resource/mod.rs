// Resource Effects Module
//
// This module provides the effect-based interface for working with resources.

mod resource;
pub mod integration;
mod utils;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod standalone_tests;

pub use resource::*;
pub use integration::*;
pub use utils::*; 