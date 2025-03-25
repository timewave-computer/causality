// REPL implementation for interactive use
// Original file: src/bin/repl.rs

// ResourceRegister REPL for Causality
//
// This module provides a Read-Eval-Print Loop for interacting with
// ResourceRegister and UnifiedRegistry functionality.

use std::io::{self, Write};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use colored::Colorize;
use rustyline::Editor;
use rustyline::error::ReadlineError;

use adapter_generator::error::{Error, Result};
use adapter_generator::crypto::hash::ContentId;
use adapter_generator::resource::resource_register::{
    ResourceRegister, ResourceLogic, RegisterState, 
    FungibilityDomain, Quantity, StorageStrategy, StateVisibility
};
use adapter_generator::resource::unified_registry::UnifiedRegistry;
use adapter_generator::tel::types::Metadata;

fn main() -> Result<()> {
    // Print welcome banner
    print_welcome();
    
    // Create a unified registry
    let registry = UnifiedRegistry::shared();
    
    // Create a rustyline editor for history
    let mut rl = Editor::<()>::new().unwrap();
    if rl.load_history(".repl_history").is_err() {
        println!("No previous history.");
    }
    
    // Start REPL loop
    loop {
        let readline = rl.readline("causality> ");
        match readline {
            Ok(line) => {
                // Add to history
                rl.add_history_entry(line.as_str());
                
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }
                
                // Process command
                match process_command(line, &registry) {
                    Ok(should_exit) => {
                        if should_exit {
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("{} {}", "Error:".red().bold(), e);
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C received, exiting...");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D received, exiting...");
                break;
            },
            Err(err) => {
                eprintln!("{} {}", "Error:".red().bold(), err);
                break;
            }
        }
    }
    
    // Save history
    rl.save_history(".repl_history").unwrap_or(());
    
    Ok(())
}

// Process a command
fn process_command(command: String, registry: &Arc<RwLock<UnifiedRegistry>>) -> Result<bool> {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    
    if parts.is_empty() {
        return Ok(false);
    }
    
    match parts[0].to_lowercase().as_str() {
        "exit" | "quit" => {
            println!("Goodbye!");
            return Ok(true);
        },
        "help" => {
            print_help();
        },
        "create" => {
            if parts.len() < 3 {
                println!("Usage: create <id> <logic_type> [quantity]");
                return Ok(false);
            }
            
            let id = parts[1];
            let logic_type = parts[2];
            let quantity = if parts.len() > 3 {
                parts[3].parse::<u128>().unwrap_or(1)
            } else {
                1
            };
            
            create_register(registry, id, logic_type, quantity)?;
        },
        "list" => {
            list_registers(registry)?;
        },
        "get" => {
            if parts.len() < 2 {
                println!("Usage: get <id>");
                return Ok(false);
            }
            
            let id = parts[1];
            get_register(registry, id)?;
        },
        "update" => {
            if parts.len() < 3 {
                println!("Usage: update <id> <state>");
                return Ok(false);
            }
            
            let id = parts[1];
            let state = parts[2];
            update_register_state(registry, id, state)?;
        },
        "remove" => {
            if parts.len() < 2 {
                println!("Usage: remove <id>");
                return Ok(false);
            }
            
            let id = parts[1];
            remove_register(registry, id)?;
        },
        "consume" => {
            if parts.len() < 2 {
                println!("Usage: consume <id>");
                return Ok(false);
            }
            
            let id = parts[1];
            consume_register(registry, id)?;
        },
        _ => {
            println!("Unknown command: {}", parts[0]);
            println!("Type 'help' for a list of commands");
        }
    }
    
    Ok(false)
}

// Print the welcome banner
fn print_welcome() {
    println!("{}", r#"
  __  __ _    _ ___ ___ ___ ___ ___  _____ ___ ___ ___ ___ _____ ___ ___ 
 |  \/  | |  | | _ \ __| __|   \  _/ __|  |  |_ _/ __|   \_ _|_   _| __| _ \
 | |\/| | |__| |   / _|| _|| |) / /\__ \  |  || |\__ \ |) | |  | | | _||   /
 |_|  |_|____|_|_|_\___|___|___/___|___/___/|___|___/___/___| |_| |___|_|_\
    "#.blue().bold());
    println!("{}", "ResourceRegister REPL for Causality".bright_green().bold());
    println!("{}", "Type 'help' for a list of commands".cyan());
    println!("");
}

