# Getting Started with Causality

This guide will help you set up and run your first Causality project.

## Prerequisites

Before you begin, ensure you have the following installed:

- Rust (1.70 or later)
- Cargo (typically installed with Rust)
- Nix (for reproducible builds and environments)
- Git (for version control)

### Installing Rust and Cargo

If you don't have Rust installed, you can install it using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation.

### Installing Nix

To install Nix on macOS or Linux:

```bash
curl -L https://nixos.org/nix/install | sh
```

After installing Nix, enable the Flakes feature by creating or editing `~/.config/nix/nix.conf`:

```
experimental-features = nix-command flakes
```

## Setting Up a Causality Project

### Using the Nix Development Environment

1. Clone the Causality repository:

```bash
git clone https://github.com/timewave-team/causality.git
cd causality
```

2. Set up the development environment using Nix:

```bash
# If you have direnv installed
direnv allow

# Or without direnv
nix develop
```

This will set up a complete development environment with all required dependencies.

### Project Structure

A typical Causality project has the following structure:

```
my-causality-project/
├── Cargo.toml
├── flake.nix
├── src/
│   ├── main.rs
│   ├── programs/
│   ├── resources/
│   └── effects/
└── tests/
```

## Creating Your First Program

Let's create a simple Causality program that demonstrates key concepts.

1. Create a new file `src/programs/counter.rs`:

```rust
use causality_core::program::{Program, ProgramDefinition};
use causality_effects::effect::{Effect, EffectContext, EffectResult};
use causality_types::schema::{Schema, SchemaField, FieldType};
use serde::{Deserialize, Serialize};

// Define the counter program's state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CounterState {
    count: u64,
}

// Define the counter program
pub struct CounterProgram {
    // Program definition details
    definition: ProgramDefinition,
    // Current program state
    state: CounterState,
}

impl Program for CounterProgram {
    // Program implementation methods
    
    fn get_definition(&self) -> &ProgramDefinition {
        &self.definition
    }
    
    fn get_state(&self) -> serde_json::Value {
        serde_json::to_value(&self.state).unwrap()
    }
    
    fn increment(&mut self, ctx: &EffectContext) -> EffectResult<u64> {
        self.state.count += 1;
        Ok(self.state.count)
    }
    
    fn reset(&mut self, ctx: &EffectContext) -> EffectResult<()> {
        self.state.count = 0;
        Ok(())
    }
}

// Function to create a new counter program
pub fn create_counter_program() -> CounterProgram {
    // Define the schema for the program
    let schema = Schema::new(
        "counter",
        vec![
            SchemaField::new("count", FieldType::Uint64, serde_json::json!(0)),
        ],
    );
    
    // Create program definition
    let definition = ProgramDefinition::new(
        "counter",
        schema,
        vec![], // No dependencies
    );
    
    // Create program with initial state
    CounterProgram {
        definition,
        state: CounterState { count: 0 },
    }
}
```

2. Now, let's create a main file that uses this program at `src/main.rs`:

```rust
mod programs;

use causality_core::executor::{Executor, ExecutorConfig};
use causality_effects::registry::EffectRegistry;
use programs::counter::create_counter_program;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the effect registry
    let mut registry = EffectRegistry::new();
    
    // Add effect handlers (simplified for example)
    // registry.register_handler(...);
    
    // Create executor with configuration
    let config = ExecutorConfig::default();
    let mut executor = Executor::new(config, registry);
    
    // Create and register our counter program
    let counter_program = create_counter_program();
    let program_id = executor.register_program(Box::new(counter_program))?;
    
    println!("Counter program registered with ID: {}", program_id);
    
    // Get the program state
    let state = executor.get_program_state(&program_id)?;
    println!("Initial state: {}", state);
    
    // Execute the increment effect
    let result = executor.execute_program_method(&program_id, "increment", serde_json::json!({}))?;
    println!("After increment: {}", result);
    
    // Get updated state
    let state = executor.get_program_state(&program_id)?;
    println!("Updated state: {}", state);
    
    Ok(())
}
```

3. Update your `Cargo.toml` to include the necessary dependencies:

```toml
[package]
name = "my-causality-project"
version = "0.1.0"
edition = "2021"

[dependencies]
causality-core = "0.1"
causality-effects = "0.1"
causality-types = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Running Your First Program

1. Build and run your program:

```bash
cargo run
```

You should see output similar to:

```
Counter program registered with ID: 0x...
Initial state: {"count":0}
After increment: 1
Updated state: {"count":1}
```

## Next Steps

Now that you've created your first Causality program, here are some next steps to explore:

1. **Add Cross-Domain Effects**: Learn how to interact with blockchain domains like Ethereum or CosmWasm.
   - See [Cross-Domain Effects Guide](implementation/domain-system.md)

2. **Implement Resource Management**: Create, transfer, and manage content-addressed resources.
   - See [Resource System Guide](implementation/resource-system.md)

3. **Explore Capability-Based Security**: Learn about the permission model.
   - See [Capability System Guide](implementation/capability-system.md)

4. **Implement Time-Based Logic**: Leverage the time system for temporal constraints.
   - See [Time System Guide](implementation/time-system.md)

5. **Use Content Addressing**: Understand how to work with content-addressed objects.
   - See [Content Addressing Guide](implementation/content-addressing.md)

## Troubleshooting

### Common Issues

1. **Missing Dependencies**

If you see errors about missing dependencies, ensure you're using the Nix environment:

```bash
nix develop
```

2. **Version Mismatches**

If you encounter version mismatch errors, check that your `Cargo.toml` is using compatible versions of the Causality crates.

3. **Runtime Errors**

For effect execution errors, check:
- Effect handler registration
- Capability permissions
- Domain connectivity (for cross-chain operations)

### Getting Help

- Check the [Reference Documentation](../reference/)
- Join the [Causality Discord](https://discord.gg/causality)
- File issues on [GitHub](https://github.com/timewave-team/causality/issues)

## Further Resources

- [Architecture Overview](../architecture/README.md)
- [API Reference](../reference/api/rest.md)
- [CLI Reference](../reference/api/cli.md)
