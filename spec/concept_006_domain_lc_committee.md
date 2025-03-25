# Committee/Domain Definitions for Causality Architecture

## Committee
A Committee is a set of actors (validators) who collectively participate in a consensus mechanism. This same committee may be materializing multiple domains if there are forks.

**Key characteristics:**
- A collection of consensus participants with defined weights and roles
- Can simultaneously validate multiple competing forks of a chain
- Committee membership and weights evolve over time
- Provides signed attestations about chain states

## Light Client
A Light Client is a deterministic algorithm implementing a specific blockchain's fork choice rule. It processes validator attestations to determine canonical chain tips.

**Key characteristics:**
- Deterministically applies a specific fork choice rule
- Consumes committee attestations to identify canonical chain tips
- May identify multiple valid tips when the chain is forked
- Produces cryptographic proofs of blockchain state

## Domain
A Domain is a specific blockchain fork identified by a unique chain ID, genesis parameters, and a canonical tip. Multiple domains can be materialized by the same committee during a fork.

**Key characteristics:**
- Uniquely identified by DomainID (chain ID + fork identifier)
- Represents a specific consensus history with a single canonical tip
- When a chain forks, each fork becomes a distinct domain
- Multiple domains can share the same committee (temporarily)

## Controller
A Controller is a reference to a specific Domain that has authority over an object. It precisely identifies which fork has control of a resource.

**Key characteristics:**
- Always references a specific Domain (not just a committee)
- Ensures objects have well-defined ownership even during chain forks
- Tracked in ControllerLabels for complete resource provenance
- Used for validating state transitions across domains

## Fork Dynamics

This refinement captures important fork dynamics:

1. **Chain Fork → Multiple Domains**:
   - When a chain forks, it creates multiple domains
   - Initially, the same committee materializes both domains
   - Over time, committees typically diverge as validator sets change
   - Each domain follows its own evolution path

2. **Controller During Forks**:
   - Resources must have a clearly defined controller even during forks
   - ControllerLabels precisely specify which fork (domain) controls a resource
   - Cross-domain transfers become more complex during active forks
   - Dual validation becomes critical for security during forking events

## Implementation Example

```rust
/// A committee of validators
pub struct Committee {
    /// Unique committee identifier
    pub committee_id: CommitteeId,
    /// Committee members with voting weights
    pub members: HashMap<ValidatorId, VotingWeight>,
    /// Signature scheme used
    pub signature_scheme: SignatureScheme,
    /// Domains this committee is currently materializing
    pub materialized_domains: Vec<DomainId>,
}

/// A light client implementation
pub struct LightClient {
    /// Light client identifier
    pub light_client_id: LightClientId,
    /// Fork choice rule implementation
    pub fork_choice_rule: Box<dyn ForkChoiceRule>,
    /// Domain identification logic (how to distinguish forks)
    pub domain_identifier: Box<dyn DomainIdentifier>,
}

/// A domain representing a specific blockchain fork
pub struct Domain {
    /// Domain identifier (must uniquely identify the fork)
    pub domain_id: DomainId,
    /// Committee currently materializing this domain
    pub committee: CommitteeId,
    /// Light client algorithm used
    pub light_client: LightClientId,
    /// Current canonical tip
    pub canonical_tip: BlockHeader,
    /// Fork identifier (how this domain differs from others)
    pub fork_identifier: ForkIdentifier,
}

/// A controller label tracking resource provenance
pub struct ControllerLabel {
    /// Domain that created the resource (specific fork)
    pub creating_controller: DomainId,
    /// Current domain controlling the resource (specific fork)
    pub terminal_controller: DomainId,
    /// History of domains that have controlled this resource
    pub affecting_controllers: Vec<DomainId>,
    /// Backup controllers for recovery
    pub backup_controllers: Vec<DomainId>,
}

/// Handler for fork events in the system
pub struct ForkHandler {
    /// Detect when a fork has occurred
    pub detect_fork: Box<dyn ForkDetector>,
    /// Create new domains when forks occur
    pub create_fork_domains: Box<dyn ForkDomainCreator>,
    /// Update controller labels during forks
    pub update_controller_labels: Box<dyn ControllerLabelUpdater>,
}
```

This refined model accurately captures the complex relationship between committees and domains, particularly during fork events, and ensures precise tracking of resource control across forking events.