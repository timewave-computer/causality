# ADR-002: Effect Adapters


## Status

Accepted

## Implementation Status

This ADR has been partially implemented with a shift in approach from the original schema-driven generation model to a more traditional factory-based approach. Key components implemented include:

1. **Domain Adapters for Multiple Chains**:
   - Implemented EVM adapter for Ethereum and EVM-compatible chains
   - Implemented CosmWasm adapter for Cosmos ecosystem
   - Added support for Succinct adapter for ZK proofs
   - Created interfaces for cross-domain operations

2. **Factory Pattern**:
   - Implemented the `DomainAdapterFactory` trait for creating adapters
   - Implemented concrete factories like `EthereumAdapterFactory`
   - Added configuration-based adapter creation

3. **Effect Handler Integration**:
   - Integrated adapters with the effect system through interfaces
   - Created boundary crossing mechanisms for secure domain interactions
   - Added support for effect execution across domains

4. **Proof Management**:
   - Implemented proof validation for transactions
   - Added fact observation and verification
   - Created time map integration for temporal consistency

5. **Storage Strategies**:
   - Implemented storage effect factories for each domain
   - Created commitment-based and fully on-chain storage options
   - Added ZK integration for storage proofs

While the core principles of domain adapter isolation and standardized interfaces have been implemented, the implementation diverged from the original schema-driven code generation approach. The adapter-generator directory exists but is in an early state, with the system currently using manually coded adapters with factory patterns instead of generated adapters.

Recent development indicates a move toward further simplification and refactoring of this system, as indicated by the latest commit message: "code cleanup from refactor, move hashing into traits, depreciate effect_adapters".

The implementation has evolved to align with ADR-023's three-layer effect architecture, which provides a more comprehensive model for effect handling that includes domain adapters as part of a unified approach.


## Context

Causality programs interact with **multiple external chains** (referred to as "Domains" in system terminology), each with different:

- **APIs** (transaction formats, signing schemes, RPC models).
- **Proof formats** (inclusion proofs, state proofs, receipt proofs).
- **Asset models** (UTXO vs account-based).
- **Serialization formats** (RLP, Borsh, protobuf, etc.).

Initially, these interactions were handled by manually coded adapters for each chain, which was:
- Error-prone.
- Difficult to maintain.
- A security risk if inconsistencies crept into proof handling.
- A major barrier to adding support for new Domains.

As the system evolved, **effect adapters** became the **standard boundary between programs and external chains**. These adapters **mediate all external effects**, ensuring that:

- External data is correctly observed and proven (fact creation).
- External calls (deposits, withdrawals) correctly map to chain transactions.
- External facts and events become **observed facts** with valid proofs.


## Decision

### Core Principle: Generated, Schema-Driven Adapters

Each supported Domain will have a **generated effect adapter** responsible for:

- **Encoding outgoing effects into Domain-specific transactions.**  
- **Validating incoming proofs and facts against the external Domain.**  
- **Converting external facts into canonical Causality facts.**  
- **Preserving external time observations into time map snapshots.**


## Adapter Scope

| Adapter Scope | Description |
|---|---|
| Per Domain | Each adapter only handles one Domain (e.g., Ethereum, Solana). |
| Effect Mapping | Each adapter implements only effects relevant to that Domain. |
| Proof Handling | Each adapter validates Domain-specific proofs. |
| Time Map Observation | Each adapter records the Domain observation (block height, hash, etc.) at the time of effect application. |


## Changes in Effect Flow

1. **Account Programs as Mediators**  
    - All external deposits, withdrawals, transfers pass through account programs.
    - Account programs **delegate Domain-specific handling to adapters**.

2. **Fact Observation through Adapters**  
    - Committees no longer encode all fact logic directly.
    - Committees **invoke adapters to transform Domain data into canonical facts**.

3. **Proof Embedding**  
    - Each effect recorded in the unified log will include:
        - The effect payload itself.
        - The **Domain proof** (Domain-specific).
        - The **time map snapshot** (external observations).

4. **Replay Requires Adapters**  
    - Replay engines will load the appropriate adapter to verify effects during replay.
    - This keeps replay deterministic even if external formats evolve.


