//! Fault injection for resilience testing

use std::collections::BTreeMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use crate::error::SimulationResult;
use crate::engine::SessionOperation;
use causality_core::lambda::base::{SessionType, TypeInner};

/// Types of faults that can be injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    /// Network failures
    NetworkPartition { duration_ms: u64 },
    NetworkLatency { additional_latency_ms: u64 },
    PacketLoss { probability: f64 },
    
    /// Resource failures
    ResourceExhaustion { resource_type: String },
    ResourceDelay { delay_ms: u64 },
    
    /// Effect execution failures
    EffectFailure { probability: f64 },
    EffectTimeout { timeout_ms: u64 },
    
    /// System failures
    ProcessCrash,
    MemoryCorruption { probability: f64 },
    
    /// Time-based failures
    ClockSkew { skew_ms: i64 },
    TimeoutExpiry,
    
    /// Session-specific faults
    SessionMessageLoss { 
        /// Probability of losing a send/receive message
        probability: f64,
        /// Whether to preserve duality (dual operation still occurs)
        preserve_duality: bool,
    },
    SessionMessageReordering {
        /// Number of messages to buffer before reordering
        buffer_size: usize,
        /// Probability of reordering messages in buffer
        reorder_probability: f64,
    },
    SessionProtocolViolation {
        /// Type of protocol violation to inject
        violation_type: SessionViolationType,
        /// Whether to continue after violation or halt
        continue_after_violation: bool,
    },
    SessionDuplicateMessage {
        /// Probability of duplicating a message
        probability: f64,
        /// Number of duplicates to create
        duplicate_count: usize,
    },
    SessionChoiceManipulation {
        /// Force a specific choice in external choice operations
        forced_choice: Option<String>,
        /// Probability of choice manipulation
        probability: f64,
    },
    SessionTypeConfusion {
        /// Inject type mismatches in send/receive operations
        probability: f64,
        /// Replacement type to use instead of expected type
        replacement_type: Option<TypeInner>,
    },
    SessionPartialFailure {
        /// Fail only specific participants in multi-party protocols
        failed_participants: Vec<String>,
        /// Duration of the failure
        duration_ms: u64,
    },
}

/// Configuration for fault injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultConfig {
    pub fault_type: FaultType,
    pub target: String, // Component or operation to target
    pub probability: f64, // Probability of fault occurring (0.0 - 1.0)
    pub duration_ms: Option<u64>, // How long the fault lasts
    pub trigger_condition: Option<String>, // Condition to trigger fault
}

/// Types of session protocol violations that can be injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionViolationType {
    /// Send without corresponding receive
    UnmatchedSend,
    /// Receive without corresponding send  
    UnmatchedReceive,
    /// Wrong choice in external/internal choice
    InvalidChoice,
    /// Premature session termination
    PrematureEnd,
    /// Out-of-order operations
    OutOfOrder,
    /// Type mismatch in communication
    TypeMismatch,
}

/// Session-aware fault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFaultConfig {
    pub fault_type: FaultType,
    pub target_participants: Vec<String>, // Specific participants to target
    pub target_operations: Vec<SessionOperationType>, // Specific operation types to target
    pub probability: f64,
    pub session_context: Option<String>, // Session type context for semantic awareness
    pub preserve_protocol_safety: bool, // Whether to maintain protocol safety properties
}

/// Types of session operations that can be targeted by faults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionOperationType {
    Send,
    Receive,
    InternalChoice,
    ExternalChoice,
    End,
    Any, // Target any operation type
}

/// Result of a session-aware fault injection
#[derive(Debug, Clone)]
pub enum SessionFaultResult {
    /// No fault effect
    NoEffect,
    
    /// Message loss fault
    MessageLoss {
        lost_operation: SessionOperation,
        preserve_duality: bool,
    },
    
    /// Message reordering fault  
    MessageReordering {
        original_operation: SessionOperation,
        buffer_size: usize,
    },
    
