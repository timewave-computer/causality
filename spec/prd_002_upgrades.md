# PRD 002: Program and Protocol Upgrades (Updated - 2025-03-07)


## Context and Motivation

Causality is a system for deploying, executing, and proving the execution of **cross-domain programs** — programs that operate across multiple independent chains (Domains). These programs depend on external state (prices, balances, external messages) and must preserve **causal integrity** across all effects, facts, and observations.

With the introduction of the formalized resource model (ADR_018), programs also track formalized resources with explicit tuples, controller labels, and conservation laws. This new model adds complexity to the upgrade process, particularly for programs that manage cross-domain resources.

Over time, both the **Causality protocol itself** (the infrastructure operated by Causality nodes) and **individual programs** (authored and owned by Users) must evolve to support:

- New Domains and effect types.
- New proof formats.
- New cryptographic primitives.
- New user-defined program logic.
- New resource types and controller mechanisms.
- Updates to resource formalization rules.
- Changes in dual validation requirements.

This document defines the requirements, constraints, and design decisions for enabling safe and secure program and protocol upgrades.


## Requirements

### R1 - User Sovereignty
- **Users fully own and control their programs.**
- No upgrade — to either program logic or protocol rules — can be **forced** on a program.
- Users can **opt out** of protocol upgrades and choose to stay pinned to a prior version — as long as they can find Operators willing to run the legacy version.

### R2 - Safe Upgrade Defaults
- The **default path** favors **automatic upgrades** to the latest protocol version.
- If a program is in a **safe state**, upgrades should:
    - Automatically apply schema migrations (where possible).
    - Migrate formalized resources to new schema formats.
    - Update controller labels to comply with new requirements.
    - Migrate to the new version without User intervention.

### R3 - Explicit Safe States for Migration
- Users should optionally declare **safe state strategies** defining when upgrades are permissible.
- Safe states can be:
    - **Always Safe:** Upgrade at any time.
    - **No Pending Returns:** Only upgrade when no cross-program calls are unresolved.
    - **No Cross-domain Resources:** Only upgrade when no cross-domain resources are in transit.
    - **Resource Quiescence:** Only upgrade when all resources are in a stable state (not in process of creation/consumption).
    - **Manual Trigger:** User explicitly signals a safe state.

### R4 - Schema Evolution without Manual Migration
- Schema changes should, whenever possible, be **automatic**.
- Only **semantic shifts** (meaning changes, not format changes) should require User-authored migration code.
- Schema evolution rules should cover:
    - Adding fields.
    - Removing unused fields.
    - Adding default fields.
    - Updating resource formalizations.
    - Migrating controller labels.

### R5 - Resource Conservation During Upgrades
- Resource conservation laws (ΔTX = 0) must be maintained during and after upgrades.
- Upgrades must not create, destroy, or duplicate formalized resources.
- Controller labels must maintain valid provenance information across upgrades.


## Core Concepts

### Program Version Pinning
Every program declares:
```toml
[program]
name = "CrossDomainArb"
version = "1.2.0"
protocolVersion = "2.3.0"
safeStateStrategy = "no-pending-returns"
resourceStrategy = "quiescent-resources"

[history]
previousVersions = ["sha256-oldversion1", "sha256-oldversion2"]

[resourceFormalization]
version = "1.0.0"
controllerLabelVersion = "1.1.0"
```

This allows Operators to validate **which runtime version applies** for every effect and which resource formalization rules to use.


### Epoch-Based Protocol Upgrades
The Causality **protocol version** evolves in discrete **epochs**. All Operators agree on:

```toml
[current]
protocolVersion = "2.3.0"
compatibleProgramVersions = ["1.x", "2.x"]
resourceFormalizationVersion = "1.0.0"

[upcoming]
activationEpoch = 180000
newVersion = "3.0.0"
newResourceFormalizationVersion = "1.1.0"
```

This ensures coordinated, deterministic network upgrades.


### Compatibility Policy

| Program Version | Protocol Version | Resource Formalization | Behavior |
|---|---|---|---|
| Older than compatible range | Any | Rejected until upgraded. |
| Within compatible range | Compatible | Runs normally. |
| Within compatible range | Incompatible | Resource migration required. |
| Newer than Operator version | Any | Rejected (Operators must upgrade). |


### Safe State Declaration
Programs declare their safe state policy at deployment:

| Strategy | Meaning |
|---|---|
| `always` | Safe to upgrade anytime. |
| `no-pending-returns` | Safe only if no pending cross-program calls. |
| `no-cross-domain-resources` | Safe only when no resources are in transit across Domains. |
| `quiescent-resources` | Safe only when all resources are in stable states. |
| `manual` | User explicitly signals safe points. |


