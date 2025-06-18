// Layer 3 to Layer 2 compiler - translates choreographies to outcomes

use crate::layer3::choreography::{Choreography, ChoreographyStep, Message, LocalAction};
use crate::layer3::agent::{AgentId, AgentRegistry};
use crate::layer2::outcome::{Value, StateLocation};
use crate::layer2::effect::{Effect, EffectRow, EffectOp};
use std::marker::PhantomData;

/// Compile a choreography to a sequence of effects
pub fn compile_choreography(
    choreo: &Choreography,
    agents: &AgentRegistry,
) -> Result<Vec<Effect<(), EffectRow>>, CompileError> {
    match choreo {
        Choreography::Step(step) => compile_step(step, agents),
        
        Choreography::Sequence(steps) => {
            let mut effects = Vec::new();
            for sub_choreo in steps {
                let sub_effects = compile_choreography(sub_choreo, agents)?;
                effects.extend(sub_effects);
            }
            Ok(effects)
        }
        
        Choreography::Parallel(branches) => {
            // For now, compile parallel as sequence
            let mut effects = Vec::new();
            for branch in branches {
                let branch_effects = compile_choreography(branch, agents)?;
                effects.extend(branch_effects);
            }
            Ok(effects)
        }
        
        Choreography::Choice { chooser: _, branches } => {
            // For now, just compile the first branch
            if let Some((_, first_branch)) = branches.first() {
                compile_choreography(first_branch, agents)
            } else {
                Ok(vec![])
            }
        }
        
        Choreography::If { condition: _, then_branch, else_branch: _ } => {
            // For now, always take the then branch
            compile_choreography(then_branch, agents)
        }
        
        Choreography::While { .. } => {
            // Don't compile loops for now (could be infinite)
            Err(CompileError::InvalidChoreography("While loops not supported".to_string()))
        }
        
        Choreography::Empty => Ok(vec![]),
    }
}

/// Compile a single choreography step
fn compile_step(
    step: &ChoreographyStep,
    agents: &AgentRegistry,
) -> Result<Vec<Effect<(), EffectRow>>, CompileError> {
    match step {
        ChoreographyStep::Send { from, to, message } => {
            // Check if sender exists
            let sender = agents.get(from)
                .ok_or_else(|| CompileError::AgentNotFound(from.clone()))?;
            
            // Check if receiver exists
            let _receiver = agents.get(to)
                .ok_or_else(|| CompileError::AgentNotFound(to.clone()))?;
            
            // Check if sender has communication capability
            if !sender.can_perform("comm_send") {
                return Err(CompileError::MissingCapability {
                    agent: from.clone(),
                    required: "comm_send".to_string(),
                });
            }
            
            // Create communication effect
            let channel = format!("{}→{}", from.0, to.0);
            let value = Value::String(message_to_value(message));
            
            let effect = Effect::<(), EffectRow>::Do {
                op: EffectOp::CommSend(channel, value),
                cont: Box::new(|_| Effect::Pure(())),
                _phantom: PhantomData,
            };
            
            Ok(vec![effect])
        }
        
        ChoreographyStep::Local { agent, action } => {
            // Check if agent exists
            let _actor = agents.get(agent)
                .ok_or_else(|| CompileError::AgentNotFound(agent.clone()))?;
            
            // Compile local action
            compile_local_action(agent, action)
        }
        
        ChoreographyStep::Spawn { creator, new_agent, agent_type } => {
            // Check if spawner exists
            let spawner = agents.get(creator)
                .ok_or_else(|| CompileError::AgentNotFound(creator.clone()))?;
            
            // Check if spawner has spawn capability
            if !spawner.can_perform("agent_spawn") {
                return Err(CompileError::MissingCapability {
                    agent: creator.clone(),
                    required: "agent_spawn".to_string(),
                });
            }
            
            // Create state update effect for spawn
            let location = StateLocation(format!("agent_spawn_{}_{}", new_agent.0, agent_type));
            let value = Value::String(format!("spawned_by_{}", creator.0));
            
            let effect = Effect::<(), EffectRow>::Do {
                op: EffectOp::StateWrite(location, value),
                cont: Box::new(|_| Effect::Pure(())),
                _phantom: PhantomData,
            };
            
            Ok(vec![effect])
        }
        
        ChoreographyStep::Delegate { from, to, capability } => {
            // Check if delegator exists
            let _delegator = agents.get(from)
                .ok_or_else(|| CompileError::AgentNotFound(from.clone()))?;
            
            // Check if delegate exists
            let _delegate = agents.get(to)
                .ok_or_else(|| CompileError::AgentNotFound(to.clone()))?;
            
            // For now, delegation is just a state update
            let location = StateLocation(format!("delegation_{}_{}", from.0, to.0));
            let value = Value::String(capability.clone());
            
            let effect = Effect::<(), EffectRow>::Do {
                op: EffectOp::StateWrite(location, value),
                cont: Box::new(|_| Effect::Pure(())),
                _phantom: PhantomData,
            };
            
            Ok(vec![effect])
        }
    }
}

