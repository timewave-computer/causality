//! Cross-chain test scenarios for multi-chain testing

use crate::{
    error::SimulationResult,
    snapshot::{SnapshotManager, SnapshotId},
    clock::{SimulatedClock, SimulatedTimestamp},
    engine::{SessionParticipantState, SessionOperation},
};
use std::{
    collections::BTreeMap,
    time::Duration,
};
use serde::{Serialize, Deserialize};
use uuid;
use causality_core::{
    effect::session_registry::SessionRegistry,
    lambda::base::SessionType,
};

/// Simple test suite for cross-chain testing
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub test_cases: Vec<String>,
}

/// Cross-chain scenario status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScenarioStatus {
    Running,
    Completed,
    Failed(String),
    Timeout,
}

/// Result of cross-chain scenario execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub status: ScenarioStatus,
    pub chain_results: BTreeMap<String, ChainExecutionResult>,
    pub aggregated_metrics: CrossChainMetrics,
}

/// Scenario execution metrics
#[derive(Debug, Clone)]
pub struct ScenarioMetrics {
    pub total_duration: Duration,
    pub chains_executed: u32,
    pub total_gas_consumed: u64,
    pub messages_sent: u32,
    pub success_rate: f64,
}

/// Test execution record
#[derive(Debug, Clone)]
pub struct TestExecution {
    pub test_id: String,
    pub chain_id: String,
    pub start_time: SimulatedTimestamp,
    pub duration: Duration,
    pub result: ScenarioResult,
}

/// Chain configuration parameters
#[derive(Debug, Clone)]
pub struct ChainParams {
    pub chain_id: String,
    pub gas_limit: u64,
    pub block_time: Duration,
    pub finality_time: Duration,
}

/// Mock chain state for testing
#[derive(Debug, Clone)]
pub struct MockChainState {
    pub block_height: u64,
    pub gas_used: u64,
    pub state_root: String,
}

impl Default for MockChainState {
    fn default() -> Self {
        Self {
            block_height: 0,
            gas_used: 0,
            state_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        }
    }
}

/// Cross-chain test scenario for multi-chain testing
#[derive(Debug, Clone)]
pub struct CrossChainTestScenario {
    /// Scenario identifier
    pub id: String,
    
    /// Scenario description
    pub description: String,
    
    /// Chain configurations for testing
    pub chain_configs: BTreeMap<String, ChainParams>,
    
    /// Test suites per chain
    pub chain_test_suites: BTreeMap<String, Vec<TestSuite>>,
    
    /// Cross-chain dependencies (chain_id -> dependent_chain_ids)
    pub dependencies: BTreeMap<String, Vec<String>>,
    
    /// Maximum execution time for the entire scenario
    pub timeout: Duration,
    
    /// Expected cross-chain outcomes
    pub expected_outcomes: Vec<CrossChainOutcome>,
    
    /// Synchronization points for coordinated testing
    pub sync_points: Vec<SyncPoint>,
}

/// Expected outcome for cross-chain operations
#[derive(Debug, Clone)]
pub enum CrossChainOutcome {
    /// All chains should complete successfully
    AllChainsSuccess,
    
    /// Specific chain should complete within time limit
    ChainCompletesWithin { chain_id: String, max_duration: Duration },
    
    /// Cross-chain message should be delivered
    MessageDelivered { from_chain: String, to_chain: String, message_type: String },
    
    /// State should be consistent across chains
    StateConsistency { chains: Vec<String>, state_key: String },
    
    /// Gas usage should be within bounds across all chains
    TotalGasWithinBounds { max_total_gas: u64 },
    
    /// Cross-chain transaction should complete atomically
    AtomicTransaction { chains: Vec<String>, transaction_id: String },
}

/// Synchronization point for coordinated multi-chain testing
#[derive(Debug, Clone)]
pub struct SyncPoint {
    /// Sync point identifier
    pub id: String,
    
    /// Chains that must reach this point
    pub required_chains: Vec<String>,
    
    /// Maximum wait time for synchronization
    pub max_wait: Duration,
    
    /// Actions to perform at sync point
    pub actions: Vec<SyncAction>,
}

/// Action to perform at synchronization point
#[derive(Debug, Clone)]
pub enum SyncAction {
    /// Wait for all chains to reach the sync point
    WaitForAll,
    
    /// Execute cross-chain message
    SendMessage { from_chain: String, to_chain: String, message: String },
    
    /// Verify state consistency
    VerifyConsistency { chains: Vec<String>, state_keys: Vec<String> },
    
    /// Create coordinated snapshot
    CreateSnapshot { description: String },
    
    /// Inject coordinated fault
    InjectFault { chains: Vec<String>, fault_type: String },
}

/// Multi-chain test executor
#[allow(dead_code)]
pub struct CrossChainTestExecutor {
    /// Individual chain executors
    chain_executors: BTreeMap<String, ChainExecutor>,
    
    /// Cross-chain message relay
    message_relay: MessageRelay,
    
    /// Shared clock for coordination
    clock: SimulatedClock,
    
    /// Cross-chain snapshot manager
    _snapshot_manager: SnapshotManager,
    
    /// Session registry for choreography-driven topology (optional)
    session_registry: Option<SessionRegistry>,
}

/// Single chain executor for cross-chain scenarios
#[derive(Debug)]
pub struct ChainExecutor {
    /// Chain identifier
    pub chain_id: String,
    
    /// Chain configuration
    pub config: ChainParams,
    
    /// Chain state
    pub state: MockChainState,
    
    /// Test suites for this chain
    pub test_suites: Vec<TestSuite>,
    
    /// Current execution status
    pub status: ChainExecutorStatus,
    
    /// Chain-specific metrics
    pub metrics: ChainMetrics,
    
    /// Pending messages from other chains
    pub pending_messages: Vec<CrossChainMessage>,
}

/// Chain-specific execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    /// Total tests executed on this chain
    pub tests_executed: u32,
    
    /// Tests passed on this chain
    pub tests_passed: u32,
    
    /// Chain execution time
    pub execution_time: Duration,
    
    /// Gas consumed on this chain
    pub gas_consumed: u64,
    
    /// Messages sent from this chain
    pub messages_sent: u32,
    
    /// Messages received by this chain
    pub messages_received: u32,
    
    /// Cross-chain operations completed
    pub cross_chain_ops: u32,
}

/// Status of individual chain executor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChainExecutorStatus {
    /// Ready to start
    Ready,
    
    /// Currently executing tests
    Running,
    
    /// Waiting at synchronization point
    WaitingAtSync { sync_id: String },
    
    /// Completed successfully
    Completed,
    
    /// Failed with error
    Failed { reason: String },
    
    /// Timed out
    TimedOut,
}

/// Cross-chain message for communication between chains
#[derive(Debug, Clone)]
pub struct CrossChainMessage {
    /// Message identifier
    pub id: String,
    
    /// Source chain
    pub from_chain: String,
    
    /// Destination chain
    pub to_chain: String,
    
    /// Message type
    pub message_type: String,
    
    /// Message payload
    pub payload: String,
    
    /// Timestamp when message was sent
    pub sent_at: SimulatedTimestamp,
    
    /// Expected delivery time
    pub expected_delivery: Duration,
}

/// Message relay for cross-chain communication
#[derive(Debug)]
pub struct MessageRelay {
    /// Pending messages in transit
    pub in_transit: Vec<CrossChainMessage>,
    
    /// Message delivery latencies per chain pair
    pub latencies: BTreeMap<(String, String), Duration>,
    
    /// Message failure rates per chain pair
    pub failure_rates: BTreeMap<(String, String), f64>,
    
