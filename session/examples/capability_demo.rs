// Example demonstrating type-level capabilities in the Causality-Valence architecture

use session::layer3::capability::Capability;
use session::layer2::effect::{EffectRow, EffectType};

fn main() {
    println!("=== Causality-Valence Type-Level Capability Demo ===\n");
    
    // Example 1: Rate-limited API capability
    println!("1. Communication Capability:");
    let comm_cap = Capability::new(
        "Communication".to_string(),
        EffectRow::from_effects(vec![
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
        ])
    );
    println!("   Name: {}", comm_cap.name);
    println!("   Allows 'comm_send': {}", comm_cap.allowed_effects.has_effect("comm_send"));
    println!("   Allows 'database_write': {}", comm_cap.allowed_effects.has_effect("database_write"));
    
    // Example 2: Data access capability with state operations
    println!("\n2. Data Access Capability:");
    let data_cap = Capability::new(
        "DataAccess".to_string(),
        EffectRow::from_effects(vec![
            ("state_read".to_string(), EffectType::State),
            ("state_write".to_string(), EffectType::State),
        ])
    );
    println!("   Name: {}", data_cap.name);
    println!("   Allows 'state_read': {}", data_cap.allowed_effects.has_effect("state_read"));
    println!("   Allows 'state_write': {}", data_cap.allowed_effects.has_effect("state_write"));
    println!("   Allows 'comm_send': {}", data_cap.allowed_effects.has_effect("comm_send"));
    
    // Example 3: Proof generation capability
    println!("\n3. Proof Generation Capability:");
    let proof_cap = Capability::new(
        "ProofGeneration".to_string(),
        EffectRow::from_effects(vec![
            ("proof_generate".to_string(), EffectType::Proof),
            ("proof_verify".to_string(), EffectType::Proof),
        ])
    );
    println!("   Name: {}", proof_cap.name);
    println!("   Allows 'proof_generate': {}", proof_cap.allowed_effects.has_effect("proof_generate"));
    println!("   Allows 'proof_verify': {}", proof_cap.allowed_effects.has_effect("proof_verify"));
    
    // Example 4: Full access capability
    println!("\n4. Full Access Capability:");
    let full_cap = Capability::new(
        "FullAccess".to_string(),
        EffectRow::from_effects(vec![
            ("state_read".to_string(), EffectType::State),
            ("state_write".to_string(), EffectType::State),
            ("comm_send".to_string(), EffectType::Comm),
            ("comm_receive".to_string(), EffectType::Comm),
            ("proof_generate".to_string(), EffectType::Proof),
            ("proof_verify".to_string(), EffectType::Proof),
        ])
    );
    
    println!("   Name: {}", full_cap.name);
    println!("   Allows 'state_read': {}", full_cap.allowed_effects.has_effect("state_read"));
    println!("   Allows 'comm_send': {}", full_cap.allowed_effects.has_effect("comm_send"));
    println!("   Allows 'proof_generate': {}", full_cap.allowed_effects.has_effect("proof_generate"));
    println!("   Allows 'unauthorized_op': {}", full_cap.allowed_effects.has_effect("unauthorized_op"));
    
    println!("\n=== Capability Composition Demo ===");
    
    // Show how capabilities restrict agent behavior
    use session::layer3::agent::Agent;
    
    let mut restricted_agent = Agent::new("RestrictedAgent");
    restricted_agent.add_capability(comm_cap.clone());
    
    let mut full_agent = Agent::new("FullAgent");
    full_agent.add_capability(full_cap.clone());
    
    println!("\nRestricted Agent capabilities: {} total", restricted_agent.capabilities.len());
    for cap in &restricted_agent.capabilities {
        println!("  - {}", cap.name);
    }
    
    println!("\nFull Agent capabilities: {} total", full_agent.capabilities.len());
    for cap in &full_agent.capabilities {
        println!("  - {}", cap.name);
    }
    
    println!("\nKey insights:");
    println!("- Capabilities define what effects an agent can perform");
    println!("- Effect rows specify precise permissions");
    println!("- Type system enforces constraints at compile time");
    println!("- Agents can have multiple capabilities for different operations");
    println!("- Capability-based access control prevents unauthorized operations");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_demo() {
        // Test that the demo runs without panicking
        main();
    }
    
    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(
            "TestCapability".to_string(),
            EffectRow::from_effects(vec![
                ("test_effect".to_string(), EffectType::State),
            ])
        );
        
        assert_eq!(cap.name, "TestCapability");
        assert!(cap.allowed_effects.has_effect("test_effect"));
        assert!(!cap.allowed_effects.has_effect("nonexistent_effect"));
    }
} 