// Print help
fn print_help() {
    println!("{}", "Commands:".bright_green().bold());
    println!("  {} - Create a new ResourceRegister", "create <id> <logic_type> [quantity]".yellow());
    println!("  {} - List all registers", "list".yellow());
    println!("  {} - Get a register by ID", "get <id>".yellow());
    println!("  {} - Update a register's state", "update <id> <state>".yellow());
    println!("  {} - Remove a register", "remove <id>".yellow());
    println!("  {} - Consume a register", "consume <id>".yellow());
    println!("  {} - Show this help", "help".yellow());
    println!("  {} - Exit the REPL", "exit".yellow());
    println!("");
    println!("{}", "Resource logic types:".bright_green().bold());
    println!("  {} - Fungible tokens", "fungible".yellow());
    println!("  {} - Non-fungible tokens", "nonfungible".yellow());
    println!("  {} - Capability", "capability".yellow());
    println!("  {} - Data", "data".yellow());
    println!("  {} - Custom logic", "custom:<type>".yellow());
    println!("");
    println!("{}", "Register states:".bright_green().bold());
    println!("  {} - Initial state", "initial".yellow());
    println!("  {} - Active state", "active".yellow());
    println!("  {} - Consumed state", "consumed".yellow());
    println!("  {} - Pending state", "pending".yellow());
    println!("  {} - Locked state", "locked".yellow());
    println!("  {} - Frozen state", "frozen".yellow());
    println!("  {} - Archived state", "archived".yellow());
    println!("");
}