    /// Protocol violation fault
    ProtocolViolation {
        violation_type: SessionViolationType,
        affected_operation: SessionOperation,
        continue_execution: bool,
    },
    
    /// Duplicate message fault
    DuplicateMessage {
        original_operation: SessionOperation,
        duplicate_count: usize,
    },
    
    /// Choice manipulation fault
    ChoiceManipulation {
        original_operation: SessionOperation,
        forced_choice: Option<String>,
    },
    
    /// Type confusion fault
    TypeConfusion {
        original_operation: SessionOperation,
        replacement_type: Option<TypeInner>,
    },
    
    /// Partial failure fault
    PartialFailure {
        affected_operation: SessionOperation,
        failure_duration: u64,
    },
}

/// Critical points in session protocols for targeted fault injection
#[derive(Debug, Clone)]
pub enum CriticalPoint {
    /// Communication channel between participants
    CommunicationChannelOpen {
        from: String,
        to: String,
        message_type: TypeInner,
    },
    
    /// Protocol branching point (choice)
    ProtocolBranch {
        choice_type: String, // "internal" or "external"
        chosen_branch: String,
    },
    
    /// Session type state transition
    SessionTypeTransition,
    
    /// Protocol termination point
    ProtocolTermination,
}

/// Enhanced fault statistics including session-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFaultStatistics {
    pub basic_statistics: FaultStatistics,
    pub session_faults_triggered: usize,
    pub protocol_violations_injected: usize,
    pub duality_violations: usize,
    pub message_loss_events: usize,
    pub choice_manipulations: usize,
}

/// Manages fault injection during simulation
#[derive(Debug)]
pub struct FaultInjector {
    active_faults: BTreeMap<String, FaultConfig>,
    fault_history: Vec<FaultEvent>,
    rng: StdRng,
    enabled: bool,
}

/// Record of a fault that was injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultEvent {
    pub fault_id: String,
    pub fault_type: FaultType,
    pub target: String,
    pub timestamp: crate::clock::SimulatedTimestamp,
    pub duration_ms: Option<u64>,
    pub triggered: bool,
}

/// Analysis of a session protocol for fault injection opportunities
#[derive(Debug, Clone)]
pub struct SessionProtocolAnalysis {
    pub session_type: SessionType,
    pub participants: Vec<String>,
    pub communication_points: Vec<CommunicationPoint>,
    pub choice_points: Vec<ChoicePoint>,
    pub termination_points: Vec<TerminationPoint>,
    pub duality_requirements: Vec<DualityRequirement>,
    pub fault_injection_recommendations: Vec<FaultRecommendation>,
}

/// A communication point in the session protocol
#[derive(Debug, Clone)]
pub struct CommunicationPoint {
    pub from: String,
    pub to: String,
    pub operation_type: SessionOperationType,
    pub message_type: TypeInner,
    pub criticality: CriticalityLevel,
}

/// A choice point in the session protocol
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub participant: String,
    pub choice_type: String, // "internal" or "external"
    pub available_branches: Vec<String>,
    pub criticality: CriticalityLevel,
}

/// A termination point in the session protocol
#[derive(Debug, Clone)]
pub struct TerminationPoint {
    pub participants: Vec<String>,
    pub termination_type: String,
    pub criticality: CriticalityLevel,
}

/// A duality requirement between operations
#[derive(Debug, Clone)]
pub struct DualityRequirement {
    pub operation_pair: (String, String), // e.g., ("send", "receive")
    pub participants: (String, String),
    pub type_constraint: TypeInner,
    pub criticality: CriticalityLevel,
}

/// Criticality level for fault injection targeting
#[derive(Debug, Clone)]
pub enum CriticalityLevel {
    High,    // Critical path operations
    Medium,  // Important but not critical
    Low,     // Optional or redundant operations
}

/// A fault injection recommendation
#[derive(Debug, Clone)]
pub struct FaultRecommendation {
    pub fault_type: FaultType,
    pub target: String,
    pub rationale: String,
    pub priority: RecommendationPriority,
}

