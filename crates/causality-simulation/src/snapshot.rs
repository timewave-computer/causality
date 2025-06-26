//! Snapshot management for simulation state capture and rollback

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::{
    clock::SimulatedTimestamp, 
    error::{SnapshotError, SimulationResult},
    engine::{SessionParticipantState, SessionOperation},
    fault_injection::FaultType,
};
use causality_core::lambda::base::SessionType;

/// Snapshot identifier for simulation checkpoints
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SnapshotId(String);

impl SnapshotId {
    /// Create a new snapshot ID
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    /// Generate a unique snapshot ID based on timestamp
    pub fn generate(timestamp: SimulatedTimestamp) -> Self {
        Self(format!("snapshot_{}", timestamp.as_secs()))
    }
    
    /// Get the inner string value (for testing)
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Execution metrics captured in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub effects_executed: usize,
    pub resources_allocated: usize,
    pub resources_consumed: usize,
    pub total_execution_time: std::time::Duration,
    pub average_effect_time: std::time::Duration,
    pub memory_usage_bytes: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            effects_executed: 0,
            resources_allocated: 0,
            resources_consumed: 0,
            total_execution_time: std::time::Duration::ZERO,
            average_effect_time: std::time::Duration::ZERO,
            memory_usage_bytes: 0,
        }
    }
}

/// Parameters for creating a session snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshotParams {
    pub id: SnapshotId,
    pub timestamp: SimulatedTimestamp,
    pub description: String,
    pub session_participants: BTreeMap<String, SessionParticipantState>,
    pub active_protocols: BTreeMap<String, SessionType>,
    pub protocol_execution_trace: Vec<SessionOperation>,
    pub fault_recovery_context: Option<FaultRecoveryContext>,
}

/// Session-aware snapshot containing protocol state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub id: SnapshotId,
    pub timestamp: SimulatedTimestamp,
    pub description: String,
    pub session_participants: BTreeMap<String, SessionParticipantState>,
    pub active_protocols: BTreeMap<String, SessionType>,
    pub protocol_execution_trace: Vec<SessionOperation>,
    pub fault_recovery_context: Option<FaultRecoveryContext>,
    pub checkpoint_boundaries: Vec<CheckpointBoundary>,
    pub resilience_metrics: ResilienceMetrics,
}

/// Context for fault recovery in session protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultRecoveryContext {
    pub fault_type: FaultType,
    pub affected_participants: Vec<String>,
    pub recovery_strategy: RecoveryStrategy,
    pub fault_injection_time: SimulatedTimestamp,
    pub recovery_attempts: usize,
    pub partial_recovery_state: BTreeMap<String, String>,
}

/// Recovery strategies for different types of session faults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Restart the entire session from the beginning
    FullRestart,
    
    /// Resume from the last checkpoint
    CheckpointRestore,
    
    /// Compensate for the fault and continue
    CompensatingActions {
        compensation_operations: Vec<SessionOperation>,
    },
    
    /// Partial recovery for multi-party protocols
    PartialRecovery {
        failed_participants: Vec<String>,
        substitute_participants: Vec<String>,
    },
    
    /// Protocol adaptation to work around the fault
    ProtocolAdaptation {
        adapted_protocol: SessionType,
        adaptation_reason: String,
    },
}

/// Natural checkpoint boundaries in session protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointBoundary {
    pub boundary_type: CheckpointBoundaryType,
    pub operation_index: usize,
    pub participants_involved: Vec<String>,
    pub protocol_state: BTreeMap<String, String>,
    pub timestamp: SimulatedTimestamp,
}

/// Types of natural checkpoint boundaries in session protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointBoundaryType {
    /// After a send/receive pair completes
    CommunicationComplete {
        sender: String,
        receiver: String,
        message_type: String,
    },
    
    /// After a choice operation resolves
    ChoiceResolved {
        participant: String,
        chosen_branch: String,
    },
    
    /// Before entering a critical section
    CriticalSectionEntry {
        section_name: String,
        participants: Vec<String>,
    },
    
    /// After completing a protocol phase
    PhaseComplete {
        phase_name: String,
        next_phase: Option<String>,
    },
    
    /// Before session termination
    PreTermination {
        terminating_participants: Vec<String>,
    },
}

/// Metrics for session resilience and recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceMetrics {
    pub total_faults_injected: usize,
    pub successful_recoveries: usize,
    pub failed_recoveries: usize,
    pub recovery_time_stats: RecoveryTimeStats,
    pub protocol_adaptation_count: usize,
    pub checkpoint_utilization: CheckpointUtilizationStats,
}