### Migration Functions
For **schema changes not covered by automatic evolution rules**, Users provide:

```rust
type MigrationFunction = fn(old_program_state: OldProgramState) -> Result<NewProgramState, MigrationError>;

// For resource-specific migrations
type ResourceMigrationFunction = fn(old_resource: OldResource) -> Result<NewResource, MigrationError>;

// For controller label migrations
type ControllerLabelMigrationFunction = fn(old_controller_label: OldControllerLabel) -> Result<NewControllerLabel, MigrationError>;
```

This is a logged effect in the program's unified log.


## Example Migration Effect

```rust
enum EffectType {
    Deposit {
        resource: Resource,
        amount: Amount,
    },
    Withdraw {
        resource: Resource,
        amount: Amount,
    },
    Transfer {
        resource: Resource,
        amount: Amount,
        recipient: Recipient,
    },
    UpgradeProgram {
        old_hash: ProgramHash,
        new_hash: ProgramHash,
        migration: Option<MigrationFunction>,
        resource_migrations: Option<HashMap<ResourceType, ResourceMigrationFunction>>,
        controller_label_migration: Option<ControllerLabelMigrationFunction>,
    },
}
```

## Autonomous Schema Evolution
By default, Causality applies **schema evolution rules** when upgrading programs.

Example schema evolution:

```toml
[schema]
version = "1.2.0"
fields = ["balances", "riskTolerance"]

[evolution]
allowed = ["add-optional-field", "remove-unused-field"]

[defaultValues]
riskTolerance = 0.05

[resourceFormalization]
autoMigrateResources = true
preserveControllerHistory = true
updateFungibilityDomains = { "ERC20:USDT": "MultiAsset:USDT" }
```

This allows Operators to automatically evolve program states **without User action** if the changes are non-breaking.


## Resource Migration During Upgrades

When upgrading programs that use the formalized resource model, special care must be taken:

```rust
// Example resource migration function
fn migrate_resource(old: &OldResource) -> Result<NewResource, MigrationError> {
    // Ensure resource conservation
    let old_delta = calculate_delta(&[old]);
    
    // Create new resource with updated fields
    let new = Resource {
        resource_logic: upgrade_logic(&old.resource_logic),
        fungibility_domain: old.fungibility_domain.clone(),
        quantity: old.quantity,
        metadata: add_version_field(&old.metadata),
        ephemeral: old.ephemeral,
        nonce: old.nonce,
        nullifier_pub_key: upgrade_nullifier_key(&old.nullifier_pub_key),
        randomness_seed: old.randomness_seed.clone(),
    };
    
    // Verify conservation law is maintained
    let new_delta = calculate_delta(&[&new]);
    if new_delta != old_delta {
        Err(MigrationError::ConservationViolation { 
            old_delta, 
            new_delta 
        })
    } else {
        Ok(new)
    }
}
```

### Controller Label Migration

```rust
// Example controller label migration
fn migrate_controller_label(old: &OldControllerLabel) -> Result<NewControllerLabel, MigrationError> {
    // Check if creating controller is still valid
    let new_creating_controller = upgrade_controller(&old.creating_controller);
    
    // Update terminal controller if needed
    let new_terminal_controller = upgrade_controller(&old.terminal_controller);
    
    // Update all affecting controllers
    let new_affecting_controllers = old.affecting_controllers
        .iter()
        .map(|controller| upgrade_controller(controller))
        .collect();
    
    // Create new controller label
    Ok(ControllerLabel {
        creating_controller: new_creating_controller,
        terminal_controller: new_terminal_controller,
        affecting_controllers: new_affecting_controllers,
        backup_controllers: old.backup_controllers
            .iter()
            .map(|controller| upgrade_controller(controller))
            .collect(),
    })
}
```

## Pending Return Hostage Risk
A special risk arises when **Program A** calls **Program B**, but B deliberately **refuses to return** — blocking A's upgrade. Options to mitigate:

| Option | Description |
|---|---|
| Timeout + Degrade | After timeout, auto-abort call and enter degraded state. |
| Fallback State | Define a fallback state for incomplete calls. |
| Self-Contained Programs | Programs can opt to avoid cross-program calls entirely. |
| Safe State Handshake | Programs negotiate return guarantees before calls. |
| Resource Timeout Nullifiers | Resources locked in pending calls can be nullified and recreated after timeout. |

Default: **Timeout + Degrade with Resource Conservation**


## Cross-domain Resource Risk During Upgrades

For programs that manage cross-domain resources, additional risks during upgrades include:

1. **In-Transit Resources**: Resources that are in the process of crossing Domains during an upgrade.
2. **Controller Label Incompatibility**: A new version changes controller label requirements.
3. **Conservation Violations**: Upgrade migrations that would violate ΔTX = 0.
4. **Dual Validation Changes**: Modifications to temporal or ancestral validation.

Solutions to these risks:

| Risk | Mitigation |
|---|---|
| In-Transit Resources | Wait for quiescence or implement resource recovery mechanisms. |
| Label Incompatibility | Provide migration functions for controller labels. |
| Conservation Violations | Enforce static verification of migration functions. |
| Validation Changes | Ensure backward compatibility for validation rules. |


## Migration Flow

| Step | Action |
|---|---|
| Epoch N | Operators run Protocol 2.3.0 |
| Program P runs | Under Protocol 2.3.0 with Resource Formalization 1.0.0 |
| Epoch N+1 | ProtocolUpgrade to 3.0.0 with Resource Formalization 1.1.0 |
| Program P | Checks safe state + resource quiescence |
| If safe | Applies auto-migration + resource migration |
| If not safe | Stays pinned to 2.3.0 |
| If schema breaks | User provides migration function |
| If resource schema changes | Apply resource migration functions |
| If controller labels change | Apply controller label migration |
| Unified Log | Records `UpgradeProgram` effect with resource details |
| Verify | Confirm resource conservation laws maintained (ΔTX = 0) |


## Consequences of Staying Pinned
Users who **refuse upgrades** may need to:
- Incentivize Operators to run legacy versions.
- Lose access to newer program capabilities.
- Manually maintain effect adapters (if external Domains evolve).
- Handle all cross-program interop manually.
- Maintain compatibility with older resource formalization models.
- Miss security improvements in dual validation.

This is intentional — sovereignty > convenience.


## Replay and Audit
- Every upgrade is an **effect** in the unified log.
- Effects are tied to time map observations.
- Replay must:
    - Re-run migrations.
    - Reconstruct safe state checks.
    - Validate schema evolution rules.
    - Verify resource conservation (ΔTX = 0) across migrations.
    - Confirm controller label validity after migration.
    - Ensure dual validation rules are properly applied.


## Benefits

- Programs fully own their upgrade path.  
- Default path favors safe automatic evolution.  
- Operators remain compatible across epochs.  
- Schema evolution is strongly typed and rule-driven.  
- All upgrades leave a **permanent audit trail**.  
- External auditors can verify upgrade correctness by replaying logs.
- Resource conservation is maintained across upgrades.
- Controller labels preserve resource provenance during migrations.
- Dual validation ensures cross-domain security after upgrades.


## Visual Domain


```
Epoch N: Operators v2.3.0, Resource Model v1.0.0

ProgramA runs with ProgramVersion 1.2.0
ProgramA manages cross-domain resources with controller labels

Epoch N+1: Operators upgrade to v3.0.0, Resource Model v1.1.0

ProgramA checks safe state (no pending returns)
ProgramA checks resource quiescence (no cross-domain transfers in progress)
ProgramA applies resource migration functions to update resource tuples
ProgramA updates controller labels to new format
ProgramA auto-upgrades to 1.3.0
Unified log records UpgradeProgram effect with resource migration details

```


## Open Questions

| Question | Current Answer |
|---|---|
| How long do Operators support old protocols? | Configurable — minimum N epochs. |
| Can Users pre-sign migration functions? | Yes — pre-approved migrations can run automatically. |
| Can a program downgrade? | No — effects are forward-only. |
| Can program and protocol upgrades be decoupled? | Partially — programs choose to accept or reject protocol upgrades. |
| How to handle resources that span multiple programs during upgrades? | Coordinated upgrades or resource quiescence requirements. |
| How to migrate controller labels when controller types change? | Explicit controller mapping functions in migration. |
| Can resource formalization changes be separated from protocol upgrades? | Yes, through versioned resource formalization rules. |


## Summary

This upgrade model:

- Respects User sovereignty.  
- Supports automatic schema evolution.  
- Preserves causal traceability for all upgrades.  
- Provides flexible safe state strategies.  
- Uses time map snapshots to anchor all upgrade effects in causal time.
- Maintains resource conservation (ΔTX = 0) across upgrades.
- Preserves controller label provenance during migrations.
- Ensures dual validation integrity for cross-domain resources.

This strikes a careful balance between:

- Protecting program sovereignty.
- Ensuring Operators can safely upgrade.
- Preserving seamless replay and auditability.
- Maintaining resource integrity across versions.
- Supporting the full resource formalization model (ADR_018).