## Adapter Generation Process

Adapters will be **generated from schemas** that define:

| Field | Description |
|---|---|
| Domain ID | Which Domain the adapter applies to. |
| Effect Mappings | Supported effects (deposit, withdraw, transfer, etc.). |
| Proof Formats | How to validate proofs for each effect type. |
| RPC Interfaces | External API bindings for submitting transactions and querying state. |
| Serialization Rules | How to encode/decode effect payloads and proofs. |
| Register Interface | How register operations are encoded and validated. |
| ZK Circuit Types | The types of circuits used for register operations. |

This ensures:

- Adapters are consistent across Domains.  
- New Domains can be added quickly by providing schemas.  
- Security is centralized into well-defined, auditable rules.  


## Example Effect Adapter Schema (TOML)

```toml
Domain = "Ethereum"

[[effects]]
type = "Deposit"
tx_format = "RLP"
proof_format = "MPT"
rpc_call = "eth_getTransactionReceipt"
required_fields = ["amount", "asset", "destination"]

[[effects]]
type = "Withdraw"
tx_format = "RLP"
proof_format = "MPT"
rpc_call = "eth_call"
required_fields = ["amount", "asset", "source"]

[proofs]
inclusion = "eth_getProof"
receipt = "eth_getTransactionReceipt"
```


## Account Program Integration

Account programs handle:

- Maintaining per-Domain balances.
- Applying all external effects (deposits, withdrawals, transfers) by **invoking adapters**.
- Enforcing User-defined policy (rate limits, multi-sig).
- Linking all external interactions to **time map observations**.

This ensures:

- All User-visible actions are recorded in their account program log.
- All cross-program resource flows are causally linked to external facts.
- All external dependencies are cryptographically proven.


## Time Map Enforcement

- Every external effect **records the external Domain state (time map snapshot)** at the time the effect applies.
- This snapshot is hashed and linked into the effect DAG.
- This guarantees that every replay will see **exactly the same external observations** as the original run.


## Generated Adapter Interface

Each generated adapter will expose:

```haskell
class EffectAdapter where
    applyEffect :: Effect -> AccountProgramState -> ExternalDomain -> IO (Either AdapterError Receipt)
    validateProof :: Effect -> ExternalDomain -> IO (Either ProofError ())
    observeFact :: ExternalDomain -> IO (Either ObservationError ObservedFact)
```


## Security Benefits

- Standardizes all Domain-specific proof handling.  
- Ensures all effects are processed via **well-tested, auditable generated code**.  
- Fully decouples **program logic from Domain specifics** — programs only emit abstract effects.  
- Forces all observed facts to pass through the same schema-defined process, ensuring they match replay expectations.  


## Extensibility Benefits

- Adding a new chain requires only a new **adapter schema** and a generated adapter.  
- Simulation can stub in mock adapters, while production uses real adapters.  
- Programs remain **Domain-agnostic**, since they never directly call external RPC.


## Example Lifecycle - Deposit

| Step | Actor | Action |
|---|---|---|
| 1 | User | Submits deposit message to account program |
| 2 | Account Program | Invokes Ethereum Adapter's `applyEffect` |
| 3 | Ethereum Adapter | Builds RLP transaction |
| 4 | Ethereum Adapter | Sends transaction via RPC |
| 5 | Ethereum Adapter | Observes deposit proof |
| 6 | Ethereum Adapter | Packages proof + time map snapshot |
| 7 | Account Program | Records effect + proof in unified log |


## Example Lifecycle - Observed Fact

| Step | Actor | Action |
|---|---|---|
| 1 | Committee | Observes new block on Ethereum |
| 2 | Committee | Invokes Ethereum Adapter's `observeFact` |
| 3 | Ethereum Adapter | Applies extraction rules (price feed, balances, etc.) |
| 4 | Ethereum Adapter | Packages fact + proof + time map snapshot |
| 5 | Committee | Signs and gossips the observed fact |
| 6 | Operators | Receive and store fact in FactLog |


## Replay and Simulation

