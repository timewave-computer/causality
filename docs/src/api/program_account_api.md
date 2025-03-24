# Program Account API Reference

## Overview

The Program Account API provides the user-facing layer for interacting with the Causality system. Program accounts serve as the primary touchpoints for users, exposing resources and available effects in a unified interface. This document outlines the core interfaces, types, and usage patterns of the Program Account API.

## Core Interfaces

### ProgramAccount

The base interface implemented by all program accounts.

```rust
trait ProgramAccount {
    // Identity
    fn id(&self) -> &AccountId;
    fn name(&self) -> &str;
    fn account_type(&self) -> AccountType;
    
    // Resources
    fn resources(&self) -> Vec<ProgramAccountResource>;
    fn get_resource(&self, id: &str) -> Option<ProgramAccountResource>;
    
    // Capabilities
    fn capabilities(&self) -> Vec<ProgramAccountCapability>;
    fn get_capability(&self, id: &CapabilityId) -> Option<ProgramAccountCapability>;
    
    // Effects
    fn available_effects(&self) -> Vec<AvailableEffect>;
    fn execute_effect(&self, effect_id: &str, params: HashMap<String, EffectParameter>) -> Result<EffectResult>;
    
    // Transactions
    fn transactions(&self) -> Vec<TransactionRecord>;
    fn get_transaction(&self, tx_id: &str) -> Option<TransactionRecord>;
}
```

### AssetProgramAccount

Extension of ProgramAccount for accounts that hold assets (tokens, NFTs).

```rust
trait AssetProgramAccount: ProgramAccount {
    fn balance(&self, asset_id: &str) -> Option<AssetBalance>;
    fn balances(&self) -> Vec<AssetBalance>;
    fn transfer(&self, to: &AccountId, asset_id: &str, amount: u64) -> Result<TransactionRecord>;
    fn collections(&self) -> Vec<AssetCollection>;
}
```

### UtilityProgramAccount

Extension of ProgramAccount for utility functions.

```rust
trait UtilityProgramAccount: ProgramAccount {
    fn store_data(&self, key: &str, data: Vec<u8>) -> Result<()>;
    fn get_data(&self, key: &str) -> Option<Vec<u8>>;
    fn compute_hash(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn verify_signature(&self, data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool>;
}
```

### DomainBridgeProgramAccount

Extension of ProgramAccount for cross-domain operations.

```rust
trait DomainBridgeProgramAccount: ProgramAccount {
    fn supported_domains(&self) -> Vec<DomainInfo>;
    fn transfer_to_domain(&self, domain_id: &str, target_address: &str, asset_id: &str, amount: u64) -> Result<CrossDomainTransfer>;
    fn pending_transfers(&self) -> Vec<CrossDomainTransfer>;
    fn transfer_status(&self, transfer_id: &str) -> Option<TransferStatus>;
}
```

## Authorization

The Program Account API includes a robust authorization system to control access to account resources and operations.

### Key Components

- **AuthorizationManager**: Manages authorization for all accounts
- **AuthorizationContext**: Encapsulates the context of an authorization request
- **Role**: Represents a named set of permissions (Admin, User, Guest, etc.)
- **AuthorizationLevel**: Defines the level of access (None, Read, Write, Full)

```rust
struct AuthorizationContext {
    account_id: AccountId,
    requester: Address,
    operation: String,
    resource_id: Option<String>,
    params: HashMap<String, String>,
}

enum AuthorizationLevel {
    None,
    Read,
    Write,
    Full,
}

enum Role {
    Owner,
    Admin,
    User,
    Guest,
    Custom(String),
}
```

### Usage Example

```rust
// Create an authorization context
let context = AuthorizationContext::new(
    account.id().clone(),
    requester_address,
    "transfer",
    Some(asset_id.to_string()),
    params,
);

// Check authorization
match auth_manager.authorize(&context) {
    AuthorizationResult::Authorized => {
        // Proceed with operation
        account.transfer(to_account, asset_id, amount)
    },
    AuthorizationResult::Denied(reason) => {
        // Handle denial
        Err(format!("Authorization denied: {}", reason))
    },
}
```

## Effect System Integration

Program accounts use the Effect System to execute operations in a capability-based manner.

### Key Components

- **AvailableEffect**: Describes an effect that can be executed by the account
- **EffectParameter**: Represents a parameter for an effect
- **EffectResult**: The result of executing an effect
- **ProgramAccountEffectAdapter**: Bridges program accounts with the effect system