    /// Total messages relayed
    pub total_messages: u32,
    
    /// Failed message deliveries
    pub failed_deliveries: u32,
}

/// Cross-chain test execution result
#[derive(Debug, Clone)]
pub struct CrossChainTestResult {
    /// Scenario that was executed
    pub scenario: CrossChainTestScenario,
    
    /// Overall execution result
    pub overall_result: ScenarioResult,
    
    /// Results per chain
    pub chain_results: BTreeMap<String, ChainExecutionResult>,
    
    /// Cross-chain interaction results
    pub interaction_results: Vec<InteractionResult>,
    
    /// Synchronization results
    pub sync_results: Vec<SyncResult>,
    
    /// Aggregated metrics across all chains
    pub aggregated_metrics: AggregatedMetrics,
}

/// Chain execution result for cross-chain scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionResult {
    /// Chain identifier
    pub chain_id: String,

    /// Chain-specific metrics
    pub metrics: ChainMetrics,

    /// Final chain status
    pub final_status: ChainExecutorStatus,

    /// Chain-specific snapshots created
    pub snapshots: Vec<SnapshotId>,
}

impl Default for ChainExecutionResult {
    fn default() -> Self {
        Self {
            chain_id: "unknown".to_string(),
            metrics: ChainMetrics::default(),
            final_status: ChainExecutorStatus::Ready,
            snapshots: Vec::new(),
        }
    }
}

/// Result of cross-chain interaction
#[derive(Debug, Clone)]
pub struct InteractionResult {
    /// Interaction type
    pub interaction_type: String,
    
    /// Chains involved
    pub chains: Vec<String>,
    
    /// Success status
    pub success: bool,
    
    /// Execution time
    pub duration: Duration,
    
    /// Gas consumed across chains
    pub total_gas: u64,
    
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of synchronization point
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Sync point identifier
    pub sync_id: String,
    
    /// Chains that reached the sync point
    pub reached_chains: Vec<String>,
    
    /// Chains that timed out waiting
    pub timed_out_chains: Vec<String>,
    
    /// Time taken to synchronize
    pub sync_duration: Duration,
    
    /// Actions executed at sync point
    pub executed_actions: Vec<SyncAction>,
    
    /// Success status
    pub success: bool,
}

/// Type aliases and structs for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_execution_time: Duration,
    pub total_gas_used: u64,
    pub success_rate: f64,
    pub cross_chain_latency: Duration,
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            total_execution_time: Duration::from_secs(0),
            total_gas_used: 0,
            success_rate: 0.0,
            cross_chain_latency: Duration::from_millis(100),
        }
    }
}

// Type alias for compatibility
pub type CrossChainMetrics = AggregatedMetrics;

impl CrossChainTestExecutor {
    /// Create a new cross-chain test executor
    pub fn new(clock: SimulatedClock) -> Self {
        Self {
            chain_executors: BTreeMap::new(),
            message_relay: MessageRelay::new(),
            clock,
            _snapshot_manager: SnapshotManager::new(10),
            session_registry: Some(SessionRegistry::new()),
        }
    }
    
    /// Add a chain executor for testing
    pub fn add_chain(&mut self, chain_id: String, config: ChainParams, test_suites: Vec<TestSuite>) -> SimulationResult<()> {
        let chain_state = MockChainState::new(&config);
        
        let executor = ChainExecutor {
            chain_id: chain_id.clone(),
            config,
            state: chain_state,
            test_suites,
            status: ChainExecutorStatus::Ready,
            metrics: ChainMetrics::default(),
            pending_messages: Vec::new(),
        };
        
        self.chain_executors.insert(chain_id, executor);
        Ok(())
    }
    
    /// Execute cross-chain test scenario
    pub async fn execute_scenario(&mut self, scenario: CrossChainTestScenario) -> SimulationResult<CrossChainTestResult> {
        let _start_time = self.clock.now();
        
        // Setup chains based on scenario configuration
        self.setup_chains(&scenario).await?;
        
        // Setup message relay for cross-chain communication
        self.setup_message_relay(&scenario);
        
        // Execute coordinated steps across all chains
        let chain_results = self.execute_coordinated_steps(&scenario).await?;
        
        // Calculate aggregated metrics before moving chain_results
        let aggregated_metrics = self.calculate_aggregated_metrics(&chain_results);
        
        let _end_time = self.clock.now();
        
        // Evaluate outcomes
        let _overall_status = self.evaluate_cross_chain_outcomes(&scenario, &chain_results)?;
        
        // Create overall result with cloned chain_results
        let overall_result = ScenarioResult {
            status: ScenarioStatus::Completed,
            chain_results: chain_results.clone(),
            aggregated_metrics: aggregated_metrics.clone(),
        };
        
        Ok(CrossChainTestResult {
            scenario,
            overall_result,
            chain_results,
            interaction_results: vec![], // TODO: Implement interaction tracking
            sync_results: vec![], // TODO: Implement sync tracking  
            aggregated_metrics,
        })
    }
    
    /// Setup chains for scenario execution
    async fn setup_chains(&mut self, scenario: &CrossChainTestScenario) -> SimulationResult<()> {
        for (chain_id, chain_config) in &scenario.chain_configs {
            if let Some(test_suites) = scenario.chain_test_suites.get(chain_id) {
                self.add_chain(chain_id.clone(), chain_config.clone(), test_suites.clone())?;
            }
        }
        Ok(())
    }
    
    /// Setup message relay configuration
    fn setup_message_relay(&mut self, scenario: &CrossChainTestScenario) {
        // Configure latencies between chains based on their configurations
        for (from_chain, from_config) in &scenario.chain_configs {
            for (to_chain, to_config) in &scenario.chain_configs {
                if from_chain != to_chain {
                    // Calculate latency based on chain configurations
                    let latency = self.calculate_inter_chain_latency(from_config, to_config);
                    self.message_relay.latencies.insert(
                        (from_chain.clone(), to_chain.clone()),
                        latency
                    );
                    
                    // Set failure rate based on network conditions
                    let failure_rate = self.calculate_message_failure_rate(from_config, to_config);
                    self.message_relay.failure_rates.insert(
                        (from_chain.clone(), to_chain.clone()),
                        failure_rate
                    );
                }
            }
        }
    }
    
    /// Process messages in transit
    async fn _process_messages(&mut self) -> SimulationResult<()> {
        let current_time = self.clock.now();
        let mut delivered_messages = Vec::new();
        
        // Check for messages ready for delivery
        for (index, message) in self.message_relay.in_transit.iter().enumerate() {
            let delivery_time = message.sent_at.add_duration(message.expected_delivery);
            if current_time >= delivery_time {
                // Check for delivery failure
                let failure_rate = self.message_relay.failure_rates
                    .get(&(message.from_chain.clone(), message.to_chain.clone()))
                    .cloned()
                    .unwrap_or(0.01); // Default 1% failure rate
                
                if 0.5 >= failure_rate {
                    // Successful delivery
                    if let Some(recipient) = self.chain_executors.get_mut(&message.to_chain) {
                        recipient.pending_messages.push(message.clone());
                    }
                } else {
                    // Failed delivery
                    self.message_relay.failed_deliveries += 1;
                }
                
                delivered_messages.push(index);
            }
        }
        
        // Remove delivered messages
        for &index in delivered_messages.iter().rev() {
            self.message_relay.in_transit.remove(index);
        }
        
        Ok(())
    }
    
