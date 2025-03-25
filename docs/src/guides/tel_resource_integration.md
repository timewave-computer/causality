<!-- Tutorial for TEL resource integration -->
<!-- Original file: docs/src/tel_resource_integration_tutorial.md -->

# TEL Resource Integration Tutorial

This tutorial will guide you through integrating the Temporal Effect Language (TEL) resource system into your application. By the end, you'll have a fully functional implementation that manages resources with temporal effects.

## Prerequisites

- Rust (1.56.0 or newer)
- Basic understanding of Rust's async/await and error handling
- Familiarity with resource management concepts

## Step 1: Add Dependencies

First, add the TEL crate to your `Cargo.toml`:

```toml
[dependencies]
tel = { git = "https://github.com/yourusername/timewave-causality.git" }
uuid = "1.3.0"
serde = { version = "1.0", features = ["derive"] }
```

## Step 2: Create a Basic TEL Instance

Let's start by creating a simple TEL instance and exploring its capabilities:

```rust
use std::sync::Arc;
use tel::{TelBuilder, TelError, TelResult};

fn main() -> TelResult<()> {
    // Create a TEL instance using the builder pattern
    let tel = TelBuilder::new()
        .with_instance_id("my-first-tel")
        .build();
    
    println!("TEL instance created successfully!");
    
    // Access the core components
    let resource_manager = tel.resource_manager;
    let version_manager = tel.version_manager;
    let snapshot_manager = tel.snapshot_manager;
    
    // Now we can use these managers to work with resources
    // ...
    
    Ok(())
}
```

## Step 3: Create and Manage Resources

Now, let's create some resources and perform operations on them:

```rust
use tel::{Domain, Address, RegisterContents};

fn create_resources(tel: &tel::TelSystem) -> TelResult<()> {
    // Create a domain for our resources
    let domain = Domain::new("Etemenanki");
    
    // Create some owners
    let alice = Address::random();
    let bob = Address::random();
    
    // Create resources
    let alice_account = tel.resource_manager.create_resource(
        &alice,
        &domain,
        RegisterContents::Number(1000) // Initial balance of 1000
    )?;
    
    let bob_account = tel.resource_manager.create_resource(
        &bob,
        &domain,
        RegisterContents::Number(500) // Initial balance of 500
    )?;
    
    println!("Created accounts: Alice: {:?}, Bob: {:?}", alice_account, bob_account);
    
    // Update a resource
    tel.resource_manager.update_resource(
        &alice_account,
        RegisterContents::Number(1100) // New balance of 1100
    )?;
    
    // Transfer ownership of Bob's account to Alice
    tel.resource_manager.transfer_resource(&bob_account, &alice)?;
    
    // List resources by owner
    let alice_resources = tel.resource_manager.list_resources_by_owner(&alice)?;
    println!("Alice now owns {} resources", alice_resources.len());
    
    Ok(())
}
```

## Step 4: Using Version Control

Track changes to resources using the version control system:

```rust
fn use_versioning(tel: &tel::TelSystem, resource_id: &tel::ResourceId) -> TelResult<()> {
    // Get the current state of the resource
    let resource = tel.resource_manager.get_resource(resource_id)?;
    
    // Create a new version
    let version_id = tel.version_manager.create_version(
        resource_id, 
        resource.data.clone()
    )?;
    
    println!("Created version: {:?}", version_id);
    
    // Update the resource
    tel.resource_manager.update_resource(
        resource_id,
        RegisterContents::Number(1200) // New balance of 1200
    )?;
    
    // Get version history
    let history = tel.version_manager.get_history(resource_id)?;
    println!("Resource has {} versions", history.len());
    
    // Rollback to the previous version
    tel.version_manager.rollback(resource_id, &version_id)?;
    
    // Verify the rollback worked
    let resource = tel.resource_manager.get_resource(resource_id)?;
    match resource.data {
        RegisterContents::Number(n) => {
            assert_eq!(n, 1100); // Should be back to 1100
            println!("Successfully rolled back to balance: {}", n);
        },
        _ => panic!("Unexpected data type"),
    }
    
    Ok(())
}
```

## Step 5: Working with Snapshots

Create and manage system-wide snapshots:

```rust
use std::time::Duration;

fn use_snapshots(tel: &tel::TelSystem) -> TelResult<()> {
    // Configure and enable automatic snapshots
    tel.snapshot_manager.configure_schedule(
        Duration::from_secs(3600), // Hourly snapshots
        10,                        // Keep last 10 snapshots
        true                       // Enable automatic snapshots
    )?;
    
    // Create a manual snapshot
    let snapshot_id = tel.snapshot_manager.create_snapshot()?;
    println!("Created snapshot: {:?}", snapshot_id);
    
    // List available snapshots
    let snapshots = tel.snapshot_manager.list_snapshots()?;
    println!("Available snapshots: {}", snapshots.len());
    
    // Make some changes to resources
    // ...
    
    // Restore from the snapshot
    tel.snapshot_manager.restore_snapshot(&snapshot_id)?;
    println!("Restored system state from snapshot");
    
    Ok(())
}
```

