# Invariant Checking in Causality Simulation

This document describes how to use the invariant checking functionality in the Causality Simulation system. Invariants are rules that must be maintained during simulation execution, and violations are automatically detected and reported.

## Overview

Invariant checking is a key feature of the simulation system that enables developers to:

1. Define rules that must be maintained during simulation execution
2. Automatically detect and report violations in real-time
3. Fail simulations when critical invariants are violated
4. Verify system properties in an automated way

## Defining Invariants in Scenario Files

Invariants are defined in the scenario TOML file using the `[invariants]` section. Each invariant is a key-value pair where the key is the invariant name and the value is a boolean indicating if the invariant should be checked.

Example:

```toml
[invariants]
no_negative_balances = true
```

### Supported Invariants

Currently, the following invariants are supported:

| Invariant Name | Description |
|----------------|-------------|
| `no_negative_balances` | Ensures that no agent's balance for any asset goes below zero |

## Running Scenarios with Invariant Checking

When running a scenario, you can use the `--fail-on-invariant-violation` flag to make the simulation fail immediately if any invariant is violated:

```sh
nix run .#controller -- run path/to/scenario.toml --fail-on-invariant-violation
```

Without this flag, violations will be reported but the simulation will continue running.

## Checking Invariants Manually

You can check invariants for a running scenario using the `check-invariants` command:

```sh
nix run .#controller -- check-invariants scenario_name
```

This will report any violations that have occurred and return an error code if violations are found.

## Adding Custom Invariants

To add a custom invariant, follow these steps:

1. Update the `InvariantConfig` struct in `src/scenario.rs` to include your new invariant:

```rust
pub struct InvariantConfig {
    pub no_negative_balances: Option<bool>,
    pub your_custom_invariant: Option<bool>,
}
```

2. Create a new invariant checker by implementing the `InvariantChecker` trait in `src/invariant.rs`:

```rust
pub struct YourCustomInvariantChecker {
    // State needed for checking
}

impl InvariantChecker for YourCustomInvariantChecker {
    fn invariant_type(&self) -> InvariantType {
        InvariantType::Custom("YourCustomInvariant".to_string())
    }
    
    fn check(&self, entry: &LogEntry) -> InvariantResult {
        // Implement your checking logic here
        // Return InvariantResult::Satisfied or InvariantResult::Violated
    }
    
    fn log_filter(&self) -> Option<LogFilter> {
        // Return a filter for log entries relevant to this invariant
    }
}
```

3. Register your checker in the `InvariantObserver::from_config` method in `src/invariant.rs`:

```rust
pub fn from_config(config: &InvariantConfig) -> Self {
    let mut observer = Self::new();
    
    if let Some(true) = config.no_negative_balances {
        observer.add_checker(Box::new(NoNegativeBalancesChecker::new()));
    }
    
    if let Some(true) = config.your_custom_invariant {
        observer.add_checker(Box::new(YourCustomInvariantChecker::new()));
    }
    
    observer
}
```

4. Add your checker to the factory method:

```rust
impl InvariantCheckerFactory {
    // ...
    
    pub fn create_all_from_config(config: &InvariantConfig) -> Vec<Box<dyn InvariantChecker>> {
        let mut checkers = Vec::new();
        
        if let Some(true) = config.no_negative_balances {
            checkers.push(Self::create_no_negative_balances_checker());
        }
        
        if let Some(true) = config.your_custom_invariant {
            checkers.push(Box::new(YourCustomInvariantChecker::new()));
        }
        
        checkers
    }
}
```

## How Invariant Checking Works

Invariant checking is implemented using the observer pattern:

1. The `InvariantObserver` implements the `Observer` trait to receive log entries
2. Each invariant checker is registered with the observer
3. When a log entry is received, it's passed to each relevant checker
4. If a violation is detected, it's reported via a callback
5. The controller tracks violations and reports them to the user

Invariant checking happens in real-time during simulation execution, allowing for immediate feedback on rule violations.

## Best Practices

- Define invariants for critical system properties
- Start with built-in invariants before creating custom ones
- Use the `--fail-on-invariant-violation` flag in CI environments
- Implement custom checkers for domain-specific rules
- Keep invariant checking logic simple and focused 