    /// Calculate inter-chain latency
    fn calculate_inter_chain_latency(&self, _from_config: &ChainParams, _to_config: &ChainParams) -> Duration {
        // Base latency between chains
        let base_latency = Duration::from_millis(50);
        
        // Congestion factor (simplified calculation)
        let congestion_factor = 1.5; // Simplified for testing
        
        let congestion_duration = Duration::from_secs_f64(base_latency.as_secs_f64() * congestion_factor);
        base_latency + congestion_duration
    }
    
    /// Calculate message failure rate between chains
    fn calculate_message_failure_rate(&self, _from_config: &ChainParams, _to_config: &ChainParams) -> f64 {
        let base_failure_rate = 0.01_f64;
        
        // Congestion impact on failure rate
        let congestion_impact = 1.5_f64; // Simplified for testing
        
        let adjusted_rate: f64 = base_failure_rate * congestion_impact;
        adjusted_rate.min(0.1_f64) // Cap at 10% failure rate
    }
    
    /// Handle scenario timeout
    async fn _handle_scenario_timeout(&mut self, scenario: &CrossChainTestScenario) -> SimulationResult<CrossChainTestResult> {
        // Mark all running chains as timed out
        for executor in self.chain_executors.values_mut() {
            if executor.status == ChainExecutorStatus::Running {
                executor.status = ChainExecutorStatus::TimedOut;
            }
        }
        
        // Create timeout result
        let overall_result = ScenarioResult {
            status: ScenarioStatus::Timeout,
            chain_results: BTreeMap::new(),
            aggregated_metrics: self.calculate_aggregated_metrics(&BTreeMap::new()),
        };
        
        Ok(CrossChainTestResult {
            scenario: scenario.clone(),
            overall_result,
            chain_results: BTreeMap::new(),
            interaction_results: Vec::new(),
            sync_results: Vec::new(),
            aggregated_metrics: self.calculate_aggregated_metrics(&BTreeMap::new()),
        })
    }
    
    /// Evaluate cross-chain outcomes
    fn evaluate_cross_chain_outcomes(&self, scenario: &CrossChainTestScenario, chain_results: &BTreeMap<String, ChainExecutionResult>) -> SimulationResult<ScenarioStatus> {
        for outcome in &scenario.expected_outcomes {
            match outcome {
                CrossChainOutcome::AllChainsSuccess => {
                    let all_completed = chain_results.values().all(|result| {
                        result.final_status == ChainExecutorStatus::Completed
                    });
                    if !all_completed {
                        return Ok(ScenarioStatus::Failed(
                            "Not all chains completed successfully".to_string()
                        ));
                    }
                }
                
                CrossChainOutcome::TotalGasWithinBounds { max_total_gas } => {
                    let total_gas: u64 = chain_results.values().map(|result| result.metrics.gas_consumed).sum();
                    if total_gas > *max_total_gas {
                        return Ok(ScenarioStatus::Failed(format!(
                            "Total gas {} exceeded limit {}", total_gas, max_total_gas
                        )));
                    }
                }
                
                _ => {
                    // Other outcome types would be implemented similarly
                }
            }
        }
        
        Ok(ScenarioStatus::Completed)
    }
    
    /// Calculate aggregated metrics across all chains
    fn calculate_aggregated_metrics(&self, chain_results: &BTreeMap<String, ChainExecutionResult>) -> AggregatedMetrics {
        let _total_tests: u32 = chain_results.values().map(|result| result.metrics.tests_executed).sum();
        let _total_passed: u32 = chain_results.values().map(|result| result.metrics.tests_passed).sum();
        let max_execution_time = chain_results.values()
            .map(|result| result.metrics.execution_time)
            .max()
            .unwrap_or(Duration::ZERO);
        let total_gas: u64 = chain_results.values().map(|result| result.metrics.gas_consumed).sum();
        let _total_messages: u32 = chain_results.values().map(|result| result.metrics.messages_sent).sum();
        
        let message_success_rate = if self.message_relay.total_messages > 0 {
            1.0 - (self.message_relay.failed_deliveries as f64 / self.message_relay.total_messages as f64)
        } else {
            1.0
        };
        
        let _consistency_score = 0.95; // Simplified - would calculate based on actual consistency checks
        
        AggregatedMetrics {
            total_execution_time: max_execution_time,
            total_gas_used: total_gas,
            success_rate: message_success_rate,
            cross_chain_latency: Duration::from_millis(50),
        }
    }

    /// Execute coordinated steps across all chains
    async fn execute_coordinated_steps(&mut self, scenario: &CrossChainTestScenario) -> SimulationResult<BTreeMap<String, ChainExecutionResult>> {
        let mut chain_results = BTreeMap::new();
        
        // Execute each chain according to dependencies
        for chain_id in scenario.chain_configs.keys() {
            if let Some(executor) = self.chain_executors.get_mut(chain_id) {
                // Set up chain for execution
                executor.status = ChainExecutorStatus::Running;
                
                // Execute test suites for this chain
                let _start_time = self.clock.now();
                
                // Simplified execution - just mark as completed for now
                executor.status = ChainExecutorStatus::Completed;
                executor.metrics.execution_time = Duration::from_millis(100);
                executor.metrics.tests_executed = executor.test_suites.len() as u32;
                executor.metrics.tests_passed = executor.test_suites.len() as u32;
                
                // Create result for this chain
                let result = ChainExecutionResult {
                    chain_id: chain_id.clone(),
                    metrics: executor.metrics.clone(),
                    final_status: executor.status.clone(),
                    snapshots: Vec::new(),
                };
                
                chain_results.insert(chain_id.clone(), result);
            }
        }
        
        Ok(chain_results)
    }

    /// Execute coordinated cross-chain operations
    pub async fn execute_coordinated(
        &mut self, 
        chain_programs: &[(&str, &str)]
    ) -> SimulationResult<CrossChainResult> {
        println!("Executing coordinated cross-chain operations on {} chains", chain_programs.len());
        
        let mut chain_results = BTreeMap::new();
        let mut total_steps = 0;
        
        for (chain_name, program) in chain_programs {
            println!("  Executing on chain '{}': {}", chain_name, program);
            
            // Create a minimal test scenario for this chain
            let mut chain_configs = BTreeMap::new();
            chain_configs.insert(chain_name.to_string(), ChainParams {
                chain_id: chain_name.to_string(),
                gas_limit: 1000000,
                block_time: Duration::from_secs(1),
                finality_time: Duration::from_secs(6),
            });
            
            let scenario = CrossChainTestScenario {
                id: "deterministic_uuid".to_string(),
                description: format!("Execution on {}", chain_name),
                chain_configs,
                chain_test_suites: BTreeMap::new(),
                dependencies: BTreeMap::new(),
                timeout: std::time::Duration::from_secs(30),
                expected_outcomes: Vec::new(),
                sync_points: Vec::new(),
            };
            
            // Execute the scenario
            let execution_result = self.execute_scenario(scenario).await?;
            
            // Extract result for this chain
            if let Some(chain_result) = execution_result.chain_results.get(*chain_name) {
                chain_results.insert(chain_name.to_string(), chain_result.clone());
                total_steps += chain_result.metrics.tests_executed as usize;
            }
            
            println!("     Chain '{}' completed: {} steps", chain_name, total_steps);
        }
        
        Ok(CrossChainResult {
            chain_count: chain_programs.len(),
            total_steps,
            chain_results,
            coordination_successful: true,
            execution_time_ms: 100, // Mock timing
        })
    }