/// Statistics for recovery time performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTimeStats {
    pub total_recovery_time_ms: u64,
    pub average_recovery_time_ms: u64,
    pub min_recovery_time_ms: u64,
    pub max_recovery_time_ms: u64,
    pub recovery_attempts_per_fault: f64,
}

/// Statistics for checkpoint utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointUtilizationStats {
    pub checkpoints_created: usize,
    pub checkpoints_utilized: usize,
    pub checkpoint_hit_rate: f64,
    pub average_operations_between_checkpoints: f64,
}

/// Recovery scenario for testing session resilience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryScenario {
    pub scenario_id: String,
    pub description: String,
    pub session_type: SessionType,
    pub participants: Vec<String>,
    pub injected_fault: FaultType,
    pub recovery_strategy: RecoveryStrategy,
    pub expected_outcomes: Vec<RecoveryOutcome>,
    pub resilience_requirements: Vec<ResilienceRequirement>,
}

/// Expected outcomes from recovery attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryOutcome {
    SessionRestarted,
    ProtocolStateReset,
    ParticipantsReinitialized,
    StateRestored,
    ProtocolResumed,
    CompensationExecuted,
    ProtocolContinued,
    FailedParticipantsReplaced,
    ProtocolAdapted,
    AdaptedProtocolExecuted,
    ChoiceRecovered,
    CommunicationRestored,
}

/// Requirements for resilience testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResilienceRequirement {
    MessageDeliveryGuarantee,
    DualityPreservation,
    MaxRecoveryTime(u64), // milliseconds
    ProtocolSafetyMaintenance,
    ViolationDetection,
    GracefulDegradation,
    PartialFailureTolerance,
    ParticipantSubstitution,
    BasicRecovery,
    ChoiceConsistency,
}

/// Result of resilience testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceTestResult {
    pub scenario_id: String,
    pub success: bool,
    pub execution_time: std::time::Duration,
    pub recovery_attempts: Vec<RecoveryAttempt>,
    pub final_state: FinalRecoveryState,
    pub resilience_metrics: ResilienceMetrics,
}

/// Individual recovery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    pub attempt_number: usize,
    pub strategy_used: RecoveryStrategy,
    pub success: bool,
    #[serde(with = "duration_serde")]
    pub duration: std::time::Duration,
    pub error_message: Option<String>,
    pub checkpoint_used: bool,
    pub compensation_actions_executed: bool,
}

/// Final state after recovery attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalRecoveryState {
    pub participants_active: usize,
    pub protocol_state: String,
    pub execution_progress: f64,
    pub errors_present: bool,
}