/// Priority of a fault injection recommendation
#[derive(Debug, Clone)]
pub enum RecommendationPriority {
    High,    // Should definitely test
    Medium,  // Recommended to test
    Low,     // Nice to test if time permits
}

impl FaultInjector {
    /// Create a new fault injector with a random seed
    pub fn new() -> Self {
        Self::with_seed(rand::random())
    }
    
    /// Create a fault injector with a specific seed for deterministic testing
    pub fn with_seed(seed: u64) -> Self {
        Self {
            active_faults: BTreeMap::new(),
            fault_history: Vec::new(),
            rng: StdRng::seed_from_u64(seed),
            enabled: true,
        }
    }
    
    /// Enable or disable fault injection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Add a fault configuration
    pub fn add_fault(&mut self, fault_id: String, config: FaultConfig) -> SimulationResult<()> {
        if config.probability < 0.0 || config.probability > 1.0 {
            return Err(crate::error::SimulationError::FaultInjectionError(
                "Fault probability must be between 0.0 and 1.0".to_string()
            ));
        }
        
        self.active_faults.insert(fault_id, config);
        Ok(())
    }
    
    /// Add a session-aware fault configuration
    pub fn add_session_fault(&mut self, fault_id: String, config: SessionFaultConfig) -> SimulationResult<()> {
        if config.probability < 0.0 || config.probability > 1.0 {
            return Err(crate::error::SimulationError::FaultInjectionError(
                "Session fault probability must be between 0.0 and 1.0".to_string()
            ));
        }
        
        // Convert session fault config to regular fault config for storage
        let regular_config = FaultConfig {
            fault_type: config.fault_type,
            target: config.target_participants.join(","), // Store participants as comma-separated
            probability: config.probability,
            duration_ms: None, // Session faults typically don't have fixed durations
            trigger_condition: config.session_context,
        };
        
        self.active_faults.insert(fault_id, regular_config);
        Ok(())
    }
    
    /// Remove a fault configuration
    pub fn remove_fault(&mut self, fault_id: &str) -> bool {
        self.active_faults.remove(fault_id).is_some()
    }
    
    /// Check if a fault should be triggered for a given target
    pub fn should_trigger_fault(&mut self, target: &str, timestamp: crate::clock::SimulatedTimestamp) -> Option<FaultType> {
        if !self.enabled {
            return None;
        }
        
        // Check all active faults for this target
        for (fault_id, config) in &self.active_faults {
            if config.target == target {
                let random_value: f64 = self.rng.gen();
                if random_value < config.probability {
                    // Record the fault event
                    let event = FaultEvent {
                        fault_id: fault_id.clone(),
                        fault_type: config.fault_type.clone(),
                        target: target.to_string(),
                        timestamp,
                        duration_ms: config.duration_ms,
                        triggered: true,
                    };
                    self.fault_history.push(event);
                    
                    return Some(config.fault_type.clone());
                }
            }
        }
        
        None
    }
    
    /// Inject a specific fault immediately
    pub fn inject_fault(&mut self, target: &str, fault_type: FaultType, timestamp: crate::clock::SimulatedTimestamp) {
        if !self.enabled {
            return;
        }
        
        let event = FaultEvent {
            fault_id: format!("manual_{}", self.fault_history.len()),
            fault_type,
            target: target.to_string(),
            timestamp,
            duration_ms: None,
            triggered: true,
        };
        self.fault_history.push(event);
    }
    
    /// Get the fault history
    pub fn get_fault_history(&self) -> &[FaultEvent] {
        &self.fault_history
    }
    
    /// Clear all faults and history
    pub fn clear(&mut self) {
        self.active_faults.clear();
        self.fault_history.clear();
    }
    
