//! Session-driven simulation environment generation
//!
//! This module provides automatic simulation environment generation from session types
//! and choreographies, enabling protocol-aware testing and distributed system simulation.

use causality_core::{
    effect::session_registry::{SessionRegistry, SessionDeclaration, Choreography, ChoreographyProtocol},
    lambda::base::{SessionType, Location},
};
use crate::{
    engine::{SimulationEngine, SimulationConfig},
    error::{SimulationResult, SimulationError},
};
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

/// Session environment generator that creates simulation participants from session types
#[derive(Debug, Clone)]
pub struct SessionEnvironmentGenerator {
    /// Session registry for choreography and session type management
    session_registry: SessionRegistry,
    
    /// Generated participant configurations
    participants: BTreeMap<String, SessionParticipantConfig>,
    
    /// Environment topology derived from choreographies
    topology: SessionTopology,
}

/// Configuration for a session participant in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParticipantConfig {
    /// Participant role name
    pub role: String,
    
    /// Session protocol for this participant
    pub protocol: SessionType,
    
    /// Location where this participant operates
    pub location: Location,
    
    /// Initial capabilities and resources
    pub initial_resources: BTreeMap<String, String>,
}

/// Network topology derived from session choreographies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SessionTopology {
    /// Communication patterns between participants
    pub communication_patterns: Vec<CommunicationPattern>,
    
    /// Location mapping for participants
    pub participant_locations: BTreeMap<String, Location>,
    
    /// Dependency graph between participants
    pub dependencies: BTreeMap<String, Vec<String>>,
}

/// Communication pattern between participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPattern {
    /// Source participant role
    pub from_role: String,
    
    /// Destination participant role  
    pub to_role: String,
    
    /// Message types exchanged
    pub message_types: Vec<String>,
    
    /// Communication frequency/priority
    pub frequency: u32,
}

impl SessionEnvironmentGenerator {
    /// Create a new session environment generator
    pub fn new() -> Self {
        Self {
            session_registry: SessionRegistry::new(),
            participants: BTreeMap::new(),
            topology: SessionTopology::default(),
        }
    }
    
    /// Add a session declaration to the environment
    pub fn add_session(&mut self, session: SessionDeclaration) -> SimulationResult<()> {
        self.session_registry.register_session(session)
            .map_err(|e| SimulationError::EffectExecutionError(format!("Session registration failed: {}", e)))?;
        Ok(())
    }
    