/// Serde module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Manages simulation snapshots for debugging and testing
#[derive(Debug)]
pub struct SnapshotManager {
    snapshots: BTreeMap<SnapshotId, SimulationSnapshot>,
    max_snapshots: usize,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: BTreeMap::new(),
            max_snapshots,
        }
    }
    
    /// Parameters for creating a session snapshot
    /// Create a session-aware snapshot with protocol state
    #[allow(clippy::too_many_arguments)]
    pub fn create_session_snapshot(
        &mut self,
        params: SessionSnapshotParams,
    ) -> SimulationResult<()> {
        // Identify natural checkpoint boundaries
        let checkpoint_boundaries = self.identify_checkpoint_boundaries(&params.protocol_execution_trace, params.timestamp);
        
        let session_snapshot = SessionSnapshot {
            id: params.id.clone(),
            timestamp: params.timestamp,
            description: params.description.clone(),
            session_participants: params.session_participants,
            active_protocols: params.active_protocols,
            protocol_execution_trace: params.protocol_execution_trace,
            fault_recovery_context: params.fault_recovery_context,
            checkpoint_boundaries,
            resilience_metrics: ResilienceMetrics::default(),
        };

        // Store as regular snapshot for compatibility
        let snapshot = SimulationSnapshot {
            id: params.id.clone(),
            timestamp: params.timestamp,
            description: format!("Session: {}", session_snapshot.description),
            resource_state: serde_json::to_vec(&session_snapshot)
                .map_err(|e| crate::error::SimulationError::SnapshotError(format!("Session snapshot serialization failed: {}", e)))?,
            effects_log: Vec::new(), // Session snapshots use protocol trace instead
            metrics: PerformanceMetrics::default(),
        };

        // Remove oldest snapshots if needed
        if self.snapshots.len() >= self.max_snapshots && !self.snapshots.contains_key(&params.id) {
            if let Some(oldest_id) = self.find_oldest_snapshot() {
                self.snapshots.remove(&oldest_id);
            }
        }

        self.snapshots.insert(params.id, snapshot);
        Ok(())
    }
    
    /// Restore session state from a session snapshot
    pub fn restore_session_snapshot(&self, id: &SnapshotId) -> Result<SessionSnapshot, SnapshotError> {
        let snapshot = self.snapshots.get(id)
            .ok_or_else(|| SnapshotError::NotFound { id: id.as_str().to_string() })?;
        
        serde_json::from_slice(&snapshot.resource_state)
            .map_err(|e| SnapshotError::DeserializationError { 
                id: id.as_str().to_string(), 
                error: e.to_string() 
            })
    }
    
    /// Generate recovery scenarios from session protocol structure
    pub fn generate_recovery_scenarios(
        &self,
        session_type: &SessionType,
        participants: &[String],
        fault_types: &[FaultType]
    ) -> Result<Vec<RecoveryScenario>, crate::error::SimulationError> {
        let mut scenarios = Vec::new();
        
        // Generate scenarios for each fault type
        for fault_type in fault_types {
            let recovery_strategy = self.determine_recovery_strategy(session_type, fault_type)?;
            
            let scenario = RecoveryScenario {
                scenario_id: format!("recovery_{:?}_{}", fault_type, scenarios.len()),
                description: format!("Recovery testing for {:?} fault", fault_type),
                session_type: session_type.clone(),
                participants: participants.to_vec(),
                injected_fault: fault_type.clone(),
                recovery_strategy: recovery_strategy.clone(),
                expected_outcomes: self.derive_recovery_outcomes(&recovery_strategy),
                resilience_requirements: self.derive_resilience_requirements(session_type, fault_type),
            };
            
            scenarios.push(scenario);
        }
        
        // Add protocol-specific scenarios
        scenarios.extend(self.generate_protocol_specific_scenarios(session_type, participants)?);
        
        Ok(scenarios)
    }
    
    /// Determine appropriate recovery strategy for a fault type and session protocol
    fn determine_recovery_strategy(
        &self,
        session_type: &SessionType,
        fault_type: &FaultType
    ) -> Result<RecoveryStrategy, crate::error::SimulationError> {
        match fault_type {
            FaultType::SessionMessageLoss { preserve_duality, .. } => {
                if *preserve_duality {
                    Ok(RecoveryStrategy::CompensatingActions {
                        compensation_operations: vec![
                            // Would generate appropriate compensation operations based on session type
                        ],
                    })
                } else {
                    Ok(RecoveryStrategy::CheckpointRestore)
                }
            }
            
            FaultType::SessionProtocolViolation { .. } => {
                Ok(RecoveryStrategy::ProtocolAdaptation {
                    adapted_protocol: self.generate_adapted_protocol(session_type)?,
                    adaptation_reason: "Protocol violation recovery".to_string(),
                })
            }
            
            FaultType::SessionPartialFailure { failed_participants, .. } => {
                Ok(RecoveryStrategy::PartialRecovery {
                    failed_participants: failed_participants.clone(),
                    substitute_participants: failed_participants.iter()
                        .map(|p| format!("{}_substitute", p))
                        .collect(),
                })
            }
            
            FaultType::SessionTypeConfusion { .. } => {
                Ok(RecoveryStrategy::FullRestart)
            }
            
            _ => Ok(RecoveryStrategy::CheckpointRestore)
        }
    }
    
    /// Generate an adapted protocol for recovery
    fn generate_adapted_protocol(&self, original: &SessionType) -> Result<SessionType, crate::error::SimulationError> {
        // Simplified protocol adaptation - in a full implementation this would
        // analyze the protocol structure and generate appropriate adaptations
        match original {
            SessionType::Send(value_type, continuation) => {
                // Add retry capability to sends
                Ok(SessionType::InternalChoice(vec![
                    ("retry".to_string(), SessionType::Send(value_type.clone(), continuation.clone())),
                    ("skip".to_string(), *continuation.clone()),
                ]))
            }
            SessionType::Receive(value_type, continuation) => {
                // Add timeout to receives
                Ok(SessionType::ExternalChoice(vec![
                    ("receive".to_string(), SessionType::Receive(value_type.clone(), continuation.clone())),
                    ("timeout".to_string(), *continuation.clone()),
                ]))
            }
            _ => Ok(original.clone()) // No adaptation needed for other types
        }
    }
    
    /// Derive expected recovery outcomes
    fn derive_recovery_outcomes(&self, strategy: &RecoveryStrategy) -> Vec<RecoveryOutcome> {
        match strategy {
            RecoveryStrategy::FullRestart => {
                vec![
                    RecoveryOutcome::SessionRestarted,
                    RecoveryOutcome::ProtocolStateReset,
                    RecoveryOutcome::ParticipantsReinitialized,
                ]
            }
            RecoveryStrategy::CheckpointRestore => {
                vec![
                    RecoveryOutcome::StateRestored,
                    RecoveryOutcome::ProtocolResumed,
                ]
            }
            RecoveryStrategy::CompensatingActions { .. } => {
                vec![
                    RecoveryOutcome::CompensationExecuted,
                    RecoveryOutcome::ProtocolContinued,
                ]
            }
            RecoveryStrategy::PartialRecovery { .. } => {
                vec![
                    RecoveryOutcome::FailedParticipantsReplaced,
                    RecoveryOutcome::ProtocolAdapted,
                    RecoveryOutcome::ProtocolContinued,
                ]
            }
            RecoveryStrategy::ProtocolAdaptation { .. } => {
                vec![
                    RecoveryOutcome::ProtocolAdapted,
                    RecoveryOutcome::AdaptedProtocolExecuted,
                ]
            }
        }
    }
    
    /// Derive resilience requirements based on session type and fault
    fn derive_resilience_requirements(
        &self,
        _session_type: &SessionType,
        fault_type: &FaultType
    ) -> Vec<ResilienceRequirement> {
        match fault_type {
            FaultType::SessionMessageLoss { .. } => {
                vec![
                    ResilienceRequirement::MessageDeliveryGuarantee,
                    ResilienceRequirement::DualityPreservation,
                    ResilienceRequirement::MaxRecoveryTime(1000), // ms
                ]
            }
            FaultType::SessionProtocolViolation { .. } => {
                vec![
                    ResilienceRequirement::ProtocolSafetyMaintenance,
                    ResilienceRequirement::ViolationDetection,
                    ResilienceRequirement::GracefulDegradation,
                ]
            }
            FaultType::SessionPartialFailure { .. } => {
                vec![
                    ResilienceRequirement::PartialFailureTolerance,
                    ResilienceRequirement::ParticipantSubstitution,
                    ResilienceRequirement::MaxRecoveryTime(2000), // ms
                ]
            }
            _ => vec![ResilienceRequirement::BasicRecovery]
        }
    }
    
    /// Generate protocol-specific recovery scenarios
    fn generate_protocol_specific_scenarios(
        &self,
        session_type: &SessionType,
        participants: &[String]
    ) -> Result<Vec<RecoveryScenario>, crate::error::SimulationError> {
        let mut scenarios = Vec::new();
        
        // Add scenarios based on protocol structure
        match session_type {
            SessionType::InternalChoice(_) | SessionType::ExternalChoice(_) => {
                scenarios.push(RecoveryScenario {
                    scenario_id: "choice_recovery".to_string(),
                    description: "Recovery from choice operation failures".to_string(),
                    session_type: session_type.clone(),
                    participants: participants.to_vec(),
                    injected_fault: FaultType::SessionChoiceManipulation { 
                        forced_choice: Some("invalid_choice".to_string()),
                        probability: 1.0 
                    },
                    recovery_strategy: RecoveryStrategy::CompensatingActions {
                        compensation_operations: vec![],
                    },
                    expected_outcomes: vec![RecoveryOutcome::ChoiceRecovered],
                    resilience_requirements: vec![ResilienceRequirement::ChoiceConsistency],
                });
            }
            SessionType::Send(_, _) | SessionType::Receive(_, _) => {
                scenarios.push(RecoveryScenario {
                    scenario_id: "communication_recovery".to_string(),
                    description: "Recovery from communication failures".to_string(),
                    session_type: session_type.clone(),
                    participants: participants.to_vec(),
                    injected_fault: FaultType::SessionMessageLoss { 
                        probability: 1.0, 
                        preserve_duality: true 
                    },
                    recovery_strategy: RecoveryStrategy::CheckpointRestore,
                    expected_outcomes: vec![RecoveryOutcome::CommunicationRestored],
                    resilience_requirements: vec![ResilienceRequirement::MessageDeliveryGuarantee],
                });
            }
            _ => {}
        }
        
        Ok(scenarios)
    }
    
    /// Identify natural checkpoint boundaries in protocol execution
    fn identify_checkpoint_boundaries(
        &self,
        protocol_trace: &[SessionOperation],
        base_timestamp: SimulatedTimestamp
    ) -> Vec<CheckpointBoundary> {
        let mut boundaries = Vec::new();
        
        for (index, operation) in protocol_trace.iter().enumerate() {
            let boundary_type = match operation {
                SessionOperation::Send { target_participant, value_type, .. } => {
                    Some(CheckpointBoundaryType::CommunicationComplete {
                        sender: "current".to_string(),
                        receiver: target_participant.clone(),
                        message_type: format!("{:?}", value_type),
                    })
                }
                SessionOperation::Receive { source_participant, value_type, .. } => {
                    Some(CheckpointBoundaryType::CommunicationComplete {
                        sender: source_participant.clone(),
                        receiver: "current".to_string(),
                        message_type: format!("{:?}", value_type),
                    })
                }
                SessionOperation::InternalChoice { chosen_branch, .. } => {
                    Some(CheckpointBoundaryType::ChoiceResolved {
                        participant: "current".to_string(),
                        chosen_branch: chosen_branch.clone(),
                    })
                }
                SessionOperation::End => {
                    Some(CheckpointBoundaryType::PreTermination {
                        terminating_participants: vec!["current".to_string()],
                    })
                }
                _ => None,
            };
            
            if let Some(boundary_type) = boundary_type {
                boundaries.push(CheckpointBoundary {
                    boundary_type,
                    operation_index: index,
                    participants_involved: vec!["current".to_string()], // Simplified
                    protocol_state: BTreeMap::new(), // Would be populated from actual state
                    timestamp: SimulatedTimestamp::from_secs(base_timestamp.as_secs() + index as u64),
                });
            }
        }
        
        boundaries
    }
    
    /// Execute resilience testing scenario
    pub async fn execute_resilience_scenario(
        &mut self,
        scenario: &RecoveryScenario,
        simulation_engine: &mut crate::engine::SimulationEngine
    ) -> Result<ResilienceTestResult, crate::error::SimulationError> {
        let start_time = std::time::Instant::now();
        
        // Create initial checkpoint
        let initial_checkpoint_id = SnapshotId::new(format!("{}_initial", scenario.scenario_id));
        self.create_session_snapshot(
            SessionSnapshotParams {
                id: initial_checkpoint_id.clone(),
                timestamp: simulation_engine.clock().now(),
                description: "Initial state before fault injection".to_string(),
                session_participants: simulation_engine.session_participants.clone(),
                active_protocols: BTreeMap::new(),
                protocol_execution_trace: Vec::new(),
                fault_recovery_context: None,
            }
        )?;
        
        // Inject fault and attempt recovery
        let recovery_attempts = self.attempt_recovery_with_strategy(
            scenario,
            simulation_engine,
            &initial_checkpoint_id
        ).await?;
        
        let execution_time = start_time.elapsed();
        
        // Calculate metrics before moving recovery_attempts
        let successful_recoveries = recovery_attempts.iter().filter(|a| a.success).count();
        let failed_recoveries = recovery_attempts.iter().filter(|a| !a.success).count();
        let checkpoint_used = recovery_attempts.iter().any(|a| a.checkpoint_used);
        let attempts_count = recovery_attempts.len() as f64;
        let scenario_success = recovery_attempts.iter().any(|attempt| attempt.success);
        
        Ok(ResilienceTestResult {
            scenario_id: scenario.scenario_id.clone(),
            success: scenario_success,
            execution_time,
            recovery_attempts,
            final_state: self.capture_final_state(simulation_engine),
            resilience_metrics: ResilienceMetrics {
                total_faults_injected: 1,
                successful_recoveries,
                failed_recoveries,
                recovery_time_stats: RecoveryTimeStats {
                    total_recovery_time_ms: execution_time.as_millis() as u64,
                    average_recovery_time_ms: execution_time.as_millis() as u64,
                    min_recovery_time_ms: execution_time.as_millis() as u64,
                    max_recovery_time_ms: execution_time.as_millis() as u64,
                    recovery_attempts_per_fault: attempts_count,
                },
                protocol_adaptation_count: if matches!(scenario.recovery_strategy, RecoveryStrategy::ProtocolAdaptation { .. }) { 1 } else { 0 },
                checkpoint_utilization: CheckpointUtilizationStats {
                    checkpoints_created: 1,
                    checkpoints_utilized: if checkpoint_used { 1 } else { 0 },
                    checkpoint_hit_rate: if checkpoint_used { 1.0 } else { 0.0 },
                    average_operations_between_checkpoints: 5.0, // Would be calculated from actual execution
                },
            },
        })
    }
    
    /// Attempt recovery using the specified strategy
    async fn attempt_recovery_with_strategy(
        &mut self,
        scenario: &RecoveryScenario,
        simulation_engine: &mut crate::engine::SimulationEngine,
        checkpoint_id: &SnapshotId
    ) -> Result<Vec<RecoveryAttempt>, crate::error::SimulationError> {
        let mut attempts = Vec::new();
        let max_attempts = 3;
        
        for attempt_number in 1..=max_attempts {
            let attempt_start = std::time::Instant::now();
            
            // Inject the fault
            // (This would integrate with the fault injection system)
            
            // Apply recovery strategy
            let success = match &scenario.recovery_strategy {
                RecoveryStrategy::FullRestart => {
                    // Reset simulation engine to initial state
                    self.reset_to_initial_state(simulation_engine).await?
                }
                RecoveryStrategy::CheckpointRestore => {
                    // Restore from checkpoint
                    self.restore_from_checkpoint(simulation_engine, checkpoint_id).await?
                }
                RecoveryStrategy::CompensatingActions { compensation_operations } => {
                    // Execute compensation operations
                    self.execute_compensating_actions(simulation_engine, compensation_operations).await?
                }
                RecoveryStrategy::PartialRecovery { failed_participants, substitute_participants } => {
                    // Replace failed participants
                    self.replace_failed_participants(simulation_engine, failed_participants, substitute_participants).await?
                }
                RecoveryStrategy::ProtocolAdaptation { adapted_protocol, .. } => {
                    // Adapt protocol and continue
                    self.adapt_protocol_and_continue(simulation_engine, adapted_protocol).await?
                }
            };
            
            let attempt_duration = attempt_start.elapsed();
            
            attempts.push(RecoveryAttempt {
                attempt_number,
                strategy_used: scenario.recovery_strategy.clone(),
                success,
                duration: attempt_duration,
                error_message: if success { None } else { Some("Recovery failed".to_string()) },
                checkpoint_used: matches!(scenario.recovery_strategy, RecoveryStrategy::CheckpointRestore),
                compensation_actions_executed: matches!(scenario.recovery_strategy, RecoveryStrategy::CompensatingActions { .. }),
            });
            
            if success {
                break; // Recovery successful, no need for more attempts
            }
        }
        
        Ok(attempts)
    }
    
    /// Reset simulation engine to initial state
    async fn reset_to_initial_state(
        &self,
        simulation_engine: &mut crate::engine::SimulationEngine
    ) -> Result<bool, crate::error::SimulationError> {
        // Reset program counter and clear session participants
        simulation_engine.pc = 0;
        simulation_engine.session_participants.clear();
        simulation_engine.effects_log.clear();
        Ok(true)
    }
    
    /// Restore simulation from checkpoint
    async fn restore_from_checkpoint(
        &self,
        _simulation_engine: &mut crate::engine::SimulationEngine,
        _checkpoint_id: &SnapshotId
    ) -> Result<bool, crate::error::SimulationError> {
        // Would restore from the specified checkpoint
        // For now, return success as placeholder
        Ok(true)
    }
    
    /// Execute compensating actions
    async fn execute_compensating_actions(
        &self,
        _simulation_engine: &mut crate::engine::SimulationEngine,
        _compensation_operations: &[SessionOperation]
    ) -> Result<bool, crate::error::SimulationError> {
        // Would execute each compensation operation
        // For now, return success as placeholder
        Ok(true)
    }
    
    /// Replace failed participants with substitutes
    async fn replace_failed_participants(
        &self,
        simulation_engine: &mut crate::engine::SimulationEngine,
        failed_participants: &[String],
        substitute_participants: &[String]
    ) -> Result<bool, crate::error::SimulationError> {
        // Remove failed participants and add substitutes
        for failed in failed_participants {
            simulation_engine.session_participants.remove(failed);
        }
        
        for (index, substitute) in substitute_participants.iter().enumerate() {
            if let Some(failed) = failed_participants.get(index) {
                // Create substitute with similar state to failed participant
                if let Some(original_state) = simulation_engine.session_participants.get(failed) {
                    simulation_engine.session_participants.insert(
                        substitute.clone(),
                        original_state.clone()
                    );
                }
            }
        }
        
        Ok(true)
    }
    
    /// Adapt protocol and continue execution
    async fn adapt_protocol_and_continue(
        &self,
        _simulation_engine: &mut crate::engine::SimulationEngine,
        _adapted_protocol: &SessionType
    ) -> Result<bool, crate::error::SimulationError> {
        // Would update the session type and continue execution
        // For now, return success as placeholder
        Ok(true)
    }
    
    /// Capture final state after recovery attempt
    fn capture_final_state(&self, simulation_engine: &crate::engine::SimulationEngine) -> FinalRecoveryState {
        FinalRecoveryState {
            participants_active: simulation_engine.session_participants.len(),
            protocol_state: "active".to_string(), // Would extract actual protocol state
            execution_progress: simulation_engine.pc as f64 / 100.0, // Use a fixed denominator since program field is private
            errors_present: false, // Would check for actual errors
        }
    }
    
    /// Create a snapshot of the current simulation state (standard method)
    pub fn create_snapshot(
        &mut self,
        id: SnapshotId,
        timestamp: SimulatedTimestamp,
        description: String,
        _resource_heap: &causality_core::ResourceManager, // TODO: Add proper serialization
        effects_log: Vec<EffectExecution>,
        metrics: PerformanceMetrics,
    ) -> SimulationResult<()> {
        // TODO: Replace with proper ResourceManager serialization when serde support is added
        let resource_state = vec![]; // Placeholder serialized state
        
        let snapshot = SimulationSnapshot {
            id: id.clone(),
            timestamp,
            description,
            resource_state,
            effects_log,
            metrics,
        };
        
        // Remove oldest snapshots if we exceed the limit
        if self.snapshots.len() >= self.max_snapshots && !self.snapshots.contains_key(&id) {
            if let Some(oldest_id) = self.find_oldest_snapshot() {
                self.snapshots.remove(&oldest_id);
            }
        }
        
        self.snapshots.insert(id, snapshot);
        Ok(())
    }
    
    /// Restore simulation state from a snapshot (standard method)
    pub fn restore_snapshot(&self, id: &SnapshotId) -> Result<(causality_core::ResourceManager, Vec<EffectExecution>, PerformanceMetrics), SnapshotError> {
        let snapshot = self.snapshots.get(id)
            .ok_or_else(|| SnapshotError::NotFound { id: id.as_str().to_string() })?;
        
        // TODO: Replace with proper ResourceManager deserialization when serde support is added
        let resource_heap = causality_core::ResourceManager::new(); // Placeholder new heap
        
        Ok((resource_heap, snapshot.effects_log.clone(), snapshot.metrics.clone()))
    }
    
    /// Get information about a snapshot without restoring it
    pub fn get_snapshot_info(&self, id: &SnapshotId) -> Option<&SimulationSnapshot> {
        self.snapshots.get(id)
    }
    
    /// List all available snapshots
    pub fn list_snapshots(&self) -> Vec<&SnapshotId> {
        self.snapshots.keys().collect()
    }
    
    /// Delete a snapshot
    pub fn delete_snapshot(&mut self, id: &SnapshotId) -> bool {
        self.snapshots.remove(id).is_some()
    }
    
    /// Clear all snapshots
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear()
    }
    
    /// Find the oldest snapshot by timestamp
    fn find_oldest_snapshot(&self) -> Option<SnapshotId> {
        self.snapshots
            .values()
            .min_by_key(|snapshot| snapshot.timestamp)
            .map(|snapshot| snapshot.id.clone())
    }
    
    /// Get a snapshot by its ID
    pub fn get_snapshot(&self, id: &SnapshotId) -> Option<&SimulationSnapshot> {
        self.snapshots.get(id)
    }
    
    /// Create a checkpoint with arbitrary data
    pub fn create_checkpoint<T>(
        &mut self, 
        checkpoint_id: &str, 
        checkpoint_name: &str, 
        data: T
    ) -> Result<(), crate::error::SimulationError> 
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + 'static,
    {
        let serialized = serde_json::to_string(&data)
            .map_err(|e| crate::error::SimulationError::SnapshotError(format!("Serialization failed: {}", e)))?;
            
        let snapshot = SimulationSnapshot {
            id: SnapshotId::new(checkpoint_id.to_string()),
            timestamp: SimulatedTimestamp::new(0), // Use default timestamp
            description: checkpoint_name.to_string(),
            resource_state: serialized.into_bytes(), // Store serialized data as resource state
            effects_log: Vec::new(), // Empty for checkpoints
            metrics: PerformanceMetrics::default(),
        };
        
        self.snapshots.insert(SnapshotId::new(checkpoint_id.to_string()), snapshot);
        Ok(())
    }
    
    /// Get checkpoint data
    pub fn get_checkpoint<T>(&self, checkpoint_id: &str) -> Result<T, crate::error::SimulationError>
    where
        T: serde::de::DeserializeOwned,
    {
        let snapshot = self.snapshots
            .get(&SnapshotId::new(checkpoint_id.to_string()))
            .ok_or_else(|| crate::error::SimulationError::SnapshotError("Checkpoint not found".to_string()))?;
            
        let data_str = String::from_utf8(snapshot.resource_state.clone())
            .map_err(|e| crate::error::SimulationError::SnapshotError(format!("UTF-8 conversion failed: {}", e)))?;
            
        serde_json::from_str(&data_str)
            .map_err(|e| crate::error::SimulationError::SnapshotError(format!("Deserialization failed: {}", e)))
    }
    
    /// Helper method to calculate checksum
    fn _calculate_checksum(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new(10) // Default to keeping 10 snapshots
    }
}