    /// Get statistics about injected faults
    pub fn get_statistics(&self) -> FaultStatistics {
        let total_faults = self.fault_history.len();
        let triggered_faults = self.fault_history.iter().filter(|e| e.triggered).count();
        
        let mut fault_type_counts = BTreeMap::new();
        for event in &self.fault_history {
            if event.triggered {
                let fault_type_name = match &event.fault_type {
                    FaultType::NetworkPartition { .. } => "NetworkPartition",
                    FaultType::NetworkLatency { .. } => "NetworkLatency",
                    FaultType::PacketLoss { .. } => "PacketLoss",
                    FaultType::ResourceExhaustion { .. } => "ResourceExhaustion",
                    FaultType::ResourceDelay { .. } => "ResourceDelay",
                    FaultType::EffectFailure { .. } => "EffectFailure",
                    FaultType::EffectTimeout { .. } => "EffectTimeout",
                    FaultType::ProcessCrash => "ProcessCrash",
                    FaultType::MemoryCorruption { .. } => "MemoryCorruption",
                    FaultType::ClockSkew { .. } => "ClockSkew",
                    FaultType::TimeoutExpiry => "TimeoutExpiry",
                    FaultType::SessionMessageLoss { .. } => "SessionMessageLoss",
                    FaultType::SessionMessageReordering { .. } => "SessionMessageReordering",
                    FaultType::SessionProtocolViolation { .. } => "SessionProtocolViolation",
                    FaultType::SessionDuplicateMessage { .. } => "SessionDuplicateMessage",
                    FaultType::SessionChoiceManipulation { .. } => "SessionChoiceManipulation",
                    FaultType::SessionTypeConfusion { .. } => "SessionTypeConfusion",
                    FaultType::SessionPartialFailure { .. } => "SessionPartialFailure",
                };
                *fault_type_counts.entry(fault_type_name.to_string()).or_insert(0) += 1;
            }
        }
        
        FaultStatistics {
            total_faults,
            triggered_faults,
            fault_type_counts,
        }
    }
    
    /// Get statistics about injected faults including session-specific metrics
    pub fn get_session_statistics(&self) -> SessionFaultStatistics {
        let basic_stats = self.get_statistics();
        
        let mut session_faults_triggered = 0;
        let mut protocol_violations_injected = 0;
        let mut duality_violations = 0;
        let mut message_loss_events = 0;
        let mut choice_manipulations = 0;
        
        for event in &self.fault_history {
            if event.triggered {
                match &event.fault_type {
                    FaultType::SessionMessageLoss { preserve_duality, .. } => {
                        session_faults_triggered += 1;
                        message_loss_events += 1;
                        if !preserve_duality {
                            duality_violations += 1;
                        }
                    }
                    FaultType::SessionMessageReordering { .. } => {
                        session_faults_triggered += 1;
                    }
                    FaultType::SessionProtocolViolation { .. } => {
                        session_faults_triggered += 1;
                        protocol_violations_injected += 1;
                    }
                    FaultType::SessionDuplicateMessage { .. } => {
                        session_faults_triggered += 1;
                    }
                    FaultType::SessionChoiceManipulation { .. } => {
                        session_faults_triggered += 1;
                        choice_manipulations += 1;
                    }
                    FaultType::SessionTypeConfusion { .. } => {
                        session_faults_triggered += 1;
                        duality_violations += 1; // Type confusion violates duality
                    }
                    FaultType::SessionPartialFailure { .. } => {
                        session_faults_triggered += 1;
                    }
                    _ => {} // Non-session faults
                }
            }
        }
        
        SessionFaultStatistics {
            basic_statistics: basic_stats,
            session_faults_triggered,
            protocol_violations_injected,
            duality_violations,
            message_loss_events,
            choice_manipulations,
        }
    }
    