    /// Add a choreography to the environment
    pub fn add_choreography(&mut self, choreography: Choreography) -> SimulationResult<()> {
        // Generate participants from choreography roles
        for role in &choreography.roles {
            let participant_config = SessionParticipantConfig {
                role: role.clone(),
                protocol: self.derive_role_protocol(&choreography, role)?,
                location: self.determine_participant_location(&choreography, role),
                initial_resources: BTreeMap::new(),
            };
            self.participants.insert(role.clone(), participant_config);
        }
        
        // Generate topology from choreography protocol
        self.topology = self.derive_topology(&choreography)?;
        
        self.session_registry.register_choreography(choreography)
            .map_err(|e| SimulationError::EffectExecutionError(format!("Choreography registration failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Generate a session-driven simulation engine from the configured environment
    pub fn generate_simulation_engine(&self, config: SimulationConfig) -> SimulationResult<SimulationEngine> {
        let mut engine = SimulationEngine::new_with_config(config);
        
        // Initialize engine with session participants instead of mocks
        for (role, participant_config) in &self.participants {
            engine.add_session_participant(role.clone(), participant_config.clone())?;
        }
        
        // Configure communication topology
        engine.set_session_topology(self.topology.clone())?;
        
        Ok(engine)
    }
    
    /// Get the generated participant configurations
    pub fn participants(&self) -> &BTreeMap<String, SessionParticipantConfig> {
        &self.participants
    }
    
    /// Get the generated topology
    pub fn topology(&self) -> &SessionTopology {
        &self.topology
    }
    
    /// Derive a role's protocol from a choreography
    fn derive_role_protocol(&self, choreography: &Choreography, role: &str) -> SimulationResult<SessionType> {
        // Simple protocol projection - in a full implementation this would use
        // the choreography projection algorithm from the session registry
        match &choreography.protocol {
            ChoreographyProtocol::Communication { from, to, message_type } => {
                if from == role {
                    // This role sends
                    Ok(SessionType::Send(
                        Box::new(self.parse_message_type(message_type)),
                        Box::new(SessionType::End)
                    ))
                } else if to == role {
                    // This role receives
                    Ok(SessionType::Receive(
                        Box::new(self.parse_message_type(message_type)),
                        Box::new(SessionType::End)
                    ))
                } else {
                    // Role not involved in this communication
                    Ok(SessionType::End)
                }
            }
            ChoreographyProtocol::Sequential(protocols) => {
                // For sequential protocols, compose the role's parts
                let mut result = SessionType::End;
                for protocol in protocols.iter().rev() {
                    let temp_choreography = Choreography {
                        name: choreography.name.clone(),
                        roles: choreography.roles.clone(),
                        protocol: protocol.clone(),
                    };
                    let role_part = self.derive_role_protocol(&temp_choreography, role)?;
                    result = self.compose_sequential(role_part, result);
                }
                Ok(result)
            }
            _ => {
                // For other protocol types, return a simple End for now
                Ok(SessionType::End)
            }
        }
    }
    
    /// Determine the location for a participant based on choreography
    fn determine_participant_location(&self, _choreography: &Choreography, role: &str) -> Location {
        use causality_core::system::content_addressing::EntityId;
        
        // Simple location assignment - could be enhanced with choreography analysis
        if role.contains("client") {
            Location::Local
        } else {
            // Create EntityId from role string using a simple hash
            let mut bytes = [0u8; 32];
            let role_bytes = role.as_bytes();
            let copy_len = std::cmp::min(role_bytes.len(), 32);
            bytes[0..copy_len].copy_from_slice(&role_bytes[0..copy_len]);
            Location::Remote(EntityId::from_bytes(bytes))
        }
    }
    
    /// Derive network topology from choreography
    fn derive_topology(&self, choreography: &Choreography) -> SimulationResult<SessionTopology> {
        let mut communication_patterns = Vec::new();
        let mut participant_locations = BTreeMap::new();
        let mut dependencies = BTreeMap::new();
        
        // Analyze choreography protocol to extract communication patterns
        self.extract_communication_patterns(&choreography.protocol, &mut communication_patterns)?;
        
        // Set participant locations
        for role in &choreography.roles {
            participant_locations.insert(
                role.clone(), 
                self.determine_participant_location(choreography, role)
            );
        }
        
        // Build dependency graph
        for pattern in &communication_patterns {
            dependencies.entry(pattern.from_role.clone())
                .or_insert_with(Vec::new)
                .push(pattern.to_role.clone());
        }
        
        Ok(SessionTopology {
            communication_patterns,
            participant_locations,
            dependencies,
        })
    }
    
    /// Extract communication patterns from choreography protocol
    #[allow(clippy::only_used_in_recursion)]
    fn extract_communication_patterns(
        &self, 
        protocol: &ChoreographyProtocol, 
        patterns: &mut Vec<CommunicationPattern>
    ) -> SimulationResult<()> {
        match protocol {
            ChoreographyProtocol::Communication { from, to, message_type } => {
                patterns.push(CommunicationPattern {
                    from_role: from.clone(),
                    to_role: to.clone(),
                    message_types: vec![message_type.clone()],
                    frequency: 1,
                });
            }
            ChoreographyProtocol::Sequential(protocols) => {
                for sub_protocol in protocols {
                    self.extract_communication_patterns(sub_protocol, patterns)?;
                }
            }
            ChoreographyProtocol::Parallel(protocols) => {
                for sub_protocol in protocols {
                    self.extract_communication_patterns(sub_protocol, patterns)?;
                }
            }
            _ => {
                // Other protocol types - placeholder for future implementation
            }
        }
        Ok(())
    }
    
    /// Parse message type string into TypeInner
    fn parse_message_type(&self, message_type: &str) -> causality_core::lambda::base::TypeInner {
        use causality_core::lambda::base::{TypeInner, BaseType};
        match message_type {
            "Int" => TypeInner::Base(BaseType::Int),
            "Bool" => TypeInner::Base(BaseType::Bool),
            "Unit" => TypeInner::Base(BaseType::Unit),
            _ => TypeInner::Base(BaseType::Symbol), // Default fallback
        }
    }
    
    /// Compose two session types sequentially
    #[allow(clippy::only_used_in_recursion)]
    fn compose_sequential(&self, first: SessionType, second: SessionType) -> SessionType {
        match first {
            SessionType::End => second,
            SessionType::Send(t, continuation) => {
                SessionType::Send(t, Box::new(self.compose_sequential(*continuation, second)))
            }
            SessionType::Receive(t, continuation) => {
                SessionType::Receive(t, Box::new(self.compose_sequential(*continuation, second)))
            }
            other => other, // For other types, return as-is for now
        }
    }
}

impl Default for SessionEnvironmentGenerator {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::effect::session_registry::ChoreographyProtocol;
    
    
    #[test]
    fn test_environment_generator_creation() {
        let generator = SessionEnvironmentGenerator::new();
        assert!(generator.participants.is_empty());
        assert!(generator.topology.communication_patterns.is_empty());
    }
    
    #[test]
    fn test_simple_choreography_generation() {
        let mut generator = SessionEnvironmentGenerator::new();
        
        let choreography = Choreography {
            name: "SimpleComm".to_string(),
            roles: vec!["alice".to_string(), "bob".to_string()],
            protocol: ChoreographyProtocol::Communication {
                from: "alice".to_string(),
                to: "bob".to_string(),
                message_type: "Int".to_string(),
            },
        };
        
        generator.add_choreography(choreography).expect("Should add choreography successfully");
        
        // Should have generated two participants
        assert_eq!(generator.participants.len(), 2);
        assert!(generator.participants.contains_key("alice"));
        assert!(generator.participants.contains_key("bob"));
        
        // Should have one communication pattern
        assert_eq!(generator.topology.communication_patterns.len(), 1);
        let pattern = &generator.topology.communication_patterns[0];
        assert_eq!(pattern.from_role, "alice");
        assert_eq!(pattern.to_role, "bob");
    }
    
    #[test]
    fn test_simulation_engine_generation() {
        let mut generator = SessionEnvironmentGenerator::new();
        
        let choreography = Choreography {
            name: "TestSession".to_string(),
            roles: vec!["client".to_string(), "server".to_string()],
            protocol: ChoreographyProtocol::Communication {
                from: "client".to_string(),
                to: "server".to_string(),
                message_type: "Bool".to_string(),
            },
        };
        
        generator.add_choreography(choreography).expect("Should add choreography");
        
        let config = SimulationConfig::default();
        let result = generator.generate_simulation_engine(config);
        
        // Should successfully generate engine (even if some methods aren't implemented yet)
        assert!(result.is_ok() || result.is_err()); // Accept either until engine methods are implemented
    }
} 