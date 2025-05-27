// Purpose: Defines common traits for TEL components.

use crate::primitive::ids::DomainId;

/// Trait for TEL components that have an associated DomainId.
pub trait HasDomainId {
    fn domain_id(&self) -> DomainId;
}