    /// Check if a session operation should be affected by a fault
    pub fn should_trigger_session_fault(
        &mut self, 
        operation: &SessionOperation,
        participant: &str,
        session_context: Option<&SessionType>,
        timestamp: crate::clock::SimulatedTimestamp
    ) -> Option<SessionFaultResult> {
        if !self.enabled {
            return None;
        }
        
        // Analyze the operation and session context
        let operation_type = self.classify_session_operation(operation);
        let critical_communication_points = self.identify_critical_points(operation, session_context);
        
        // Check all active faults that could apply to this operation
        for (fault_id, config) in &self.active_faults {
            if self.is_session_fault_applicable(config, participant, &operation_type, &critical_communication_points) {
                let random_value: f64 = self.rng.gen();
                if random_value < config.probability {
                    let fault_result = Self::generate_session_fault_result(&config.fault_type, operation);
                    
                    // Record the fault event
                    let event = FaultEvent {
                        fault_id: fault_id.clone(),
                        fault_type: config.fault_type.clone(),
                        target: participant.to_string(),
                        timestamp,
                        duration_ms: None,
                        triggered: true,
                    };
                    self.fault_history.push(event);
                    
                    return Some(fault_result);
                }
            }
        }
        
        None
    }
    
    /// Classify a session operation for fault targeting
    fn classify_session_operation(&self, operation: &SessionOperation) -> SessionOperationType {
        match operation {
            SessionOperation::Send { .. } => SessionOperationType::Send,
            SessionOperation::Receive { .. } => SessionOperationType::Receive,
            SessionOperation::InternalChoice { .. } => SessionOperationType::InternalChoice,
            SessionOperation::ExternalChoice { .. } => SessionOperationType::ExternalChoice,
            SessionOperation::End => SessionOperationType::End,
        }
    }
    
    /// Identify critical communication points for targeted fault injection
    fn identify_critical_points(&self, operation: &SessionOperation, session_context: Option<&SessionType>) -> Vec<CriticalPoint> {
        let mut critical_points = Vec::new();
        
        match operation {
            SessionOperation::Send { target_participant, value_type, .. } => {
                critical_points.push(CriticalPoint::CommunicationChannelOpen {
                    from: "current".to_string(),
                    to: target_participant.clone(),
                    message_type: value_type.clone(),
                });
            }
            SessionOperation::Receive { source_participant, value_type, .. } => {
                critical_points.push(CriticalPoint::CommunicationChannelOpen {
                    from: source_participant.clone(),
                    to: "current".to_string(),
                    message_type: value_type.clone(),
                });
            }
            SessionOperation::InternalChoice { chosen_branch, .. } => {
                critical_points.push(CriticalPoint::ProtocolBranch {
                    choice_type: "internal".to_string(),
                    chosen_branch: chosen_branch.clone(),
                });
            }
            SessionOperation::ExternalChoice { available_branches, .. } => {
                critical_points.push(CriticalPoint::ProtocolBranch {
                    choice_type: "external".to_string(),
                    chosen_branch: available_branches.first().map(|(name, _)| name.clone()).unwrap_or_default(),
                });
            }
            SessionOperation::End => {
                critical_points.push(CriticalPoint::ProtocolTermination);
            }
        }
        
        // Add session type context if available
        if let Some(_session_type) = session_context {
            critical_points.push(CriticalPoint::SessionTypeTransition);
        }
        
        critical_points
    }
    
    /// Check if a fault configuration applies to the current operation context
    fn is_session_fault_applicable(
        &self,
        config: &FaultConfig,
        participant: &str,
        operation_type: &SessionOperationType,
        _critical_points: &[CriticalPoint]
    ) -> bool {
        // Check if the participant is targeted
        if !config.target.contains(participant) && config.target != "any" {
            return false;
        }
        
        // Check operation type targeting for session faults
        match &config.fault_type {
            FaultType::SessionMessageLoss { .. } => {
                matches!(operation_type, SessionOperationType::Send | SessionOperationType::Receive)
            }
            FaultType::SessionMessageReordering { .. } => {
                matches!(operation_type, SessionOperationType::Send | SessionOperationType::Receive)
            }
            FaultType::SessionProtocolViolation { .. } => true, // Can apply to any operation
            FaultType::SessionDuplicateMessage { .. } => {
                matches!(operation_type, SessionOperationType::Send)
            }
            FaultType::SessionChoiceManipulation { .. } => {
                matches!(operation_type, SessionOperationType::InternalChoice | SessionOperationType::ExternalChoice)
            }
            FaultType::SessionTypeConfusion { .. } => {
                matches!(operation_type, SessionOperationType::Send | SessionOperationType::Receive)
            }
            FaultType::SessionPartialFailure { failed_participants, .. } => {
                failed_participants.contains(&participant.to_string())
            }
            _ => true, // Non-session faults can apply to any operation
        }
    }
    