- During replay, the replay engine loads the relevant adapter.
- The adapter is invoked to re-verify effects and proofs.
- In simulation mode, mock adapters can stub Domain calls (but still generate valid proof structures).


## Summary - What This Changes

| Area | Change |
|---|---|
| Effect Pipeline | External effects always mediated by adapters. |
| Account Programs | Route all external effects through adapters. |
| Fact Observation | Committees invoke adapters to generate facts. |
| Replay | Replay loads adapters to validate proof correctness. |
| Domain Extensibility | New Domains added via schemas, not hardcoded logic. |


## Resource Handling in Generated Adapters

The generated adapters will handle resources according to the resource model:

1. **Resource Serialization**: Convert between program-level resource tuples and chain-specific representations:
   ```rust
   fn serialize_resource(resource: Resource) -> Vec<u8> {
     // Encode resource tuple fields according to chain format
     let mut encoded = Vec::new();
     encoded.extend_from_slice(&resource.logic.serialize());
     encoded.extend_from_slice(&resource.fungibility_domain.serialize());
     encoded.extend_from_slice(&resource.quantity.serialize());
     encoded.extend_from_slice(&resource.metadata.serialize());
     encoded
   }
   ```

2. **Resource Conservation Validation**: Ensure that all operations maintain resource conservation:
   ```rust
   // Generated validation code example
   fn validate_resource_conservation(inputs: &[Resource], outputs: &[Resource]) -> Result<(), ValidationError> {
     // Compute input sum by fungibility domain
     let mut input_sums = HashMap::new();
     for resource in inputs {
       let entry = input_sums.entry(resource.fungibility_domain).or_insert(0);
       *entry += resource.quantity;
     }
     
     // Compute output sum by fungibility domain
     let mut output_sums = HashMap::new();
     for resource in outputs {
       let entry = output_sums.entry(resource.fungibility_domain).or_insert(0);
       *entry += resource.quantity;
     }
     
     // Validate conservation for each domain
     for (domain, input_sum) in input_sums {
       let output_sum = output_sums.get(&domain).unwrap_or(&0);
       if input_sum != *output_sum {
         return Err(ValidationError::ResourceConservationViolation(domain));
       }
     }
     
     Ok(())
   }
   ```

3. **Controller Label Tracking**: Track resource provenance across Domains:
   ```rust
   // Generated controller tracking code
   fn update_controller_label(label: ControllerLabel, new_controller: ControllerId) -> ControllerLabel {
     ControllerLabel {
       terminal_controller: new_controller,
       affecting_controllers: [new_controller].iter()
         .Domain(label.affecting_controllers.iter())
         .cloned()
         .collect(),
       ..label
     }
   }
   ```

4. **Dual Validation Support**: Implement both temporal and ancestral validation:
   ```rust
   // Generated dual validation code
   fn validate_cross_Domain_operation(
     source_Domain: DomainId,
     target_Domain: DomainId,
     resource: Resource,
     time_map: &TimeMap,
     controller_label: &ControllerLabel
   ) -> Result<ValidationResult, ValidationError> {
     // Temporal validation using time map
     let temporal_result = validate_temporal_consistency(source_Domain, target_Domain, time_map)?;
     
     // Ancestral validation using controller labels
     let ancestral_result = validate_controller_path(
       controller_label.creating_controller,
       controller_label.terminal_controller,
       controller_label.affecting_controllers.as_slice(),
     )?;
     
     Ok(ValidationResult {
       temporal: temporal_result,
       ancestral: ancestral_result,
     })
   }
   ```

## Implementation Example

Here's a more detailed example of schema-to-code generation for an Ethereum adapter:

### Input Schema (TOML)

```toml
name = "EthereumAdapter"
version = "1.0"
Domain_id = 1
protocol = "EVM"

[[resources]]
name = "ETH"
fungibility_domain = "ETH"
resource_logic = "TokenLogic"
decimals = 18
conservation_rule = "sum(inputs) == sum(outputs)"

[[resources]]
name = "ERC20"
fungibility_domain = "ERC20/{address}"
resource_logic = "TokenLogic"
conservation_rule = "sum(inputs) == sum(outputs)"

[resources.interface]
standard = "ERC20"
abi = "..."

[controller]
type = "Safe"
controller_id = "ethereum-mainnet"
backup_controllers = []

[[effects]]
name = "Transfer"

[[effects.inputs]]
name = "from"
type = "Address"

[[effects.inputs]]
name = "resource"
type = "Resource"

[[effects.outputs]]
name = "to"
type = "Address"

[[effects.outputs]]
name = "resource"
type = "Resource"

[effects.validation]
resource_conservation = true
signature_validation = true
```

