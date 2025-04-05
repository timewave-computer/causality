use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Instant;

use crate::crypto_primitives::ContentAddressed;
use crate::verification::error::{VerificationError, VerificationResult};
use crate::verification::registry::VerificationRegistry;

/// Trust boundary for verification
#[derive(Debug, Clone)]
pub enum TrustBoundary {
    /// System boundary (within the application)
    System,
    
    /// Domain boundary (within a specific trust domain)
    Domain,
    
    /// Cross-domain boundary (between two trust domains)
    CrossDomain {
        /// Source domain
        source: String,
        /// Target domain
        target: String,
    },
    
    /// User boundary (between the system and a user)
    User,
    
    /// Network boundary (between the system and a network)
    Network {
        /// Network type
        network_type: String,
    },
    
    /// Custom boundary
    Custom(String),
}

impl Display for TrustBoundary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustBoundary::System => write!(f, "System"),
            TrustBoundary::Domain => write!(f, "Domain"),
            TrustBoundary::CrossDomain { source, target } => {
                write!(f, "CrossDomain({}->{})", source, target)
            }
            TrustBoundary::User => write!(f, "User"),
            TrustBoundary::Network { network_type } => {
                write!(f, "Network({})", network_type)
            }
            TrustBoundary::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// A verification point for a content-addressed object
pub struct VerificationPoint<'a, T: ContentAddressed> {
    /// The object to verify
    object: &'a T,
    
    /// The trust boundary
    trust_boundary: TrustBoundary,
    
    /// The verification registry
    registry: Arc<VerificationRegistry>,
}

impl<'a, T: ContentAddressed> VerificationPoint<'a, T> {
    /// Creates a new verification point
    pub fn new(
        object: &'a T,
        trust_boundary: TrustBoundary,
        registry: Arc<VerificationRegistry>,
    ) -> Self {
        Self {
            object,
            trust_boundary,
            registry,
        }
    }
    
    /// Verifies the object
    pub fn verify(&self) -> Result<VerificationResult, VerificationError> {
        let start_time = Instant::now();
        
        // Perform the verification using the registry
        let result = self.registry.verify(self.object, self.trust_boundary.clone())?;
        
        // Log the verification result
        let duration = start_time.elapsed();
        match &result {
            VerificationResult::Verified => {
                tracing::debug!(
                    boundary = %self.trust_boundary,
                    duration_ms = %duration.as_millis(),
                    "Verification succeeded"
                );
            }
            VerificationResult::Failed { reason } => {
                tracing::warn!(
                    boundary = %self.trust_boundary,
                    duration_ms = %duration.as_millis(),
                    reason = %reason,
                    "Verification failed"
                );
            }
        }
        
        Ok(result)
    }
    
    /// Gets a reference to the object
    pub fn object(&self) -> &T {
        self.object
    }
    
    /// Gets the trust boundary
    pub fn trust_boundary(&self) -> &TrustBoundary {
        &self.trust_boundary
    }
}

/// Extension trait for content-addressed types to support verification
pub trait Verifiable: ContentAddressed + Sized {
    /// Creates a verification point for this object
    fn for_verification<'a>(
        &'a self,
        trust_boundary: TrustBoundary,
        registry: Arc<VerificationRegistry>,
    ) -> VerificationPoint<'a, Self> {
        VerificationPoint::new(self, trust_boundary, registry)
    }
    
    /// Verifies this object at a system boundary
    fn verify_at_system<'a>(&'a self, registry: Arc<VerificationRegistry>) -> VerificationPoint<'a, Self> {
        self.for_verification(TrustBoundary::System, registry)
    }
    
    /// Verifies this object at a domain boundary
    fn verify_at_domain<'a>(&'a self, registry: Arc<VerificationRegistry>) -> VerificationPoint<'a, Self> {
        self.for_verification(TrustBoundary::Domain, registry)
    }
    
    /// Verifies this object at a user boundary
    fn verify_at_user<'a>(&'a self, registry: Arc<VerificationRegistry>) -> VerificationPoint<'a, Self> {
        self.for_verification(TrustBoundary::User, registry)
    }
    
    /// Verifies this object at a network boundary
    fn verify_at_network<'a>(
        &'a self,
        network_type: &str,
        registry: Arc<VerificationRegistry>,
    ) -> VerificationPoint<'a, Self> {
        self.for_verification(
            TrustBoundary::Network {
                network_type: network_type.to_string(),
            },
            registry,
        )
    }
    
    /// Verifies this object at a cross-domain boundary
    fn verify_cross_domain<'a>(
        &'a self,
        source: &str,
        target: &str,
        registry: Arc<VerificationRegistry>,
    ) -> VerificationPoint<'a, Self> {
        self.for_verification(
            TrustBoundary::CrossDomain {
                source: source.to_string(),
                target: target.to_string(),
            },
            registry,
        )
    }
}

// Implement for all types that implement ContentAddressed
impl<T: ContentAddressed> Verifiable for T {} 