    /// Generate the result of a session fault
    fn generate_session_fault_result(fault_type: &FaultType, operation: &SessionOperation) -> SessionFaultResult {
        match fault_type {
            FaultType::SessionMessageLoss { preserve_duality, .. } => {
                SessionFaultResult::MessageLoss {
                    lost_operation: operation.clone(),
                    preserve_duality: *preserve_duality,
                }
            }
            FaultType::SessionMessageReordering { buffer_size, .. } => {
                SessionFaultResult::MessageReordering {
                    original_operation: operation.clone(),
                    buffer_size: *buffer_size,
                }
            }
            FaultType::SessionProtocolViolation { violation_type, continue_after_violation } => {
                SessionFaultResult::ProtocolViolation {
                    violation_type: violation_type.clone(),
                    affected_operation: operation.clone(),
                    continue_execution: *continue_after_violation,
                }
            }
            FaultType::SessionDuplicateMessage { duplicate_count, .. } => {
                SessionFaultResult::DuplicateMessage {
                    original_operation: operation.clone(),
                    duplicate_count: *duplicate_count,
                }
            }
            FaultType::SessionChoiceManipulation { forced_choice, .. } => {
                SessionFaultResult::ChoiceManipulation {
                    original_operation: operation.clone(),
                    forced_choice: forced_choice.clone(),
                }
            }
            FaultType::SessionTypeConfusion { replacement_type, .. } => {
                SessionFaultResult::TypeConfusion {
                    original_operation: operation.clone(),
                    replacement_type: replacement_type.clone(),
                }
            }
            FaultType::SessionPartialFailure { duration_ms, .. } => {
                SessionFaultResult::PartialFailure {
                    affected_operation: operation.clone(),
                    failure_duration: *duration_ms,
                }
            }
            _ => SessionFaultResult::NoEffect, // Non-session faults don't generate session results
        }
    }
}

/// Statistics about fault injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultStatistics {
    pub total_faults: usize,
    pub triggered_faults: usize,
    pub fault_type_counts: BTreeMap<String, usize>,
}

impl Default for FaultInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::SimulatedTimestamp;
    
    #[test]
    fn test_fault_injector_basic() {
        let mut injector = FaultInjector::with_seed(42); // Deterministic seed
        let timestamp = SimulatedTimestamp::from_secs(1000);
        
        // Add a fault with 100% probability
        let config = FaultConfig {
            fault_type: FaultType::EffectFailure { probability: 1.0 },
            target: "test_target".to_string(),
            probability: 1.0,
            duration_ms: Some(5000),
            trigger_condition: None,
        };
        
        injector.add_fault("test_fault".to_string(), config).unwrap();
        
        // Should trigger the fault
        let result = injector.should_trigger_fault("test_target", timestamp);
        assert!(matches!(result, Some(FaultType::EffectFailure { .. })));
        
        // Check statistics
        let stats = injector.get_statistics();
        assert_eq!(stats.triggered_faults, 1);
    }
    
    #[test]
    fn test_fault_injection_disabled() {
        let mut injector = FaultInjector::with_seed(42);
        injector.set_enabled(false);
        
        let config = FaultConfig {
            fault_type: FaultType::ProcessCrash,
            target: "test_target".to_string(),
            probability: 1.0,
            duration_ms: None,
            trigger_condition: None,
        };
        
        injector.add_fault("test_fault".to_string(), config).unwrap();
        
        // Should not trigger when disabled
        let timestamp = SimulatedTimestamp::from_secs(1000);
        let result = injector.should_trigger_fault("test_target", timestamp);
        assert!(result.is_none());
    }
} 