### Generated Adapter Code (Rust)

```rust
// Auto-generated Ethereum adapter
// Generated from schema version 1.0
// DO NOT EDIT MANUALLY

use time_Operators_core::resources::{Resource, ResourceLogic, FungibilityDomain, Quantity};
use time_Operators_core::controllers::{ControllerLabel, ControllerId, ControllerType};
use time_Operators_core::validation::{ValidationResult, TemporalValidation, AncestralValidation};

pub struct EthereumAdapter {
    Domain_id: u64,
    controller_id: ControllerId,
    controller_type: ControllerType,
}

impl EthereumAdapter {
    pub fn new() -> Self {
        Self {
            Domain_id: 1,
            controller_id: ControllerId::new("ethereum-mainnet"),
            controller_type: ControllerType::Safe,
        }
    }
    
    pub fn controller_id(&self) -> &ControllerId {
        &self.controller_id
    }
    
    pub fn controller_type(&self) -> ControllerType {
        self.controller_type
    }
    
    pub fn create_controller_label(&self) -> ControllerLabel {
        ControllerLabel {
            creating_controller: self.controller_id.clone(),
            terminal_controller: self.controller_id.clone(),
            affecting_controllers: vec![self.controller_id.clone()],
            backup_controllers: vec![],
        }
    }
    
    // Transfer effect adapter
    pub fn encode_transfer(
        &self,
        from: Address,
        to: Address,
        resource: &Resource,
        controller_label: &ControllerLabel,
    ) -> Result<EncodedTransaction, AdapterError> {
        // Validate resource
        self.validate_resource(resource)?;
        
        // Validate controller label
        self.validate_controller_label(controller_label)?;
        
        // Create transaction based on resource type
        let tx = match resource.fungibility_domain.as_str() {
            "ETH" => {
                // Native ETH transfer
                EncodedTransaction {
                    to: to.clone(),
                    data: vec![],
                    value: resource.quantity,
                }
            },
            domain if domain.starts_with("ERC20/") => {
                // ERC20 transfer
                let token_address = extract_token_address(domain)?;
                let encoded_transfer = encode_erc20_transfer(to, resource.quantity)?;
                
                EncodedTransaction {
                    to: token_address,
                    data: encoded_transfer,
                    value: 0,
                }
            },
            _ => return Err(AdapterError::UnsupportedResourceDomain(
                resource.fungibility_domain.clone()
            )),
        };
        
        Ok(tx)
    }
    
    // Decode observed transfer
    pub fn decode_transfer(
        &self,
        tx_receipt: &TransactionReceipt,
    ) -> Result<(Address, Address, Resource, ControllerLabel), AdapterError> {
        // Extract transaction details
        let (from, to, value, input) = extract_tx_details(tx_receipt)?;
        
        // Determine resource type
        let resource = if input.is_empty() {
            // Native ETH transfer
            Resource {
                resource_logic: ResourceLogic::TokenLogic,
                fungibility_domain: FungibilityDomain::new("ETH"),
                quantity: value,
                metadata: json!({"DomainId": self.Domain_id, "decimals": 18}),
                ephemeral: false,
                nonce: generate_nonce(),
                nullifier_pub_key: derive_nullifier_key(&to),
                randomness_seed: generate_seed(),
            }
        } else if is_erc20_transfer(input) {
            // ERC20 transfer
            let (token_to, token_value) = decode_erc20_transfer(input)?;
            let token_address = tx_receipt.to.clone();
            
            Resource {
                resource_logic: ResourceLogic::TokenLogic,
                fungibility_domain: FungibilityDomain::new(&format!("ERC20/{}", token_address)),
                quantity: token_value,
                metadata: json!({"tokenAddress": token_address, "DomainId": self.Domain_id}),
                ephemeral: false,
                nonce: generate_nonce(),
                nullifier_pub_key: derive_nullifier_key(&token_to),
                randomness_seed: generate_seed(),
            }
        } else {
            return Err(AdapterError::UnsupportedTransaction);
        };
        
        // Create controller label
        let controller_label = self.create_controller_label();
        
        Ok((from, to, resource, controller_label))
    }
    
    // Validate resource conservation for a transaction
    pub fn validate_resource_conservation(
        &self,
        inputs: &[Resource],
        outputs: &[Resource],
    ) -> Result<(), ValidationError> {
        // Group resources by fungibility domain
        let mut input_sums = HashMap::new();
        for resource in inputs {
            let entry = input_sums
                .entry(resource.fungibility_domain.clone())
                .or_insert(0);
            *entry += resource.quantity;
        }
        
        let mut output_sums = HashMap::new();
        for resource in outputs {
            let entry = output_sums
                .entry(resource.fungibility_domain.clone())
                .or_insert(0);
            *entry += resource.quantity;
        }
        
        // Verify conservation for each domain
        for (domain, input_sum) in input_sums {
            let output_sum = output_sums.get(&domain).unwrap_or(&0);
            if input_sum != *output_sum {
                return Err(ValidationError::ResourceConservationViolation{
                    domain,
                    input_sum,
                    output_sum: *output_sum,
                });
            }
        }
        
        Ok(())
    }
    
    // Perform dual validation for cross-domain operations
    pub fn validate_cross_Domain_operation(
        &self,
        source_Domain: &ControllerId,
        target_Domain: &ControllerId,
        resource: &Resource,
        time_map: &TimeMap,
        controller_label: &ControllerLabel,
    ) -> Result<ValidationResult, ValidationError> {
        // Temporal validation
        let temporal_validation = self.validate_temporal_consistency(
            source_Domain,
            target_Domain,
            time_map,
        )?;
        
        // Ancestral validation
        let ancestral_validation = self.validate_controller_path(
            &controller_label.creating_controller,
            &controller_label.terminal_controller,
            &controller_label.affecting_controllers,
        )?;
        
        Ok(ValidationResult {
            temporal: temporal_validation,
            ancestral: ancestral_validation,
        })
    }
}
```