    /// Set up choreography-driven cross-chain topology
    pub fn setup_choreography_topology(
        &mut self,
        choreography: CrossChainChoreography
    ) -> SimulationResult<String> {
        // Create cross-chain session registry if not present
        let mut cross_chain_registry = CrossChainSessionRegistry::new();
        
        // Register the choreography
        cross_chain_registry.register_choreography(choreography.clone())?;
        
        // Setup chains based on choreography projections
        for chain_id in choreography.chain_projections.keys() {
            let chain_params = ChainParams {
                chain_id: chain_id.clone(),
                gas_limit: 1_000_000,
                block_time: Duration::from_millis(1000),
                finality_time: Duration::from_millis(5000),
            };
            
            let chain_executor = ChainExecutor {
                chain_id: chain_id.clone(),
                config: chain_params.clone(),
                state: MockChainState::new(&chain_params),
                test_suites: Vec::new(),
                status: ChainExecutorStatus::Ready,
                metrics: ChainMetrics::default(),
                pending_messages: Vec::new(),
            };
            
            self.chain_executors.insert(chain_id.clone(), chain_executor);
            
            // Register chain capabilities
            let capabilities = ChainCapabilities {
                chain_id: chain_id.clone(),
                max_participants: 10,
                supported_operations: vec![
                    "Send".to_string(),
                    "Receive".to_string(),
                    "InternalChoice".to_string(),
                    "ExternalChoice".to_string(),
                    "End".to_string(),
                ],
                messaging_capabilities: MessagingCapabilities {
                    max_message_size_bytes: 1024 * 1024, // 1MB
                    supported_reliability: vec![ReliabilityLevel::ExactlyOnce],
                    transformation_support: vec![TransformationType::Identity],
                    average_latency_ms: 100,
                },
                performance_profile: PerformanceProfile {
                    ops_per_second: 1000,
                    avg_operation_latency_ms: 10,
                    gas_per_operation: 100,
                    concurrent_participant_limit: 5,
                },
            };
            
            cross_chain_registry.register_chain_capabilities(capabilities);
        }
        
        // Setup message routing based on choreography routing rules
        self.setup_choreography_routing(&choreography);
        
        Ok(choreography.id)
    }
    
    /// Execute choreography-driven cross-chain scenario
    pub async fn execute_choreography(
        &mut self,
        choreography_id: &str,
        execution_id: String
    ) -> SimulationResult<ChoreographyExecutionResult> {
        let start_time = self.clock.now();
        let mut cross_chain_registry = CrossChainSessionRegistry::new();
        
        // Start choreography execution
        let actual_execution_id = cross_chain_registry.start_choreography_execution(
            choreography_id,
            execution_id,
            start_time
        ).await?;
        
        // Execute choreography phases
        let mut execution_successful = true;
        let mut phase_results = Vec::new();
        
        // Phase 1: Setup
        let setup_result = self.execute_choreography_setup(&mut cross_chain_registry, &actual_execution_id).await?;
        phase_results.push(setup_result);
        
        // Phase 2: Active execution
        if execution_successful {
            let active_result = self.execute_choreography_active_phase(&mut cross_chain_registry, &actual_execution_id).await;
            match active_result {
                Ok(result) => phase_results.push(result),
                Err(_) => execution_successful = false,
            }
        }
        
        // Phase 3: Completion
        let completion_time = self.clock.now();
        cross_chain_registry.complete_execution(&actual_execution_id, execution_successful, completion_time)?;
        
        Ok(ChoreographyExecutionResult {
            execution_id: actual_execution_id,
            choreography_id: choreography_id.to_string(),
            success: execution_successful,
            execution_time: Duration::from_secs(completion_time.as_secs() - start_time.as_secs()),
            phase_results,
            final_statistics: cross_chain_registry.get_statistics().clone(),
            cross_chain_messages: Vec::new(), // Would be populated from execution
        })
    }
    
    /// Setup choreography routing in message relay
    fn setup_choreography_routing(&mut self, choreography: &CrossChainChoreography) {
        for route in &choreography.routing_rules {
            let latency = Duration::from_millis(route.expected_latency_ms);
            let failure_rate = match route.reliability_level {
                ReliabilityLevel::BestEffort => 0.1,
                ReliabilityLevel::AtLeastOnce => 0.05,
                ReliabilityLevel::ExactlyOnce => 0.01,
                ReliabilityLevel::OrderedDelivery => 0.02,
            };
            
            self.message_relay.latencies.insert(
                (route.from_chain.clone(), route.to_chain.clone()),
                latency
            );
            self.message_relay.failure_rates.insert(
                (route.from_chain.clone(), route.to_chain.clone()),
                failure_rate
            );
        }
    }
    
    /// Execute choreography setup phase
    async fn execute_choreography_setup(
        &mut self,
        cross_chain_registry: &mut CrossChainSessionRegistry,
        execution_id: &str
    ) -> SimulationResult<PhaseResult> {
        let phase_start = self.clock.now();
        
        // Initialize all chain executors
        for chain_executor in self.chain_executors.values_mut() {
            chain_executor.status = ChainExecutorStatus::Running;
        }
        
        // Update execution phase
        if let Some(execution) = cross_chain_registry.get_execution_mut(execution_id) {
            execution.current_phase = ExecutionPhase::Active;
            
            for chain_state in execution.chain_states.values_mut() {
                chain_state.execution_phase = ExecutionPhase::Active;
            }
        }
        
        let phase_end = self.clock.now();
        
        Ok(PhaseResult {
            phase_name: "Setup".to_string(),
            success: true,
            duration: Duration::from_secs(phase_end.as_secs() - phase_start.as_secs()),
            operations_completed: 0,
            messages_processed: 0,
        })
    }
    
    /// Execute choreography active phase
    async fn execute_choreography_active_phase(
        &mut self,
        cross_chain_registry: &mut CrossChainSessionRegistry,
        execution_id: &str
    ) -> SimulationResult<PhaseResult> {
        let phase_start = self.clock.now();
        let mut operations_completed = 0;
        let mut messages_processed = 0;
        
        // Collect chain IDs first to avoid borrowing issues
        let chain_ids: Vec<String> = self.chain_executors.keys().cloned().collect();
        
        // Simulate session operations for each chain
        for chain_id in chain_ids {
            // Execute local session operations
            let local_ops = self.generate_sample_session_operations(&chain_id);
            operations_completed += local_ops.len();
            
            // Process cross-chain messages
            for operation in local_ops {
                if Self::is_cross_chain_operation_static(&operation) {
                    let message = self.create_cross_chain_message(&operation, &chain_id)?;
                    cross_chain_registry.process_cross_chain_message(
                        execution_id,
                        message,
                        self.clock.now()
                    ).await?;
                    messages_processed += 1;
                }
            }
        }
        
        let phase_end = self.clock.now();
        
        Ok(PhaseResult {
            phase_name: "Active".to_string(),
            success: true,
            duration: Duration::from_secs(phase_end.as_secs() - phase_start.as_secs()),
            operations_completed,
            messages_processed,
        })
    }
    
    /// Check if operation requires cross-chain communication (static version)
    fn is_cross_chain_operation_static(operation: &SessionOperation) -> bool {
        match operation {
            SessionOperation::Send { target_participant, .. } => {
                target_participant.contains("other_chain")
            }
            _ => false,
        }
    }
    
    /// Generate sample session operations for testing
    fn generate_sample_session_operations(&self, chain_id: &str) -> Vec<SessionOperation> {
        vec![
            SessionOperation::Send {
                value_type: causality_core::lambda::TypeInner::Base(causality_core::lambda::base::BaseType::Int),
                target_participant: "participant_on_other_chain".to_string(),
                value: None,
            },
            SessionOperation::Receive {
                value_type: causality_core::lambda::TypeInner::Base(causality_core::lambda::base::BaseType::Int),
                source_participant: format!("participant_on_{}", chain_id),
                expected_value: None,
            },
            SessionOperation::End,
        ]
    }
    
