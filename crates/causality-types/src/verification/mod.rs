// Content Verification Module
//
// This module provides utilities for verifying content-addressed objects
// across trust boundaries and domains.

// Re-export the main verification types
pub use self::error::{VerificationError, VerificationResult};
pub use self::metrics::{VerificationMetric, VerificationMetricsCollector};
pub use self::registry::VerificationRegistry;
pub use self::trust::{TrustBoundary, VerificationPoint, Verifiable};

// Module structure
mod error;
mod metrics;
mod registry;
mod trust;

// Cross-domain verification module
pub mod cross_domain;
pub use self::cross_domain::*; 