// Create a new resource register
fn create_register(registry: &Arc<RwLock<UnifiedRegistry>>, id: &str, logic_type: &str, quantity: u128) -> Result<()> {
    // Parse the logic type
    let logic = match logic_type.to_lowercase().as_str() {
        "fungible" => ResourceLogic::Fungible,
        "nonfungible" => ResourceLogic::NonFungible,
        "capability" => ResourceLogic::Capability,
        "data" => ResourceLogic::Data,
        custom if custom.starts_with("custom:") => {
            let custom_type = custom.strip_prefix("custom:").unwrap_or("unknown");
            ResourceLogic::Custom(custom_type.to_string())
        },
        _ => {
            return Err(Error::InvalidArgument("Invalid logic type specified".to_string()));
        }
    };
    
    // Create the quantity
    let qty = Quantity(quantity);
    
    // Create a new content ID
    let content_id = ContentId::new(id);
    
    // Create the register with correct parameters
    let register = ResourceRegister::new(
        content_id.clone(),
        logic,
        FungibilityDomain(id.to_string()),
        qty,
        Metadata::new(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Register it
    let mut registry_guard = registry.write().map_err(|_| 
        Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
    
    registry_guard.register(register)?;
    
    println!("{} Register created with ID: {}", "Success:".green().bold(), id);
    
    Ok(())
}

// List all registers
fn list_registers(registry: &Arc<RwLock<UnifiedRegistry>>) -> Result<()> {
    // Get registry guard
    let registry_guard = registry.read().map_err(|_| 
        Error::ResourceError("Failed to acquire read lock on registry".to_string()))?;
    
    // Get all registers
    let registers = registry_guard.get_all()?;
    
    if registers.is_empty() {
        println!("No registers found.");
        return Ok(());
    }
    
    println!("{} {} registers found", "Info:".blue().bold(), registers.len());
    println!("\n{:<20} {:<15} {:<15} {:<15}", 
        "ID".green().bold(), 
        "Logic".green().bold(), 
        "Quantity".green().bold(), 
        "State".green().bold()
    );
    println!("{}", "-".repeat(70));
    
    for (id, register) in registers {
        // Format the logic type
        let logic = match register.resource_logic {
            ResourceLogic::Fungible => "Fungible".to_string(),
            ResourceLogic::NonFungible => "NonFungible".to_string(),
            ResourceLogic::Capability => "Capability".to_string(),
            ResourceLogic::Data => "Data".to_string(),
            ResourceLogic::Custom(ref custom) => format!("Custom:{}", custom),
        };
        
        // Format the state
        let state = format!("{:?}", register.state);
        
        println!("{:<20} {:<15} {:<15} {:<15}", 
            id.to_string(), 
            logic,
            register.quantity.0.to_string(),
            state
        );
    }
    
    println!("");
    Ok(())
}

// Get a register by ID
fn get_register(registry: &Arc<RwLock<UnifiedRegistry>>, id: &str) -> Result<()> {
    // Create a content ID
    let content_id = ContentId::new(id);
    
    // Get registry guard
    let registry_guard = registry.read().map_err(|_| 
        Error::ResourceError("Failed to acquire read lock on registry".to_string()))?;
    
    // Get the register
    let register_opt = registry_guard.get(&content_id)?;
    
    match register_opt {
        Some(register) => {
            println!("{} Register Details:", "Info:".blue().bold());
            println!("{}", "-".repeat(50));
            println!("{:<15} {}", "ID:".yellow(), register.id.to_string());
            
            // Format the logic type
            let logic = match register.resource_logic {
                ResourceLogic::Fungible => "Fungible".to_string(),
                ResourceLogic::NonFungible => "NonFungible".to_string(),
                ResourceLogic::Capability => "Capability".to_string(),
                ResourceLogic::Data => "Data".to_string(),
                ResourceLogic::Custom(ref custom) => format!("Custom:{}", custom),
            };
            
            println!("{:<15} {}", "Logic:".yellow(), logic);
            println!("{:<15} {}", "Quantity:".yellow(), register.quantity.0.to_string());
            println!("{:<15} {}", "State:".yellow(), format!("{:?}", register.state));
            println!("{:<15} {}", "Storage:".yellow(), format!("{:?}", register.storage_strategy));
            
            // If there's metadata, display it
            if !register.metadata.is_empty() {
                println!("{:<15}", "Metadata:".yellow());
                for (key, value) in register.metadata.iter() {
                    println!("  {:<15} {}", key.cyan(), value);
                }
            }
            
            println!("");
        },
        None => {
            println!("{} Register not found with ID: {}", "Error:".red().bold(), id);
        }
    }
    
    Ok(())
}

// Update a register's state
fn update_register_state(registry: &Arc<RwLock<UnifiedRegistry>>, id: &str, state: &str) -> Result<()> {
    // Create a content ID
    let content_id = ContentId::new(id);
    
    // Parse the state
    let new_state = match state.to_lowercase().as_str() {
        "initial" => RegisterState::Initial,
        "active" => RegisterState::Active,
        "consumed" => RegisterState::Consumed,
        "pending" => RegisterState::Pending,
        "locked" => RegisterState::Locked,
        "frozen" => RegisterState::Frozen,
        "archived" => RegisterState::Archived,
        _ => {
            return Err(Error::InvalidArgument("Invalid state specified".to_string()));
        }
    };
    
    // Get registry guard
    let mut registry_guard = registry.write().map_err(|_| 
        Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
    
    // Update the register
    registry_guard.update(&content_id, |register| {
        register.state = new_state.clone();
        Ok(())
    })?;
    
    println!("{} Register state updated to: {:?}", "Success:".green().bold(), new_state);
    
    Ok(())
}

// Remove a register
fn remove_register(registry: &Arc<RwLock<UnifiedRegistry>>, id: &str) -> Result<()> {
    // Create a content ID
    let content_id = ContentId::new(id);
    
    // Get registry guard
    let mut registry_guard = registry.write().map_err(|_| 
        Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
    
    // Remove the register
    let removed = registry_guard.remove(&content_id)?;
    
    match removed {
        Some(_) => {
            println!("{} Register removed: {}", "Success:".green().bold(), id);
        },
        None => {
            println!("Register not found with ID: {}", id);
        }
    }
    
    Ok(())
}

// Consume a register
fn consume_register(registry: &Arc<RwLock<UnifiedRegistry>>, id: &str) -> Result<()> {
    // Create a content ID
    let content_id = ContentId::new(id);
    
    // Get registry guard
    let mut registry_guard = registry.write().map_err(|_| 
        Error::ResourceError("Failed to acquire write lock on registry".to_string()))?;
    
    // Mark the register as consumed
    registry_guard.consume(&content_id)?;
    
    println!("{} Register consumed: {}", "Success:".green().bold(), id);
    
    Ok(())
} 