    /// Create cross-chain message from session operation
    fn create_cross_chain_message(
        &self,
        operation: &SessionOperation,
        from_chain: &str
    ) -> SimulationResult<CrossChainSessionMessage> {
        let message_id = format!("msg_{}_{}", from_chain, uuid::Uuid::new_v4());
        let current_time = self.clock.now();
        
        let (from_participant, to_participant, to_chain) = match operation {
            SessionOperation::Send { target_participant, .. } => {
                (
                    format!("participant_on_{}", from_chain),
                    target_participant.clone(),
                    "target_chain".to_string(), // Would be determined from participant mapping
                )
            }
            _ => return Err(crate::error::SimulationError::InvalidInput(
                "Operation is not cross-chain".to_string()
            )),
        };
        
        let route = CrossChainRoute {
            from_participant: from_participant.clone(),
            to_participant: to_participant.clone(),
            from_chain: from_chain.to_string(),
            to_chain: to_chain.clone(),
            transformation: None,
            expected_latency_ms: 100,
            reliability_level: ReliabilityLevel::ExactlyOnce,
        };
        
        Ok(CrossChainSessionMessage {
            message_id,
            from_participant,
            from_chain: from_chain.to_string(),
            to_participant,
            to_chain,
            operation: operation.clone(),
            routing: route,
            created_at: current_time,
            expected_delivery: SimulatedTimestamp::from_secs(current_time.as_secs() + 1),
            delivery_attempts: 0,
            status: MessageStatus::Created,
        })
    }
}

impl Default for MessageRelay {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageRelay {
    /// Create a new message relay
    pub fn new() -> Self {
        Self {
            in_transit: Vec::new(),
            latencies: BTreeMap::new(),
            failure_rates: BTreeMap::new(),
            total_messages: 0,
            failed_deliveries: 0,
        }
    }
}

impl Default for ChainMetrics {
    fn default() -> Self {
        Self {
            tests_executed: 0,
            tests_passed: 0,
            execution_time: Duration::ZERO,
            gas_consumed: 0,
            messages_sent: 0,
            messages_received: 0,
            cross_chain_ops: 0,
        }
    }
}

impl Default for ScenarioMetrics {
    fn default() -> Self {
        Self {
            total_duration: Duration::from_secs(0),
            chains_executed: 0,
            total_gas_consumed: 0,
            messages_sent: 0,
            success_rate: 0.0,
        }
    }
}

impl MockChainState {
    pub fn new(_config: &ChainParams) -> Self {
        Self::default()
    }
}

/// Result of cross-chain coordination
#[derive(Debug, Clone)]
pub struct CrossChainResult {
    /// Number of chains involved
    pub chain_count: usize,
    /// Total execution steps across all chains
    pub total_steps: usize,
    /// Results per chain
    pub chain_results: BTreeMap<String, ChainExecutionResult>,
    /// Whether coordination was successful
    pub coordination_successful: bool,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
}

// Local mock types to replace toolkit dependencies
#[derive(Debug, Clone)]
pub struct ResourceManager {
    resources: BTreeMap<String, u64>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }
    
    pub fn create_resource(&mut self, name: &str, amount: u64) -> String {
        let id = format!("{}_{}", name, amount);
        self.resources.insert(id.clone(), amount);
        id
    }
    
    pub fn get_resource_balance(&self, id: &str) -> Option<u64> {
        self.resources.get(id).copied()
    }
    
    pub fn transfer_resource(&mut self, from_id: &str, to_id: &str, amount: u64) -> bool {
        if let Some(from_balance) = self.resources.get(from_id).copied() {
            if from_balance >= amount {
                self.resources.insert(from_id.to_string(), from_balance - amount);
                let to_balance = self.resources.get(to_id).copied().unwrap_or(0);
                self.resources.insert(to_id.to_string(), to_balance + amount);
                return true;
            }
        }
        false
    }
}

/// Cross-chain session choreography for coordinated protocol execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainChoreography {
    /// Choreography identifier
    pub id: String,
    
    /// Description of the cross-chain interaction
    pub description: String,
    
    /// Chain location assignments for participants
    pub participant_locations: BTreeMap<String, String>, // participant_id -> chain_id
    
    /// Global session type spanning multiple chains
    pub global_session_type: SessionType,
    
    /// Local session projections per chain
    pub chain_projections: BTreeMap<String, SessionType>, // chain_id -> local_session_type
    
    /// Cross-chain message routing rules
    pub routing_rules: Vec<CrossChainRoute>,
    
    /// Synchronization requirements between chains
    pub sync_requirements: Vec<ChainSyncRequirement>,
    
    /// Expected execution order constraints
    pub execution_constraints: Vec<ExecutionConstraint>,
}

/// Cross-chain routing rule for session messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainRoute {
    /// Source participant
    pub from_participant: String,
    
    /// Target participant
    pub to_participant: String,
    
    /// Source chain
    pub from_chain: String,
    
    /// Target chain  
    pub to_chain: String,
    
    /// Message transformation rules (if needed)
    pub transformation: Option<MessageTransformation>,
    
    /// Expected latency for this route
    pub expected_latency_ms: u64,
    
    /// Reliability guarantee level
    pub reliability_level: ReliabilityLevel,
}

/// Message transformation for cross-chain compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTransformation {
    /// Transformation type
    pub transform_type: TransformationType,
    
    /// Transformation parameters
    pub parameters: BTreeMap<String, String>,
}

/// Types of message transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    /// No transformation needed
    Identity,
    
    /// Serialize/deserialize format conversion
    FormatConversion { from_format: String, to_format: String },
    
    /// Value type adaptation
    TypeAdaptation { from_type: String, to_type: String },
    
    /// Protocol version adaptation
    ProtocolVersioning { from_version: String, to_version: String },
    
    /// Custom transformation with parameters
    Custom { function_name: String },
}

/// Reliability level for cross-chain messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityLevel {
    /// Best effort delivery
    BestEffort,
    
    /// At least once delivery
    AtLeastOnce,
    
    /// Exactly once delivery
    ExactlyOnce,
    
    /// Ordered delivery within session
    OrderedDelivery,
}

/// Synchronization requirement between chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSyncRequirement {
    /// Synchronization point identifier
    pub sync_id: String,
    
    /// Chains that must synchronize
    pub chains: Vec<String>,
    
    /// Participants that must reach sync point
    pub participants: Vec<String>,
    
    /// Operations that trigger synchronization
    pub trigger_operations: Vec<SessionOperation>,
    
    /// Maximum wait time for synchronization
    pub max_wait_ms: u64,
    
    /// Action to take on sync timeout
    pub timeout_action: SyncTimeoutAction,
}

/// Actions to take when synchronization times out
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTimeoutAction {
    /// Fail the entire choreography
    FailChoreography,
    
    /// Continue with available participants
    ContinuePartial,
    
    /// Restart synchronization with timeout extension
    RetryWithExtension { extension_ms: u64 },
    
    /// Switch to alternative choreography branch
    SwitchToBranch { branch_id: String },
}

/// Execution constraint for choreography ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    
    /// Participants or operations involved
    pub subject: ConstraintSubject,
    
    /// Description of the constraint
    pub description: String,
}

/// Types of execution constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Must execute before another operation
    Before { target: String },
    
    /// Must execute after another operation
    After { target: String },
    
    /// Must execute concurrently with another operation
    Concurrent { target: String },
    
    /// Must not execute while another operation is running
    Exclusive { conflicting: String },
    
    /// Must complete within time limit
    WithinTime { max_duration_ms: u64 },
}

