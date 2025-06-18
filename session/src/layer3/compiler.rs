// Layer 3 → Layer 2 compiler: Choreographies to effects
// Compiles choreographies to pure algebraic effects

use crate::layer2::effect::{Effect, EffectRow};
use crate::layer2::outcome::{Value, StateLocation};
use crate::layer3::agent::{AgentRegistry, AgentId};
use crate::layer3::choreography::{Choreography, ChoreographyStep, Message};
use thiserror::Error;

/// Compilation errors
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent {agent} lacks capability: {capability}")]
    MissingCapability { agent: String, capability: String },
    
    #[error("Invalid choreography: {0}")]
    InvalidChoreography(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

/// Compile a choreography to a list of effects
pub fn compile_choreography(
    choreography: &Choreography,
    registry: &AgentRegistry,
) -> Result<Vec<Effect<(), EffectRow>>, CompilerError> {
    match choreography {
        Choreography::Step(step) => {
            let effect = compile_step(step, registry)?;
            Ok(vec![effect])
        }
        
        Choreography::Sequence(steps) => {
            let mut effects = Vec::new();
            for choreo in steps {
                let mut step_effects = compile_choreography(choreo, registry)?;
                effects.append(&mut step_effects);
            }
            Ok(effects)
        }
        
        Choreography::Parallel(choreos) => {
            let mut all_effects = Vec::new();
            for choreo in choreos {
                let mut choreo_effects = compile_choreography(choreo, registry)?;
                all_effects.append(&mut choreo_effects);
            }
            Ok(all_effects)
        }
        
        Choreography::Choice(choreos) => {
            // For now, just compile the first choice
            if let Some(first) = choreos.first() {
                compile_choreography(first, registry)
            } else {
                Ok(vec![])
            }
        }
    }
}

/// Compile a single choreography step to an effect
fn compile_step(
    step: &ChoreographyStep,
    registry: &AgentRegistry,
) -> Result<Effect<(), EffectRow>, CompilerError> {
    match step {
        ChoreographyStep::Send { from, to, message } => {
            // Validate sender has communication capability
            let sender = registry.lookup(from)
                .ok_or_else(|| CompilerError::AgentNotFound(from.to_string()))?;
                
            if !has_communication_capability(&sender) {
                return Err(CompilerError::MissingCapability {
                    agent: from.to_string(),
                    capability: "Communication".to_string(),
                });
            }
            
            // Validate receiver exists
            let _receiver = registry.lookup(to)
                .ok_or_else(|| CompilerError::AgentNotFound(to.to_string()))?;
            
            // Create communication effect
            let channel = format!("{}→{}", from, to);
            let value = message_to_value(message);
            
            Ok(Effect::send(channel, value))
        }
        
        ChoreographyStep::Spawn { parent, agent } => {
            // Validate parent has spawning capability
            let parent_agent = registry.lookup(parent)
                .ok_or_else(|| CompilerError::AgentNotFound(parent.to_string()))?;
                
            if !has_spawning_capability(&parent_agent) {
                return Err(CompilerError::MissingCapability {
                    agent: parent.to_string(),
                    capability: "AgentSpawning".to_string(),
                });
            }
            
            // Create agent spawning effect (as a state write)
            let location = StateLocation(format!("agent_{}", agent.id));
            let value = Value::String(agent.id.to_string());
            
            Ok(Effect::write(location, value))
        }
        
        ChoreographyStep::Parallel(steps) => {
            // Compile all parallel steps
            let mut effects = Vec::new();
            for step in steps {
                let effect = compile_step(step, registry)?;
                effects.push(effect);
            }
            
            // For now, just sequence them (true parallelism would require more complex effect handling)
            if effects.is_empty() {
                Ok(Effect::pure(()))
            } else {
                let mut result = effects.into_iter().next().unwrap();
                // In a true parallel implementation, we'd compose effects in parallel
                Ok(result)
            }
        }
        
        ChoreographyStep::Sequence(steps) => {
            // Compile all sequential steps
            let mut effects = Vec::new();
            for step in steps {
                let effect = compile_step(step, registry)?;
                effects.push(effect);
            }
            
            // Chain effects sequentially
            if effects.is_empty() {
                Ok(Effect::pure(()))
            } else {
                let mut iter = effects.into_iter();
                let mut result = iter.next().unwrap();
                for effect in iter {
                    result = result.then(effect);
                }
                Ok(result)
            }
        }
    }
}

/// Convert a message to a value
fn message_to_value(message: &Message) -> Value {
    match message {
        Message::Text(text) => Value::String(text.clone()),
        Message::Data(value) => value.clone(),
        Message::Request(id, value) => Value::Struct(vec![
            ("id".to_string(), Value::String(id.clone())),
            ("value".to_string(), value.clone())
        ]),
        Message::Response(id, value) => Value::Struct(vec![
            ("id".to_string(), Value::String(id.clone())),
            ("value".to_string(), value.clone())
        ]),
    }
}

/// Check if an agent has communication capability
fn has_communication_capability(agent: &crate::layer3::agent::Agent) -> bool {
    agent.capabilities.iter().any(|cap| {
        cap.allowed_effects.has_effect("comm_send") || 
        cap.allowed_effects.has_effect("communication")
    })
}

/// Check if an agent has spawning capability
fn has_spawning_capability(agent: &crate::layer3::agent::Agent) -> bool {
    agent.capabilities.iter().any(|cap| {
        cap.allowed_effects.has_effect("agent_spawn") ||
        cap.allowed_effects.has_effect("spawning")
    })
}

/// Validate that a step can be executed by the given agents
pub fn validate_step(
    step: &ChoreographyStep,
    registry: &AgentRegistry,
) -> Result<(), CompilerError> {
    match step {
        ChoreographyStep::Send { from, to, .. } => {
            // Check both agents exist and sender has capability
            let sender = registry.lookup(from)
                .ok_or_else(|| CompilerError::AgentNotFound(from.to_string()))?;
            let _receiver = registry.lookup(to)
                .ok_or_else(|| CompilerError::AgentNotFound(to.to_string()))?;
                
            if !has_communication_capability(&sender) {
                return Err(CompilerError::MissingCapability {
                    agent: from.to_string(),
                    capability: "Communication".to_string(),
                });
            }
            
            Ok(())
        }
        
        ChoreographyStep::Spawn { parent, .. } => {
            let parent_agent = registry.lookup(parent)
                .ok_or_else(|| CompilerError::AgentNotFound(parent.to_string()))?;
                
            if !has_spawning_capability(&parent_agent) {
                return Err(CompilerError::MissingCapability {
                    agent: parent.to_string(),
                    capability: "AgentSpawning".to_string(),
                });
            }
            
            Ok(())
        }
        
        ChoreographyStep::Parallel(steps) => {
            for step in steps {
                validate_step(step, registry)?;
            }
            Ok(())
        }
        
        ChoreographyStep::Sequence(steps) => {
            for step in steps {
                validate_step(step, registry)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::agent::Agent;
    use crate::layer3::capability::Capability;
    use crate::layer2::effect::EffectType;
    
    #[test]
    fn test_compile_send() {
        let mut registry = AgentRegistry::new();
        
        // Create Alice with communication capability
        let mut alice = Agent::new("Alice");
        let comm_cap = Capability::new(
            "Communication".to_string(),
            EffectRow::from_effects(vec![
                ("comm_send".to_string(), EffectType::Comm),
            ])
        );
        alice.add_capability(comm_cap);
        
        let bob = Agent::new("Bob");
        
        registry.register(alice).unwrap();
        registry.register(bob).unwrap();
        
        let step = ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello".to_string()),
        };
        
        let choreo = Choreography::Step(step);
        
        let effects = compile_choreography(&choreo, &registry).unwrap();
        assert_eq!(effects.len(), 1);
    }
    
    #[test]
    fn test_missing_capability() {
        let mut registry = AgentRegistry::new();
        
        // Alice without communication capability
        let alice = Agent::new("Alice");
        let bob = Agent::new("Bob");
        
        registry.register(alice).unwrap();
        registry.register(bob).unwrap();
        
        let step = ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello".to_string()),
        };
        
        let choreo = Choreography::Step(step);
        
        let result = compile_choreography(&choreo, &registry);
        assert!(result.is_err());
        
        match result {
            Err(CompilerError::MissingCapability { agent, capability }) => {
                assert_eq!(agent, "Alice");
                assert_eq!(capability, "Communication");
            }
            _ => panic!("Expected MissingCapability error"),
        }
    }
    
    #[test]
    fn test_compile_spawn() {
        let mut registry = AgentRegistry::new();
        
        // Create Alice with spawn capability
        let mut alice = Agent::new("Alice");
        let spawn_cap = Capability::new(
            "Spawn".to_string(),
            EffectRow::from_effects(vec![
                ("agent_spawn".to_string(), EffectType::State),
            ])
        );
        alice.add_capability(spawn_cap);
        
        registry.register(alice).unwrap();
        
        let new_agent = Agent::new("Worker1");
        let step = ChoreographyStep::Spawn {
            parent: AgentId::new("Alice"),
            agent: new_agent,
        };
        
        let choreo = Choreography::Step(step);
        
        let effects = compile_choreography(&choreo, &registry).unwrap();
        assert_eq!(effects.len(), 1);
    }
    
    #[test]
    fn test_compile_sequence() {
        let mut registry = AgentRegistry::new();
        
        // Create agents with capabilities
        let mut alice = Agent::new("Alice");
        let comm_cap = Capability::new(
            "Communication".to_string(),
            EffectRow::from_effects(vec![
                ("comm_send".to_string(), EffectType::Comm),
            ])
        );
        alice.add_capability(comm_cap.clone());
        
        let mut bob = Agent::new("Bob");
        bob.add_capability(comm_cap);
        
        registry.register(alice).unwrap();
        registry.register(bob).unwrap();
        
        let steps = vec![
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Alice"),
                to: AgentId::new("Bob"),
                message: Message::Text("Request".to_string()),
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Alice"),
                message: Message::Text("Response".to_string()),
            }),
        ];
        
        let choreo = Choreography::Sequence(steps);
        
        let effects = compile_choreography(&choreo, &registry).unwrap();
        assert_eq!(effects.len(), 2);
    }
}
