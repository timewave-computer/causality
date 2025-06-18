// Purpose: Type-level capability system using session types and row types

use crate::layer1::types::{SessionType, Type, RowType};
use crate::layer2::effect::{Effect, EffectRow, EffectType, EffectOp, OpResult, Handler};
use crate::layer3::agent::AgentId;
use std::marker::PhantomData;

/// A capability is a session type that grants access to handlers
#[derive(Debug, Clone, PartialEq)]
pub struct Capability {
    pub name: String,
    pub session_type: SessionType,
    pub allowed_effects: EffectRow,
}

/// Capability protocol - how capabilities transform effects
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityProtocol {
    /// Transform an effect with constraints to a pure effect
    UseEffect,
    /// Revoke the capability
    Revoke,
}

impl Capability {
    /// Create a new capability with row type constraints
    pub fn new(name: String, allowed_effects: EffectRow) -> Self {
        // Build session type for capability protocol
        let session_type = SessionType::ExternalChoice(
            vec![
                // Branch 1: Use effect
                ("use_effect".to_string(), SessionType::Receive(
                    Box::new(Type::Record(RowType::Empty)), // Simplified for now
                    Box::new(SessionType::Send(
                        Box::new(Type::Record(RowType::Empty)),
                        Box::new(SessionType::End)
                    ))
                )),
                // Branch 2: Revoke
                ("revoke".to_string(), SessionType::End),
            ]
        );
        
        Self {
            name,
            session_type,
            allowed_effects,
        }
    }
    
    /// Create a rate-limited API capability
    pub fn rate_limited_api(max_calls_per_minute: u32) -> Self {
        let allowed_effects = EffectRow::from_effects(vec![
            ("api_call".to_string(), EffectType::IO),
        ]);
        
        Self::new(
            format!("RateLimitedAPI({})", max_calls_per_minute),
            allowed_effects
        )
    }
    
    /// Create a data access capability with table restrictions
    pub fn data_access(allowed_tables: Vec<String>, exclude_tables: Vec<String>) -> Self {
        let mut effects = vec![];
        
        // Add read/write effects for allowed tables
        for table in &allowed_tables {
            if !exclude_tables.contains(table) {
                effects.push((format!("read_{}", table), EffectType::State));
                
                // Don't allow writes to sensitive tables
                if !["audit_log", "permissions"].contains(&table.as_str()) {
                    effects.push((format!("write_{}", table), EffectType::State));
                }
            }
        }
        
        let allowed_effects = EffectRow::from_effects(effects);
        Self::new("DataAccess".to_string(), allowed_effects)
    }
    
    /// Check if this capability allows a specific effect
    pub fn allows_effect(&self, effect_name: &str) -> bool {
        self.allowed_effects.has_effect(effect_name)
    }
    
    /// Check if this capability allows a specific operation
    pub fn allows_operation(&self, op: &EffectOp) -> bool {
        match op {
            EffectOp::StateRead(loc) => self.allows_effect(&format!("read_{:?}", loc)),
            EffectOp::StateWrite(loc, _) => self.allows_effect(&format!("write_{:?}", loc)),
            EffectOp::CommSend(chan, _) => self.allows_effect(&format!("send_{}", chan)),
            EffectOp::CommReceive(chan) => self.allows_effect(&format!("recv_{}", chan)),
            EffectOp::ProofGenerate(_, _) => self.allows_effect("proof_gen"),
            EffectOp::ProofVerify(_, _) => self.allows_effect("proof_verify"),
        }
    }
    
    /// Compose two capabilities by merging their allowed effects
    pub fn compose(&self, other: &Capability) -> Capability {
        // For now, we'll manually merge the effect rows
        // In a real implementation, we'd have proper row union
        let mut effects = vec![];
        
        // Extract effects from self
        self.extract_effects(&self.allowed_effects, &mut effects);
        
        // Extract effects from other
        self.extract_effects(&other.allowed_effects, &mut effects);
        
        let combined_effects = EffectRow::from_effects(effects);
        Capability::new(
            format!("{}+{}", self.name, other.name),
            combined_effects,
        )
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn extract_effects(&self, row: &EffectRow, effects: &mut Vec<(String, EffectType)>) {
        match row {
            EffectRow::Empty => {},
            EffectRow::Extend(label, ty, rest) => {
                effects.push((label.clone(), ty.clone()));
                self.extract_effects(rest, effects);
            },
            EffectRow::RowVar(_) => {},
        }
    }
}

/// Capability handler that enforces effect constraints
pub struct CapabilityHandler {
    capability: Capability,
}

impl CapabilityHandler {
    pub fn new(capability: Capability) -> Self {
        Self { capability }
    }
}

impl<R: 'static> Handler<R> for CapabilityHandler {
    fn transform_op(&self, op: EffectOp) -> Effect<OpResult, R> {
        if self.capability.allows_operation(&op) {
            // Allowed - convert operation to pure effect
            match op {
                EffectOp::StateRead(location) => Effect::StateRead {
                    location,
                    _result_type: PhantomData,
                },
                EffectOp::StateWrite(location, value) => Effect::StateWrite {
                    location,
                    value,
                    _result_type: PhantomData,
                },
                EffectOp::CommSend(channel, value) => Effect::CommSend {
                    channel,
                    value,
                    _result_type: PhantomData,
                },
                EffectOp::CommReceive(channel) => Effect::CommReceive {
                    channel,
                    _result_type: PhantomData,
                },
                EffectOp::ProofGenerate(claim, witness) => Effect::ProofGenerate {
                    claim,
                    witness,
                    _result_type: PhantomData,
                },
                EffectOp::ProofVerify(proof, claim) => Effect::ProofVerify {
                    proof,
                    claim,
                    _result_type: PhantomData,
                },
            }
        } else {
            // Not allowed - return error
            panic!("Capability {} does not allow operation", self.capability.name);
        }
    }
    
    fn name(&self) -> &str {
        &self.capability.name
    }
}

/// Use a capability to transform an effect
pub fn use_capability<T: Clone + 'static, R: 'static>(
    _capability: &Capability,
    effect: Effect<T, R>,
) -> Result<Effect<T, R>, String> {
    // In a real implementation, we'd check the effect against the capability
    // and apply the appropriate handler transformation
    Ok(effect)
}

#[allow(dead_code)]
fn check_delegation(
    _capability: &Capability,
    _delegator: &AgentId,
    _delegate: &AgentId,
) -> bool {
    // For now, allow all delegations
    // In a real implementation, would check permission rules
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limited_capability() {
        let cap = Capability::rate_limited_api(100);
        assert!(cap.allows_effect("api_call"));
        assert!(!cap.allows_effect("database_write"));
    }
    
    #[test]
    fn test_data_access_capability() {
        let cap = Capability::data_access(
            vec!["users".to_string(), "orders".to_string(), "audit_log".to_string()],
            vec!["audit_log".to_string()],
        );
        
        assert!(cap.allows_effect("read_users"));
        assert!(cap.allows_effect("write_users"));
        assert!(cap.allows_effect("read_orders"));
        assert!(!cap.allows_effect("read_audit_log"));
        assert!(!cap.allows_effect("write_audit_log"));
    }
    
    #[test]
    fn test_capability_composition() {
        let api_cap = Capability::rate_limited_api(100);
        let data_cap = Capability::data_access(
            vec!["users".to_string()],
            vec![],
        );
        
        let combined = api_cap.compose(&data_cap);
        assert!(combined.allows_effect("api_call"));
        assert!(combined.allows_effect("read_users"));
        assert!(combined.allows_effect("write_users"));
    }
}