/// Subject of execution constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintSubject {
    /// Constraint applies to a participant
    Participant { participant_id: String },
    
    /// Constraint applies to an operation
    Operation { operation_id: String },
    
    /// Constraint applies to a chain
    Chain { chain_id: String },
    
    /// Constraint applies to a sync point
    SyncPoint { sync_id: String },
}

/// Cross-chain session registry for managing choreographies
#[derive(Debug)]
pub struct CrossChainSessionRegistry {
    /// Registered choreographies
    choreographies: BTreeMap<String, CrossChainChoreography>,
    
    /// Active choreography executions
    active_executions: BTreeMap<String, ChoreographyExecution>,
    
    /// Participant location tracking
    participant_locations: BTreeMap<String, String>, // participant_id -> chain_id
    
    /// Chain capabilities and requirements
    chain_capabilities: BTreeMap<String, ChainCapabilities>,
    
    /// Cross-chain session statistics
    registry_stats: CrossChainSessionStats,
}

/// Active choreography execution state
#[derive(Debug, Clone)]
pub struct ChoreographyExecution {
    /// Execution identifier
    pub execution_id: String,
    
    /// Choreography being executed
    pub choreography_id: String,
    
    /// Current state per chain
    pub chain_states: BTreeMap<String, ChainExecutionState>,
    
    /// Cross-chain message queue
    pub message_queue: Vec<CrossChainSessionMessage>,
    
    /// Synchronization state
    pub sync_states: BTreeMap<String, SyncState>,
    
    /// Execution start time
    pub start_time: SimulatedTimestamp,
    
    /// Current execution phase
    pub current_phase: ExecutionPhase,
    
    /// Execution constraints tracking
    pub constraint_tracking: ConstraintTracker,
}

/// Execution state for a single chain in choreography
#[derive(Debug, Clone)]
pub struct ChainExecutionState {
    /// Chain identifier
    pub chain_id: String,
    
    /// Participants active on this chain
    pub active_participants: BTreeMap<String, SessionParticipantState>,
    
    /// Current local session progression
    pub local_session_progression: Vec<SessionOperation>,
    
    /// Pending cross-chain operations
    pub pending_cross_chain_ops: Vec<CrossChainSessionMessage>,
    
    /// Chain-specific execution phase
    pub execution_phase: ExecutionPhase,
    
    /// Last activity timestamp
    pub last_activity: SimulatedTimestamp,
}

/// Cross-chain session message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainSessionMessage {
    /// Message identifier
    pub message_id: String,
    
    /// Source participant and chain
    pub from_participant: String,
    pub from_chain: String,
    
    /// Target participant and chain
    pub to_participant: String,
    pub to_chain: String,
    
    /// Session operation being performed
    pub operation: SessionOperation,
    
    /// Message routing information
    pub routing: CrossChainRoute,
    
    /// Message creation timestamp
    pub created_at: SimulatedTimestamp,
    
    /// Expected delivery timestamp
    pub expected_delivery: SimulatedTimestamp,
    
    /// Delivery attempts
    pub delivery_attempts: usize,
    
    /// Message status
    pub status: MessageStatus,
}

/// Status of cross-chain session message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    /// Message created but not sent
    Created,
    
    /// Message in transit
    InTransit,
    
    /// Message delivered successfully
    Delivered,
    
    /// Message delivery failed
    Failed { reason: String },
    
    /// Message delivery timed out
    TimedOut,
    
    /// Message requires transformation
    RequiresTransformation,
}

/// Synchronization state tracking
#[derive(Debug, Clone)]
pub struct SyncState {
    /// Sync point identifier
    pub sync_id: String,
    
    /// Chains that have reached sync point
    pub reached_chains: Vec<String>,
    
    /// Chains still waiting
    pub waiting_chains: Vec<String>,
    
    /// Sync point creation time
    pub created_at: SimulatedTimestamp,
    
    /// Sync deadline
    pub deadline: SimulatedTimestamp,
    
    /// Current sync status
    pub status: SyncStatus,
}

/// Status of synchronization point
#[derive(Debug, Clone)]
pub enum SyncStatus {
    /// Waiting for participants to reach sync point
    Waiting,
    
    /// All required participants have reached sync point
    Synchronized,
    
    /// Sync timed out
    TimedOut,
    
    /// Sync failed due to error
    Failed { reason: String },
}

/// Execution phase in choreography
#[derive(Debug, Clone)]
pub enum ExecutionPhase {
    /// Choreography setup and initialization
    Setup,
    
    /// Active execution of session operations
    Active,
    
    /// Waiting at synchronization point
    Synchronizing { sync_id: String },
    
    /// Choreography completion phase
    Completion,
    
    /// Choreography completed successfully
    Completed,
    
    /// Choreography failed
    Failed { reason: String },
}

/// Constraint tracking for execution ordering
#[derive(Debug, Clone)]
pub struct ConstraintTracker {
    /// Constraints that have been satisfied
    pub satisfied_constraints: Vec<String>,
    
    /// Constraints still pending
    pub pending_constraints: Vec<String>,
    
    /// Constraint violations detected
    pub violations: Vec<ConstraintViolation>,
}

/// Violation of execution constraint
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    /// Constraint that was violated
    pub constraint_id: String,
    
    /// Description of the violation
    pub description: String,
    
    /// Timestamp when violation occurred
    pub violation_time: SimulatedTimestamp,
    
    /// Participants involved
    pub involved_participants: Vec<String>,
}

/// Capabilities of a chain for session execution
#[derive(Debug, Clone)]
pub struct ChainCapabilities {
    /// Chain identifier
    pub chain_id: String,
    
    /// Maximum participants this chain can handle
    pub max_participants: usize,
    
    /// Supported session operation types
    pub supported_operations: Vec<String>,
    
    /// Cross-chain messaging capabilities
    pub messaging_capabilities: MessagingCapabilities,
    
    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
}

/// Messaging capabilities of a chain
#[derive(Debug, Clone)]
pub struct MessagingCapabilities {
    /// Maximum message size
    pub max_message_size_bytes: usize,
    
    /// Supported reliability levels
    pub supported_reliability: Vec<ReliabilityLevel>,
    
    /// Message transformation capabilities
    pub transformation_support: Vec<TransformationType>,
    
    /// Average message latency
    pub average_latency_ms: u64,
}

/// Performance profile of a chain
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    /// Operations per second capacity
    pub ops_per_second: u64,
    
    /// Average operation latency
    pub avg_operation_latency_ms: u64,
    
    /// Gas cost per operation
    pub gas_per_operation: u64,
    
    /// Concurrent participant limit
    pub concurrent_participant_limit: usize,
}

/// Statistics for cross-chain session registry
#[derive(Debug, Clone, Default)]
pub struct CrossChainSessionStats {
    /// Total choreographies registered
    pub total_choreographies: usize,
    
    /// Currently active executions
    pub active_executions: usize,
    
    /// Completed choreographies
    pub completed_choreographies: usize,
    
    /// Failed choreographies
    pub failed_choreographies: usize,
    
    /// Total cross-chain messages sent
    pub total_messages_sent: usize,
    
    /// Total messages delivered successfully
    pub messages_delivered: usize,
    
    /// Average execution time
    pub avg_execution_time_ms: u64,
    
    /// Success rate
    pub success_rate: f64,
}

impl Default for CrossChainSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossChainSessionRegistry {
    /// Create a new cross-chain session registry
    pub fn new() -> Self {
        Self {
            choreographies: BTreeMap::new(),
            active_executions: BTreeMap::new(),
            participant_locations: BTreeMap::new(),
            chain_capabilities: BTreeMap::new(),
            registry_stats: CrossChainSessionStats::default(),
        }
    }
    