/// Compile a local action
fn compile_local_action(
    agent: &AgentId,
    action: &LocalAction,
) -> Result<Vec<Effect<(), EffectRow>>, CompileError> {
    match action {
        LocalAction::UpdateState { key, value } => {
            let location = StateLocation(format!("{}_{}", agent.0, key));
            let state_value = Value::String(value.clone());
            
            let effect = Effect::<(), EffectRow>::Do {
                op: EffectOp::StateWrite(location, state_value),
                cont: Box::new(|_| Effect::Pure(())),
                _phantom: PhantomData,
            };
            
            Ok(vec![effect])
        }
        
        LocalAction::Compute { operation, args } => {
            // For now, computation is a no-op
            println!("Agent {} computes {} with args {:?}", agent.0, operation, args);
            Ok(vec![Effect::Pure(())])
        }
        
        LocalAction::Validate { what } => {
            // For now, validation always succeeds
            println!("Agent {} validates {}", agent.0, what);
            Ok(vec![Effect::Pure(())])
        }
        
        LocalAction::Log(message) => {
            // Log to state
            let location = StateLocation(format!("{}_log", agent.0));
            let log_value = Value::String(message.clone());
            
            let effect = Effect::<(), EffectRow>::Do {
                op: EffectOp::StateWrite(location, log_value),
                cont: Box::new(|_| Effect::Pure(())),
                _phantom: PhantomData,
            };
            
            Ok(vec![effect])
        }
    }
}

/// Convert message to string
fn message_to_value(message: &Message) -> String {
    match message {
        Message::Text(text) => text.clone(),
        Message::Typed { msg_type, value } => format!("{}:{:?}", msg_type, value),
        Message::Request { request_type, payload } => {
            format!("request:{}:{}", request_type, message_to_value(payload))
        }
        Message::Response { response_type, payload } => {
            format!("response:{}:{}", response_type, message_to_value(payload))
        }
    }
}

/// Compilation error types
#[derive(Debug)]
pub enum CompileError {
    /// Agent not found in registry
    AgentNotFound(AgentId),
    
    /// Agent lacks required capability
    MissingCapability {
        agent: AgentId,
        required: String,
    },
    
    /// Invalid choreography
    InvalidChoreography(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::AgentNotFound(id) => write!(f, "Agent not found: {:?}", id),
            CompileError::MissingCapability { agent, required } => {
                write!(f, "Agent {:?} lacks capability: {}", agent, required)
            }
            CompileError::InvalidChoreography(reason) => {
                write!(f, "Invalid choreography: {}", reason)
            }
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Agent, Capability};
    
    #[test]
    fn test_compile_send() {
        let mut registry = AgentRegistry::new();
        
        // Create Alice with communication capability
        let mut alice = Agent::new("Alice");
        let comm_cap = Capability::new(
            "Communication".to_string(),
            EffectRow::from_effects(vec![
                ("comm_send".to_string(), crate::layer2::effect::EffectType::Comm),
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
            Err(CompileError::MissingCapability { agent, required }) => {
                assert_eq!(agent, AgentId::new("Alice"));
                assert_eq!(required, "comm_send");
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
                ("agent_spawn".to_string(), crate::layer2::effect::EffectType::State),
            ])
        );
        alice.add_capability(spawn_cap);
        
        registry.register(alice).unwrap();
        
        let step = ChoreographyStep::Spawn {
            creator: AgentId::new("Alice"),
            new_agent: AgentId::new("Worker1"),
            agent_type: "Worker".to_string(),
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
                ("comm_send".to_string(), crate::layer2::effect::EffectType::Comm),
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
            Choreography::Step(ChoreographyStep::Local {
                agent: AgentId::new("Bob"),
                action: LocalAction::Validate { what: "request".to_string() },
            }),
            Choreography::Step(ChoreographyStep::Send {
                from: AgentId::new("Bob"),
                to: AgentId::new("Alice"),
                message: Message::Text("Response".to_string()),
            }),
        ];
        
        let choreo = Choreography::Sequence(steps);
        
        let effects = compile_choreography(&choreo, &registry).unwrap();
        assert_eq!(effects.len(), 3);
    }
}