### Generated Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eth_transfer_encoding() {
        let adapter = EthereumAdapter::new();
        let from = Address::from_str("0x1234...").unwrap();
        let to = Address::from_str("0x5678...").unwrap();
        
        let resource = Resource {
            resource_logic: ResourceLogic::TokenLogic,
            fungibility_domain: FungibilityDomain::new("ETH"),
            quantity: 1_000_000_000_000_000_000, // 1 ETH
            metadata: json!({"DomainId": 1, "decimals": 18}),
            ephemeral: false,
            nonce: [0; 32],
            nullifier_pub_key: [0; 32],
            randomness_seed: [0; 32],
        };
        
        let controller_label = adapter.create_controller_label();
        
        let tx = adapter.encode_transfer(
            from,
            to,
            &resource,
            &controller_label,
        ).unwrap();
        
        assert_eq!(tx.to, to);
        assert_eq!(tx.value, 1_000_000_000_000_000_000);
        assert!(tx.data.is_empty());
    }
    
    #[test]
    fn test_resource_conservation_validation() {
        let adapter = EthereumAdapter::new();
        
        // Create input and output resources that balance
        let input_resource = Resource {
            resource_logic: ResourceLogic::TokenLogic,
            fungibility_domain: FungibilityDomain::new("ETH"),
            quantity: 1_000_000_000_000_000_000, // 1 ETH
            metadata: json!({"DomainId": 1, "decimals": 18}),
            ephemeral: false,
            nonce: [0; 32],
            nullifier_pub_key: [0; 32],
            randomness_seed: [0; 32],
        };
        
        let output_resource = Resource {
            resource_logic: ResourceLogic::TokenLogic,
            fungibility_domain: FungibilityDomain::new("ETH"),
            quantity: 1_000_000_000_000_000_000, // 1 ETH
            metadata: json!({"DomainId": 1, "decimals": 18}),
            ephemeral: false,
            nonce: [1; 32], // Different nonce
            nullifier_pub_key: [0; 32],
            randomness_seed: [0; 32],
        };
        
        // This should validate successfully
        let result = adapter.validate_resource_conservation(
            &[input_resource.clone()],
            &[output_resource.clone()],
        );
        assert!(result.is_ok());
        
        // Now create imbalanced resources
        let imbalanced_output = Resource {
            resource_logic: ResourceLogic::TokenLogic,
            fungibility_domain: FungibilityDomain::new("ETH"),
            quantity: 900_000_000_000_000_000, // 0.9 ETH
            metadata: json!({"DomainId": 1, "decimals": 18}),
            ephemeral: false,
            nonce: [1; 32],
            nullifier_pub_key: [0; 32],
            randomness_seed: [0; 32],
        };
        
        // This should fail validation
        let result = adapter.validate_resource_conservation(
            &[input_resource],
            &[imbalanced_output],
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_dual_validation() {
        let adapter = EthereumAdapter::new();
        let ethereum = ControllerId::new("ethereum-mainnet");
        let optimism = ControllerId::new("optimism-mainnet");
        
        let resource = Resource {
            resource_logic: ResourceLogic::TokenLogic,
            fungibility_domain: FungibilityDomain::new("ETH"),
            quantity: 1_000_000_000_000_000_000, // 1 ETH
            metadata: json!({"DomainId": 1, "decimals": 18}),
            ephemeral: false,
            nonce: [0; 32],
            nullifier_pub_key: [0; 32],
            randomness_seed: [0; 32],
        };
        
        let mut controller_label = adapter.create_controller_label();
        // Update for cross-domain transfer to Optimism
        controller_label.terminal_controller = optimism.clone();
        controller_label.affecting_controllers.push(optimism.clone());
        
        // Create time map with Ethereum ahead of Optimism
        let mut time_map = TimeMap::new();
        time_map.insert(ethereum.clone(), 1000);
        time_map.insert(optimism.clone(), 500);
        
        // Should validate successfully
        let result = adapter.validate_cross_Domain_operation(
            &ethereum,
            &optimism,
            &resource,
            &time_map,
            &controller_label,
        );
        assert!(result.is_ok());
        
        // Now create an invalid controller path
        let mut invalid_label = controller_label.clone();
        invalid_label.affecting_controllers = vec![optimism.clone()]; // Missing Ethereum
        
        // Should fail ancestral validation
        let result = adapter.validate_cross_Domain_operation(
            &ethereum,
            &optimism,
            &resource,
            &time_map,
            &invalid_label,
        );
        assert!(result.is_err());
    }
}
```

## Consequences

### Positive Consequences

1. **Improved Correctness**: Generated adapters ensure consistent handling of chain interactions, reducing errors.
2. **Easier Onboarding**: New chains can be supported by creating a schema rather than writing a full adapter.
3. **Better Testing**: Generated adapters include comprehensive test suites.
4. **Resource Safety**: Built-in enforcement of resource conservation laws.
5. **Dual Validation**: Consistent application of both temporal and ancestral validation.
6. **Documentation**: Schemas serve as self-documenting interfaces.

### Negative Consequences

1. **Complexity**: Code generator adds another layer of complexity to the system.
2. **Learning Curve**: Team members must learn schema format and generator details.
3. **Flexibility Tradeoff**: Some custom cases may be harder to express in the schema format.

### Mitigation Strategies

1. **Extensibility**: Allow for custom code to be integrated with generated adapters.
2. **Documentation**: Provide clear documentation on schema format and generation process.
3. **Tooling**: Build tools to validate schemas and preview generated code.
4. **Gradual Adoption**: Start with simpler chains, iterate on the generator.

## References

* [ADR 018: Resource Formalization](adr_018_resource_formalization.md)
* [ADR 006: Resource Ownership](adr_006_resource_ownership.md)
* [JSON Schema](https://json-schema.org/)
* [chain interface standardization efforts](https://github.com/DomainAgnostic/CAIPs)