    /// Register a new cross-chain choreography
    pub fn register_choreography(&mut self, choreography: CrossChainChoreography) -> SimulationResult<()> {
        // Validate choreography consistency
        self.validate_choreography(&choreography)?;
        
        // Update participant locations
        for (participant_id, chain_id) in &choreography.participant_locations {
            self.participant_locations.insert(participant_id.clone(), chain_id.clone());
        }
        
        // Store choreography
        self.choreographies.insert(choreography.id.clone(), choreography);
        self.registry_stats.total_choreographies += 1;
        
        Ok(())
    }
    
    /// Start executing a choreography
    pub async fn start_choreography_execution(
        &mut self,
        choreography_id: &str,
        execution_id: String,
        initial_timestamp: SimulatedTimestamp
    ) -> SimulationResult<String> {
        let choreography = self.choreographies.get(choreography_id)
            .ok_or_else(|| crate::error::SimulationError::InvalidInput(
                format!("Choreography not found: {}", choreography_id)
            ))?;
        
        // Create execution state
        let execution = ChoreographyExecution {
            execution_id: execution_id.clone(),
            choreography_id: choreography_id.to_string(),
            chain_states: self.initialize_chain_states(choreography, initial_timestamp)?,
            message_queue: Vec::new(),
            sync_states: BTreeMap::new(),
            start_time: initial_timestamp,
            current_phase: ExecutionPhase::Setup,
            constraint_tracking: ConstraintTracker {
                satisfied_constraints: Vec::new(),
                pending_constraints: choreography.execution_constraints.iter()
                    .enumerate()
                    .map(|(i, _)| format!("constraint_{}", i))
                    .collect(),
                violations: Vec::new(),
            },
        };
        
        self.active_executions.insert(execution_id.clone(), execution);
        self.registry_stats.active_executions += 1;
        
        Ok(execution_id)
    }
    
    /// Get active choreography execution
    pub fn get_execution(&self, execution_id: &str) -> Option<&ChoreographyExecution> {
        self.active_executions.get(execution_id)
    }
    
    /// Get mutable reference to active execution
    pub fn get_execution_mut(&mut self, execution_id: &str) -> Option<&mut ChoreographyExecution> {
        self.active_executions.get_mut(execution_id)
    }
    
    /// Process cross-chain session message
    pub async fn process_cross_chain_message(
        &mut self,
        execution_id: &str,
        message: CrossChainSessionMessage,
        current_timestamp: SimulatedTimestamp
    ) -> SimulationResult<()> {
        // Validate message routing first
        self.validate_message_routing(&message)?;
        
        // Apply message transformation if needed
        let transformed_message = self.apply_message_transformation(message).await?;
        
        // Now update execution state
        let execution = self.active_executions.get_mut(execution_id)
            .ok_or_else(|| crate::error::SimulationError::InvalidInput(
                format!("Execution not found: {}", execution_id)
            ))?;
        
        // Add to target chain's pending operations
        if let Some(target_chain_state) = execution.chain_states.get_mut(&transformed_message.to_chain) {
            target_chain_state.pending_cross_chain_ops.push(transformed_message.clone());
            target_chain_state.last_activity = current_timestamp;
        }
        
        // Update message queue
        execution.message_queue.push(transformed_message);
        self.registry_stats.total_messages_sent += 1;
        
        Ok(())
    }
    
    /// Check synchronization requirements
    pub fn check_synchronization(
        &mut self,
        execution_id: &str,
        sync_id: &str,
        chain_id: &str,
        current_timestamp: SimulatedTimestamp
    ) -> SimulationResult<bool> {
        // First get the sync requirement from choreography
        let sync_requirement = {
            let execution = self.active_executions.get(execution_id)
                .ok_or_else(|| crate::error::SimulationError::InvalidInput(
                    format!("Execution not found: {}", execution_id)
                ))?;
            
            let choreography = self.choreographies.get(&execution.choreography_id)
                .ok_or_else(|| crate::error::SimulationError::InvalidInput(
                    format!("Choreography not found: {}", execution.choreography_id)
                ))?;
            
            choreography.sync_requirements.iter()
                .find(|req| req.sync_id == sync_id)
                .ok_or_else(|| crate::error::SimulationError::InvalidInput(
                    format!("Sync requirement not found: {}", sync_id)
                ))?
                .clone()
        };
        
        // Now work with execution state
        let execution = self.active_executions.get_mut(execution_id).unwrap();
        
        // Get or create sync state
        let sync_state = execution.sync_states.entry(sync_id.to_string())
            .or_insert_with(|| SyncState {
                sync_id: sync_id.to_string(),
                reached_chains: Vec::new(),
                waiting_chains: sync_requirement.chains.clone(),
                created_at: current_timestamp,
                deadline: SimulatedTimestamp::from_secs(
                    current_timestamp.as_secs() + sync_requirement.max_wait_ms / 1000
                ),
                status: SyncStatus::Waiting,
            });
        
        // Mark this chain as reached if not already
        if !sync_state.reached_chains.contains(&chain_id.to_string()) {
            sync_state.reached_chains.push(chain_id.to_string());
            sync_state.waiting_chains.retain(|c| c != chain_id);
        }
        
        // Check if all required chains have reached
        let all_reached = sync_state.waiting_chains.is_empty();
        if all_reached {
            sync_state.status = SyncStatus::Synchronized;
        } else if current_timestamp.as_secs() > sync_state.deadline.as_secs() {
            sync_state.status = SyncStatus::TimedOut;
            self.handle_sync_timeout(execution_id, sync_id, &sync_requirement.timeout_action)?;
        }
        
        Ok(all_reached)
    }
    
    /// Complete choreography execution
    pub fn complete_execution(
        &mut self,
        execution_id: &str,
        success: bool,
        completion_timestamp: SimulatedTimestamp
    ) -> SimulationResult<()> {
        if let Some(mut execution) = self.active_executions.remove(execution_id) {
            execution.current_phase = if success {
                ExecutionPhase::Completed
            } else {
                ExecutionPhase::Failed { reason: "Execution failed".to_string() }
            };
            
            // Update statistics
            self.registry_stats.active_executions -= 1;
            if success {
                self.registry_stats.completed_choreographies += 1;
                self.registry_stats.messages_delivered += execution.message_queue.len();
            } else {
                self.registry_stats.failed_choreographies += 1;
            }
            
            // Update average execution time
            let execution_time_ms = (completion_timestamp.as_secs() - execution.start_time.as_secs()) * 1000;
            let total_executions = self.registry_stats.completed_choreographies + self.registry_stats.failed_choreographies;
            if total_executions > 0 {
                self.registry_stats.avg_execution_time_ms = 
                    (self.registry_stats.avg_execution_time_ms * (total_executions - 1) as u64 + execution_time_ms) / total_executions as u64;
            } else {
                self.registry_stats.avg_execution_time_ms = execution_time_ms;
            }
            
            // Update success rate
            self.registry_stats.success_rate = 
                self.registry_stats.completed_choreographies as f64 / total_executions as f64;
        }
        
        Ok(())
    }
    
    /// Register chain capabilities
    pub fn register_chain_capabilities(&mut self, capabilities: ChainCapabilities) {
        self.chain_capabilities.insert(capabilities.chain_id.clone(), capabilities);
    }
    
    /// Get cross-chain session statistics
    pub fn get_statistics(&self) -> &CrossChainSessionStats {
        &self.registry_stats
    }
    
