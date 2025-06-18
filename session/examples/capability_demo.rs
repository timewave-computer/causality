// Example demonstrating type-level capabilities in the Causality-Valence architecture

// Simulated imports (in real code these would come from the session crate)
use std::marker::PhantomData;

/// Effect row types (simplified)
#[derive(Debug, Clone, PartialEq)]
enum EffectRow {
    Empty,
    Extend(String, EffectType, Box<EffectRow>),
}

#[derive(Debug, Clone, PartialEq)]
enum EffectType {
    State,
    IO,
    Comm,
}

/// Session types (simplified)
#[derive(Debug, Clone, PartialEq)]
enum SessionType {
    ExternalChoice(Vec<(String, SessionType)>),
    Receive(Box<SessionType>),
    Send(Box<SessionType>),
    End,
}

/// A capability is a session type that grants access to handlers
#[derive(Debug, Clone)]
struct Capability {
    name: String,
    session_type: SessionType,
    allowed_effects: EffectRow,
}

impl Capability {
    fn new(name: String, allowed_effects: EffectRow) -> Self {
        // Capability protocol: receive effect, send transformed effect, or revoke
        let session_type = SessionType::ExternalChoice(vec![
            ("use_effect".to_string(), SessionType::Receive(
                Box::new(SessionType::Send(
                    Box::new(SessionType::End)
                ))
            )),
            ("revoke".to_string(), SessionType::End),
        ]);
        
        Self {
            name,
            session_type,
            allowed_effects,
        }
    }
    
    /// Create a rate-limited API capability
    fn rate_limited_api(max_calls: u32) -> Self {
        let effects = EffectRow::Extend(
            "api_call".to_string(),
            EffectType::IO,
            Box::new(EffectRow::Empty),
        );
        
        Self::new(
            format!("RateLimitedAPI({})", max_calls),
            effects,
        )
    }
    
    /// Create a data access capability
    fn data_access(allowed_tables: Vec<&str>, exclude_tables: Vec<&str>) -> Self {
        let mut effects = EffectRow::Empty;
        
        // Build effect row for allowed operations
        for table in allowed_tables {
            if !exclude_tables.contains(&table) {
                // Read effect
                effects = EffectRow::Extend(
                    format!("read_{}", table),
                    EffectType::State,
                    Box::new(effects),
                );
                
                // Write effect (exclude sensitive tables)
                if !["audit_log", "permissions"].contains(&table) {
                    effects = EffectRow::Extend(
                        format!("write_{}", table),
                        EffectType::State,
                        Box::new(effects),
                    );
                }
            }
        }
        
        Self::new("DataAccess".to_string(), effects)
    }
    
    /// Check if capability allows an effect
    fn allows_effect(&self, effect_name: &str) -> bool {
        self.check_row(&self.allowed_effects, effect_name)
    }
    
    fn check_row(&self, row: &EffectRow, name: &str) -> bool {
        match row {
            EffectRow::Empty => false,
            EffectRow::Extend(label, _, rest) => {
                label == name || self.check_row(rest, name)
            }
        }
    }
}

fn main() {
    println!("=== Causality-Valence Type-Level Capability Demo ===\n");
    
    // Example 1: Rate-limited API capability
    println!("1. Rate-Limited API Capability:");
    let api_cap = Capability::rate_limited_api(100);
    println!("   Name: {}", api_cap.name);
    println!("   Allows 'api_call': {}", api_cap.allows_effect("api_call"));
    println!("   Allows 'database_write': {}", api_cap.allows_effect("database_write"));
    
    // Example 2: Data access capability with table restrictions
    println!("\n2. Data Access Capability:");
    let data_cap = Capability::data_access(
        vec!["users", "orders", "products", "audit_log"],
        vec!["audit_log"],
    );
    println!("   Name: {}", data_cap.name);
    println!("   Allows 'read_users': {}", data_cap.allows_effect("read_users"));
    println!("   Allows 'write_users': {}", data_cap.allows_effect("write_users"));
    println!("   Allows 'read_audit_log': {}", data_cap.allows_effect("read_audit_log"));
    println!("   Allows 'write_audit_log': {}", data_cap.allows_effect("write_audit_log"));
    println!("   Allows 'write_permissions': {}", data_cap.allows_effect("write_permissions"));
    
    // Example 3: Composing capabilities
    println!("\n3. Composed Capability:");
    
    // In the real system, this would use row type union
    let mut combined_effects = api_cap.allowed_effects.clone();
    
    // Add data access effects
    fn add_effects(base: EffectRow, to_add: &EffectRow) -> EffectRow {
        match to_add {
            EffectRow::Empty => base,
            EffectRow::Extend(label, ty, rest) => {
                let new_base = EffectRow::Extend(
                    label.clone(),
                    ty.clone(),
                    Box::new(base),
                );
                add_effects(new_base, rest)
            }
        }
    }
    
    combined_effects = add_effects(combined_effects, &data_cap.allowed_effects);
    
    let combined_cap = Capability::new(
        format!("{} + {}", api_cap.name, data_cap.name),
        combined_effects,
    );
    
    println!("   Name: {}", combined_cap.name);
    println!("   Allows 'api_call': {}", combined_cap.allows_effect("api_call"));
    println!("   Allows 'read_users': {}", combined_cap.allows_effect("read_users"));
    
    // Example 4: Session type protocol
    println!("\n4. Capability Protocol:");
    println!("   The capability uses a session type that allows:");
    println!("   - 'use_effect': Transform an effect with constraints");
    println!("   - 'revoke': End the capability session");
    
    println!("\nKey insights:");
    println!("- Capabilities are session types, not a separate system");
    println!("- Effect constraints are expressed as row types");
    println!("- Type system enforces constraints at compile time");
    println!("- No need for runtime constraint checking");
} 