```rust
struct AvailableEffect {
    id: String,
    name: String,
    description: String,
    parameters: Vec<EffectParameterDesc>,
    required_capabilities: Vec<CapabilityId>,
}

enum EffectParameter {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Bytes(Vec<u8>),
    Address(Address),
    Array(Vec<EffectParameter>),
    Map(HashMap<String, EffectParameter>),
}

struct EffectResult {
    status: EffectStatus,
    output: Option<EffectParameter>,
    transaction_id: Option<String>,
}
```

### Usage Example

```rust
// Get available effects
let effects = account.available_effects();

// Prepare parameters
let mut params = HashMap::new();
params.insert("recipient".to_string(), EffectParameter::Address(recipient));
params.insert("amount".to_string(), EffectParameter::Number(100));

// Execute effect
let result = account.execute_effect("transfer", params)?;

// Check result
match result.status {
    EffectStatus::Success => {
        println!("Transfer successful: {}", result.transaction_id.unwrap());
    },
    EffectStatus::Pending => {
        println!("Transfer pending: {}", result.transaction_id.unwrap());
    },
    EffectStatus::Failed(reason) => {
        println!("Transfer failed: {}", reason);
    },
}
```

## Resource and Capability Model

Program accounts provide access to resources through capabilities, ensuring secure and controlled access.

### Key Components

- **ProgramAccountResource**: Represents a resource accessible through a program account
- **ProgramAccountCapability**: Represents a capability to access and operate on resources
- **Right**: Specific rights granted by a capability (Read, Write, Execute, etc.)
- **Restrictions**: Constraints on the use of capabilities

### Usage Example

```rust
// Get resources
let resources = account.resources();

// Find a specific resource
if let Some(resource) = account.get_resource("my_token") {
    println!("Resource: {} ({})", resource.name(), resource.id());
    
    // Get capabilities for this resource
    let capabilities = account.capabilities()
        .iter()
        .filter(|cap| cap.resource_id() == resource.id())
        .collect::<Vec<_>>();
        
    for cap in capabilities {
        println!("Capability: {:?} with rights: {:?}", cap.id(), cap.rights());
    }
}
```

## UI Integration

The Program Account API includes UI representation to facilitate building user interfaces.

### Key Components

- **ProgramAccountView**: UI representation of a program account
- **ResourceView**: UI representation of a resource
- **CapabilityView**: UI representation of a capability
- **ViewTransformer**: Converts program account objects to their UI representations

### Usage Example

```rust
// Create a view transformer
let transformer = ProgramAccountViewTransformer::new();

// Transform account to UI view
let account_view = transformer.transform_account(account);

// Serialize to JSON
let json = to_json(&account_view)?;

// Use in UI
send_to_ui(json);
```

## Registry System

The Program Account Registry manages available accounts and their lifecycle.

```rust
trait ProgramAccountRegistry {
    fn register_account(&mut self, account: Box<dyn ProgramAccount>) -> Result<()>;
    fn unregister_account(&mut self, id: &AccountId) -> Result<()>;
    fn get_account(&self, id: &AccountId) -> Option<&dyn ProgramAccount>;
    fn get_accounts(&self) -> Vec<&dyn ProgramAccount>;
    fn get_accounts_by_type(&self, account_type: AccountType) -> Vec<&dyn ProgramAccount>;
}
```

## Transaction Records

Program accounts maintain a record of transactions for auditing and history.

```rust
struct TransactionRecord {
    id: String,
    timestamp: SystemTime,
    operation: String,
    status: TransactionStatus,
    params: HashMap<String, String>,
    result: Option<String>,
    related_resources: Vec<String>,
    fees: Option<TransactionFee>,
}

enum TransactionStatus {
    Pending,
    Completed,
    Failed(String),
    Reverted(String),
}
```

## Best Practices

1. **Always check authorization** before executing operations
2. **Use capabilities** rather than direct resource access
3. **Handle effect results properly**, especially pending or failed states
4. **Validate input parameters** before passing to effects
5. **Keep UI views synchronized** with account state changes

## Error Handling

Program account operations can fail for various reasons. Always use proper error handling:

```rust
match account.execute_effect("transfer", params) {
    Ok(result) => {
        match result.status {
            EffectStatus::Success => { /* handle success */ },
            EffectStatus::Pending => { /* handle pending */ },
            EffectStatus::Failed(reason) => { /* handle failure */ },
        }
    },
    Err(e) => {
        // Handle critical errors (authorization, missing capabilities, etc.)
        println!("Error executing effect: {}", e);
    }
}
``` 