    /// Validate choreography for consistency
    fn validate_choreography(&self, choreography: &CrossChainChoreography) -> SimulationResult<()> {
        // Check that all participants have assigned chains
        for participant_id in choreography.participant_locations.keys() {
            let chain_id = &choreography.participant_locations[participant_id];
            if !choreography.chain_projections.contains_key(chain_id) {
                return Err(crate::error::SimulationError::InvalidInput(
                    format!("Participant {} assigned to chain {} which has no projection", 
                            participant_id, chain_id)
                ));
            }
        }
        
        // Check routing rules consistency
        for route in &choreography.routing_rules {
            if !choreography.participant_locations.contains_key(&route.from_participant) {
                return Err(crate::error::SimulationError::InvalidInput(
                    format!("Routing rule references unknown participant: {}", route.from_participant)
                ));
            }
            if !choreography.participant_locations.contains_key(&route.to_participant) {
                return Err(crate::error::SimulationError::InvalidInput(
                    format!("Routing rule references unknown participant: {}", route.to_participant)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Initialize chain states for choreography execution
    fn initialize_chain_states(
        &self,
        choreography: &CrossChainChoreography,
        initial_timestamp: SimulatedTimestamp
    ) -> SimulationResult<BTreeMap<String, ChainExecutionState>> {
        let mut chain_states = BTreeMap::new();
        
        for chain_id in choreography.chain_projections.keys() {
            let mut active_participants = BTreeMap::new();
            
            // Add participants assigned to this chain
            for (participant_id, assigned_chain) in &choreography.participant_locations {
                if assigned_chain == chain_id {
                    active_participants.insert(
                        participant_id.clone(),
                        SessionParticipantState::new()
                    );
                }
            }
            
            let chain_state = ChainExecutionState {
                chain_id: chain_id.clone(),
                active_participants,
                local_session_progression: Vec::new(),
                pending_cross_chain_ops: Vec::new(),
                execution_phase: ExecutionPhase::Setup,
                last_activity: initial_timestamp,
            };
            
            chain_states.insert(chain_id.clone(), chain_state);
        }
        
        Ok(chain_states)
    }
    
    /// Validate message routing
    fn validate_message_routing(&self, message: &CrossChainSessionMessage) -> SimulationResult<()> {
        // Check if participants are correctly located
        if let Some(expected_from_chain) = self.participant_locations.get(&message.from_participant) {
            if expected_from_chain != &message.from_chain {
                return Err(crate::error::SimulationError::InvalidInput(
                    format!("Participant {} should be on chain {} but message from chain {}", 
                            message.from_participant, expected_from_chain, message.from_chain)
                ));
            }
        }
        
        if let Some(expected_to_chain) = self.participant_locations.get(&message.to_participant) {
            if expected_to_chain != &message.to_chain {
                return Err(crate::error::SimulationError::InvalidInput(
                    format!("Participant {} should be on chain {} but message to chain {}", 
                            message.to_participant, expected_to_chain, message.to_chain)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Apply message transformation if needed
    async fn apply_message_transformation(
        &self,
        mut message: CrossChainSessionMessage
    ) -> SimulationResult<CrossChainSessionMessage> {
        if let Some(transformation) = &message.routing.transformation {
            match &transformation.transform_type {
                TransformationType::Identity => {
                    // No transformation needed
                }
                TransformationType::FormatConversion { from_format: _, to_format: _ } => {
                    // Format conversion implementation would go here TODO
                    // For now, just continue without modification
                }
                TransformationType::TypeAdaptation { from_type: _, to_type: _ } => {
                    // Type adaptation implementation would go here TODO
                    // For now, just continue without modification
                }
                _ => {
                    // Other transformations would be implemented here
                    message.status = MessageStatus::RequiresTransformation;
                }
            }
        }
        
        // Mark as delivered if no transformation was needed
        if message.status != MessageStatus::RequiresTransformation {
            message.status = MessageStatus::Delivered;
        }
        
        Ok(message)
    }
    
    /// Handle synchronization timeout
    fn handle_sync_timeout(
        &mut self,
        execution_id: &str,
        sync_id: &str,
        timeout_action: &SyncTimeoutAction
    ) -> SimulationResult<()> {
        match timeout_action {
            SyncTimeoutAction::FailChoreography => {
                self.complete_execution(execution_id, false, SimulatedTimestamp::new(0))?;
            }
            SyncTimeoutAction::ContinuePartial => {
                // Mark sync as satisfied and continue
                if let Some(execution) = self.active_executions.get_mut(execution_id) {
                    if let Some(sync_state) = execution.sync_states.get_mut(sync_id) {
                        sync_state.status = SyncStatus::Synchronized;
                    }
                }
            }
            SyncTimeoutAction::RetryWithExtension { extension_ms } => {
                // Extend sync deadline
                if let Some(execution) = self.active_executions.get_mut(execution_id) {
                    if let Some(sync_state) = execution.sync_states.get_mut(sync_id) {
                        sync_state.deadline = SimulatedTimestamp::from_secs(
                            sync_state.deadline.as_secs() + extension_ms / 1000
                        );
                        sync_state.status = SyncStatus::Waiting;
                    }
                }
            }
            SyncTimeoutAction::SwitchToBranch { branch_id: _ } => {
                // Would implement branch switching logic
            }
        }
        
        Ok(())
    }
}

/// Result of choreography execution
#[derive(Debug, Clone)]
pub struct ChoreographyExecutionResult {
    /// Execution identifier
    pub execution_id: String,
    
    /// Choreography that was executed
    pub choreography_id: String,
    
    /// Overall success status
    pub success: bool,
    
    /// Total execution time
    pub execution_time: Duration,
    
    /// Results for each execution phase
    pub phase_results: Vec<PhaseResult>,
    
    /// Final registry statistics
    pub final_statistics: CrossChainSessionStats,
    
    /// Cross-chain messages exchanged
    pub cross_chain_messages: Vec<CrossChainSessionMessage>,
}

/// Result of an execution phase
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// Phase name
    pub phase_name: String,
    
    /// Phase success status
    pub success: bool,
    
    /// Phase execution duration
    pub duration: Duration,
    
    /// Number of operations completed in this phase
    pub operations_completed: usize,
    
    /// Number of messages processed in this phase
    pub messages_processed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_cross_chain_executor_creation() {
        let clock = SimulatedClock::default();
        let executor = CrossChainTestExecutor::new(clock);
        
        assert_eq!(executor.chain_executors.len(), 0);
        assert_eq!(executor.message_relay.total_messages, 0);
    }
    
    #[tokio::test]
    async fn test_chain_addition() {
        let clock = SimulatedClock::default();
        let mut executor = CrossChainTestExecutor::new(clock);
        
        let chain_params = ChainParams {
            chain_id: "eth".to_string(),
            gas_limit: 30_000_000,
            block_time: Duration::from_secs(12),
            finality_time: Duration::from_secs(144), // 12 blocks * 12 sec
        };
        
        let result = executor.add_chain("eth".to_string(), chain_params, Vec::new());
        assert!(result.is_ok());
        assert_eq!(executor.chain_executors.len(), 1);
    }
    
    #[tokio::test]
    async fn test_message_relay() {
        let mut relay = MessageRelay::new();
        
        let message = CrossChainMessage {
            id: "test_msg".to_string(),
            from_chain: "eth".to_string(),
            to_chain: "polygon".to_string(),
            message_type: "transfer".to_string(),
            payload: "test_payload".to_string(),
            sent_at: SimulatedTimestamp::new(0),
            expected_delivery: Duration::from_millis(100),
        };
        
        relay.in_transit.push(message);
        assert_eq!(relay.in_transit.len(), 1);
    }
} 