## Step 6: Using the Effect System

Now, let's use the effect system to perform operations on resources:

```rust
use tel::{
    ResourceEffect, ResourceEffectAdapter, 
    ResourceOperation, ResourceOperationType
};

fn use_effects(tel: &tel::TelSystem, alice_account: &tel::ResourceId) -> TelResult<()> {
    // Create an effect adapter
    let adapter = ResourceEffectAdapter::new(Arc::clone(&tel.resource_manager));
    
    // Create an operation to update Alice's account
    let operation = ResourceOperation::new(
        ResourceOperationType::Update {
            resource_id: *alice_account,
            new_data: RegisterContents::Number(1300), // New balance of 1300
        }
    );
    
    // Create an effect
    let effect = ResourceEffect::new(operation);
    
    // Apply the effect
    let result = adapter.apply(effect)?;
    
    if result.success {
        println!("Effect applied successfully!");
    } else if let Some(error) = result.error {
        println!("Effect failed: {}", error);
    }
    
    // Verify the update
    let resource = tel.resource_manager.get_resource(alice_account)?;
    match resource.data {
        RegisterContents::Number(n) => {
            assert_eq!(n, 1300);
            println!("Alice's new balance: {}", n);
        },
        _ => panic!("Unexpected data type"),
    }
    
    Ok(())
}
```

## Step 7: Composing Effects

Let's compose multiple effects to perform a transaction between accounts:

```rust
use tel::EffectComposer;

fn transfer_funds(
    adapter: &ResourceEffectAdapter,
    from_account: &tel::ResourceId,
    to_account: &tel::ResourceId,
    amount: i64
) -> TelResult<()> {
    // Get current balances
    let from_resource = tel.resource_manager.get_resource(from_account)?;
    let to_resource = tel.resource_manager.get_resource(to_account)?;
    
    let from_balance = match from_resource.data {
        RegisterContents::Number(n) => n,
        _ => return Err(TelError::InvalidOperation("Source account has invalid data type".to_string())),
    };
    
    let to_balance = match to_resource.data {
        RegisterContents::Number(n) => n,
        _ => return Err(TelError::InvalidOperation("Target account has invalid data type".to_string())),
    };
    
    // Check if there are sufficient funds
    if from_balance < amount {
        return Err(TelError::InvalidOperation("Insufficient funds".to_string()));
    }
    
    // Create operations
    let debit_op = ResourceOperation::new(
        ResourceOperationType::Update {
            resource_id: *from_account,
            new_data: RegisterContents::Number(from_balance - amount),
        }
    );
    
    let credit_op = ResourceOperation::new(
        ResourceOperationType::Update {
            resource_id: *to_account,
            new_data: RegisterContents::Number(to_balance + amount),
        }
    );
    
    // Create effects
    let debit_effect = ResourceEffect::new(debit_op);
    let credit_effect = ResourceEffect::new(credit_op);
    
    // Compose effects
    let mut composer = EffectComposer::new();
    composer.add_effect(debit_effect);
    composer.add_effect(credit_effect);
    
    // Apply effects sequentially (important for transactions)
    let results = adapter.apply_sequence(composer.get_effects().to_vec())?;
    
    // Check if all effects were successful
    let all_successful = results.iter().all(|r| r.success);
    
    if all_successful {
        println!("Transfer of {} completed successfully", amount);
    } else {
        println!("Transfer failed: {:?}", results);
    }
    
    Ok(())
}
```

## Step 8: Using Repeating Effects

Let's implement an interest payment that occurs at regular intervals:

```rust
use tel::{RepeatingEffect, RepeatSchedule};
use std::time::Duration;

fn set_up_interest_payment(
    adapter: &ResourceEffectAdapter,
    account: &tel::ResourceId,
    interest_rate: f64,
    interval: Duration
) -> TelResult<RepeatingEffect> {
    // Create a custom operation for applying interest
    let apply_interest = move |account: &tel::ResourceId, rate: f64| -> TelResult<ResourceOperation> {
        let resource = tel.resource_manager.get_resource(account)?;
        
        let balance = match resource.data {
            RegisterContents::Number(n) => n as f64,
            _ => return Err(TelError::InvalidOperation("Account has invalid data type".to_string())),
        };
        
        // Calculate new balance with interest
        let interest = balance * rate;
        let new_balance = balance + interest;
        
        Ok(ResourceOperation::new(
            ResourceOperationType::Update {
                resource_id: *account,
                new_data: RegisterContents::Number(new_balance.round() as i64),
            }
        ))
    };
    
    // Create the initial operation
    let operation = apply_interest(account, interest_rate)?;
    
    // Create the effect
    let effect = ResourceEffect::new(operation);
    
    // Create a repeating effect that runs at the specified interval
    let repeating_effect = RepeatingEffect::repeat_interval(effect, interval);
    
    println!("Set up interest payments at rate: {}% every {:?}", 
        interest_rate * 100.0, interval);
    
    Ok(repeating_effect)
}

// To apply the repeating effect:
fn apply_interest(
    adapter: &ResourceEffectAdapter,
    repeating_effect: &RepeatingEffect
) -> TelResult<()> {
    let results = adapter.apply_repeating(repeating_effect)?;
    
    if !results.is_empty() {
        println!("Applied interest payments: {} executions", results.len());
    }
    
    Ok(())
}
```

