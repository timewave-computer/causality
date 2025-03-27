// Cross-Domain Resource Protocol Module
//
// This module provides interfaces and implementations for secure cross-domain
// resource references and transfer mechanisms between domains.

mod protocol;

pub use protocol::{
    CrossDomainResourceId,
    ResourceProjectionType,
    VerificationLevel,
    ResourceReference,
    VerificationResult,
    TransferStatus,
    ResourceTransferOperation,
    CrossDomainProtocolError,
    CrossDomainProtocolResult,
    CrossDomainResourceProtocol,
    DomainResourceAdapter,
    BasicCrossDomainResourceProtocol,
    create_cross_domain_protocol,
}; 