/// Default implementations for metrics
impl Default for ResilienceMetrics {
    fn default() -> Self {
        Self {
            total_faults_injected: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            recovery_time_stats: RecoveryTimeStats::default(),
            protocol_adaptation_count: 0,
            checkpoint_utilization: CheckpointUtilizationStats::default(),
        }
    }
}

impl Default for RecoveryTimeStats {
    fn default() -> Self {
        Self {
            total_recovery_time_ms: 0,
            average_recovery_time_ms: 0,
            min_recovery_time_ms: 0,
            max_recovery_time_ms: 0,
            recovery_attempts_per_fault: 0.0,
        }
    }
}

impl Default for CheckpointUtilizationStats {
    fn default() -> Self {
        Self {
            checkpoints_created: 0,
            checkpoints_utilized: 0,
            checkpoint_hit_rate: 0.0,
            average_operations_between_checkpoints: 0.0,
        }
    }
}

/// Simulation snapshot for standard checkpointing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    pub id: SnapshotId,
    pub timestamp: SimulatedTimestamp,
    pub description: String,
    pub resource_state: Vec<u8>, // Serialized state placeholder
    pub effects_log: Vec<EffectExecution>,
    pub metrics: PerformanceMetrics,
}

/// Effect execution record for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectExecution {
    pub effect_id: String,
    pub effect_expr: String, // Serialized EffectExpr for debugging
    pub start_time: SimulatedTimestamp,
    pub end_time: Option<SimulatedTimestamp>,
    pub result: ExecutionResult,
    pub resources_consumed: Vec<String>,
    pub resources_produced: Vec<String>,
}