## Step 9: Adding Proof Verification

Let's add proof generation and verification to secure our operations:

```rust
use tel::{
    EffectProofGenerator, EffectProofVerifier, 
    EffectProofFormat, EffectProofMetadata
};

fn use_proofs(
    adapter: &ResourceEffectAdapter,
    account: &tel::ResourceId
) -> TelResult<()> {
    // Create an operation
    let operation = ResourceOperation::new(
        ResourceOperationType::Update {
            resource_id: *account,
            new_data: RegisterContents::Number(1500), // New balance of 1500
        }
    );
    
    // Create an effect
    let effect = ResourceEffect::new(operation);
    
    // Create a proof generator
    let generator = EffectProofGenerator::new(
        EffectProofFormat::Groth16,
        Address::random() // Creator address
    );
    
    // Create metadata for the proof
    let metadata = EffectProofMetadata::new(
        Some(*account),
        Address::random(), // Creator address
        EffectProofFormat::Groth16
    );
    
    // Generate a proof
    let proof = generator.generate_proof(&effect, Some(metadata))?;
    
    // Attach the proof to the effect
    let effect_with_proof = effect.with_proof(proof.clone());
    
    // Create a verifier
    let verifier = EffectProofVerifier::default();
    
    // Verify the proof
    let is_valid = verifier.verify_proof(&effect, &proof)?;
    
    if is_valid {
        println!("Proof verified successfully");
        
        // Apply the effect
        let result = adapter.apply(effect_with_proof)?;
        
        if result.success {
            println!("Effect with proof applied successfully");
        }
    } else {
        println!("Proof verification failed");
    }
    
    Ok(())
}
```

## Step 10: Putting It All Together

Now, let's implement a complete application that uses all these components:

```rust
use std::sync::Arc;
use std::time::Duration;
use tel::{
    TelBuilder, Domain, Address, RegisterContents,
    ResourceEffectAdapter, RepeatingEffect
};

fn main() -> TelResult<()> {
    // Create a TEL instance
    let tel = TelBuilder::new()
        .with_instance_id("banking-system")
        .with_snapshot_schedule(
            Duration::from_secs(3600), // Hourly snapshots
            24,                        // Keep last 24 snapshots
            true                       // Enable automatic snapshots
        )
        .build();
    
    println!("Banking system initialized");
    
    // Create domain and accounts
    let bank_domain = Domain::new("banking");
    let alice = Address::random();
    let bob = Address::random();
    
    // Create accounts
    let alice_account = tel.resource_manager.create_resource(
        &alice,
        &bank_domain,
        RegisterContents::Number(1000) // Alice starts with 1000
    )?;
    
    let bob_account = tel.resource_manager.create_resource(
        &bob,
        &bank_domain,
        RegisterContents::Number(500) // Bob starts with 500
    )?;
    
    println!("Created accounts for Alice and Bob");
    
    // Create an effect adapter
    let adapter = ResourceEffectAdapter::new(Arc::clone(&tel.resource_manager));
    
    // Transfer 200 from Alice to Bob
    transfer_funds(&adapter, &alice_account, &bob_account, 200)?;
    
    // Set up monthly interest payments (1% per month)
    let interest_rate = 0.01;
    let monthly = Duration::from_secs(30 * 24 * 60 * 60); // 30 days
    
    let alice_interest = set_up_interest_payment(
        &adapter, &alice_account, interest_rate, monthly
    )?;
    
    let bob_interest = set_up_interest_payment(
        &adapter, &bob_account, interest_rate, monthly
    )?;
    
    // Create a snapshot
    let snapshot_id = tel.snapshot_manager.create_snapshot()?;
    println!("Created system snapshot: {:?}", snapshot_id);
    
    // Simulate time passing and apply interest
    apply_interest(&adapter, &alice_interest)?;
    apply_interest(&adapter, &bob_interest)?;
    
    // Check final balances
    let alice_resource = tel.resource_manager.get_resource(&alice_account)?;
    let bob_resource = tel.resource_manager.get_resource(&bob_account)?;
    
    match (alice_resource.data, bob_resource.data) {
        (RegisterContents::Number(alice_balance), RegisterContents::Number(bob_balance)) => {
            println!("Final balances:");
            println!("Alice: {}", alice_balance);
            println!("Bob: {}", bob_balance);
        },
        _ => println!("Unexpected data type"),
    }
    
    println!("Banking simulation completed successfully");
    
    Ok(())
}
```

## Conclusion

Congratulations! You've successfully integrated the TEL resource system into your application. You've learned how to:

1. Create and manage resources
2. Track resource versions
3. Create system snapshots
4. Apply effects to resources
5. Compose multiple effects
6. Set up repeating effects
7. Generate and verify proofs

This foundation allows you to build complex temporal applications with robust resource management, versioning, and effect handling. 