/// Execution result for effect operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success,
    Failed { error: String },
    Timeout,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_id_generation() {
        let timestamp = SimulatedTimestamp::from_secs(1234567890);
        let id = SnapshotId::generate(timestamp);
        assert_eq!(id.as_str(), "snapshot_1234567890");
    }
    
    #[test]
    fn test_snapshot_manager_basic() {
        let mut manager = SnapshotManager::new(2);
        let id1 = SnapshotId::new("test1".to_string());
        
        // Initially empty
        assert_eq!(manager.list_snapshots().len(), 0);
        
        // Create snapshots
        let resource_heap = causality_core::ResourceManager::new();
        let timestamp = SimulatedTimestamp::from_secs(1000);
        let metrics = PerformanceMetrics::default();
        
        manager.create_snapshot(
            id1.clone(),
            timestamp,
            "Test snapshot 1".to_string(),
            &resource_heap,
            vec![],
            metrics.clone(),
        ).unwrap();
        
        assert_eq!(manager.list_snapshots().len(), 1);
        assert!(manager.get_snapshot_info(&id1).is_some());
    }
    
    #[test]
    fn test_snapshot_id_creation() {
        let id1 = SnapshotId::new("test1".to_string());
        let id2 = SnapshotId::new("test2".to_string());
        
        assert_ne!(id1, id2);
        assert_eq!(id1.as_str(), "test1");
        assert_eq!(id2.as_str(), "test2");
    }
} 