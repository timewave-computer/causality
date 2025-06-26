//! TEG (Temporal Effect Graph) Executor
//!
//! This module implements dynamic orchestration for TEGs with work stealing,
//! load balancing, and adaptive scheduling for efficient parallel execution.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::{Duration, Instant};
use causality_core::effect::{
    TemporalEffectGraph, EffectNode, NodeId, NodeStatus, TegResult, ExecutionStats,
};
use causality_core::lambda::base::Value;
use crate::{RuntimeResult, Interpreter, RuntimeContext};
use serde_json;

/// Configuration for TEG execution
#[derive(Debug, Clone)]
pub struct TegExecutorConfig {
    /// Number of worker threads
    pub worker_count: usize,
    
    /// Maximum time to wait for work before stealing
    pub steal_timeout_ms: u64,
    
    /// Minimum work items before load balancing kicks in
    pub load_balance_threshold: usize,
    
    /// Maximum execution time per node (timeout)
    pub node_timeout_ms: u64,
    
    /// Enable adaptive scheduling based on execution history
    pub adaptive_scheduling: bool,
}

impl Default for TegExecutorConfig {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get().max(1),
            steal_timeout_ms: 100,
            load_balance_threshold: 4,
            node_timeout_ms: 30000, // 30 seconds
            adaptive_scheduling: true,
        }
    }
}

/// Work item for parallel execution
#[derive(Debug, Clone)]
struct WorkItem {
    node_id: NodeId,
    priority: u32,
    estimated_cost: u64,
    created_at: Instant,
}

impl PartialEq for WorkItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.estimated_cost == other.estimated_cost
    }
}

impl Eq for WorkItem {}

impl PartialOrd for WorkItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorkItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then lower cost
        self.priority.cmp(&other.priority)
            .then_with(|| other.estimated_cost.cmp(&self.estimated_cost))
    }
}

/// Work stealing queue for load balancing
#[derive(Debug)]
struct WorkStealingQueue {
    local_queue: VecDeque<WorkItem>,
    steal_queue: Arc<Mutex<VecDeque<WorkItem>>>,
    worker_id: usize,
}

impl WorkStealingQueue {
    fn new(worker_id: usize) -> Self {
        Self {
            local_queue: VecDeque::new(),
            steal_queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_id,
        }
    }
    
    /// Push work to local queue
    fn push(&mut self, item: WorkItem) {
        self.local_queue.push_back(item);
    }
    
    /// Pop work from local queue (LIFO for cache locality)
    fn pop(&mut self) -> Option<WorkItem> {
        self.local_queue.pop_back()
    }
    
    /// Try to steal work from another worker's queue
    fn steal_from(&mut self, other: &Arc<Mutex<VecDeque<WorkItem>>>) -> Option<WorkItem> {
        if let Ok(mut queue) = other.try_lock() {
            queue.pop_front() // FIFO for stolen work
        } else {
            None
        }
    }
    
    /// Make work available for stealing
    fn make_stealable(&mut self) {
        let half_point = self.local_queue.len() / 2;
        if half_point > 0 {
            if let Ok(mut steal_queue) = self.steal_queue.try_lock() {
                for _ in 0..half_point {
                    if let Some(item) = self.local_queue.pop_front() {
                        steal_queue.push_back(item);
                    }
                }
            }
        }
    }
    
    /// Get reference to steal queue for other workers
    fn steal_queue(&self) -> Arc<Mutex<VecDeque<WorkItem>>> {
        Arc::clone(&self.steal_queue)
    }
}

/// Worker thread state
#[derive(Debug)]
struct WorkerState {
    id: usize,
    queue: WorkStealingQueue,
    completed_nodes: u64,
    stolen_work: u64,
    provided_work: u64,
    execution_time: Duration,
}

impl WorkerState {
    fn new(id: usize) -> Self {
        Self {
            id,
            queue: WorkStealingQueue::new(id),
            completed_nodes: 0,
            stolen_work: 0,
            provided_work: 0,
            execution_time: Duration::ZERO,
        }
    }
}

/// Execution state shared between workers
#[derive(Debug)]
struct SharedExecutionState {
    teg: Arc<TemporalEffectGraph>,
    node_results: Arc<Mutex<BTreeMap<NodeId, Value>>>,
    node_status: Arc<Mutex<BTreeMap<NodeId, NodeStatus>>>,
    execution_errors: Arc<Mutex<Vec<(NodeId, String)>>>,
    ready_queue: Arc<(Mutex<VecDeque<WorkItem>>, Condvar)>,
    completion_signal: Arc<(Mutex<bool>, Condvar)>,
    start_time: Instant,
}

/// TEG executor with dynamic orchestration
pub struct TegExecutor {
    config: TegExecutorConfig,
    runtime_context: RuntimeContext,
}

impl TegExecutor {
    /// Create a new TEG executor with default configuration
    pub fn new(runtime_context: RuntimeContext) -> Self {
        Self {
            config: TegExecutorConfig::default(),
            runtime_context,
        }
    }
    
    /// Create a new TEG executor with custom configuration
    pub fn with_config(config: TegExecutorConfig, runtime_context: RuntimeContext) -> Self {
        Self {
            config,
            runtime_context,
        }
    }
    
    /// Execute a TEG with parallel orchestration
    pub fn execute(&mut self, teg: TemporalEffectGraph) -> RuntimeResult<TegResult> {
        let start_time = Instant::now();
        
        // Initialize shared state - fix circular reference
        let teg_arc = Arc::new(teg);
        let node_status_map: BTreeMap<NodeId, NodeStatus> = teg_arc.nodes.keys()
            .map(|&id| (id, NodeStatus::Pending))
            .collect();
        
        let shared_state = Arc::new(SharedExecutionState {
            teg: Arc::clone(&teg_arc),
            node_results: Arc::new(Mutex::new(BTreeMap::new())),
            node_status: Arc::new(Mutex::new(node_status_map)),
            execution_errors: Arc::new(Mutex::new(Vec::new())),
            ready_queue: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            completion_signal: Arc::new((Mutex::new(false), Condvar::new())),
            start_time,
        });
        
        // Find initially ready nodes
        let ready_nodes = shared_state.teg.get_ready_nodes();
        {
            let (queue_lock, _) = &*shared_state.ready_queue;
            let mut queue = queue_lock.lock().unwrap();
            for node_id in ready_nodes {
                if let Some(node) = shared_state.teg.nodes.get(&node_id) {
                    queue.push_back(WorkItem {
                        node_id,
                        priority: self.calculate_priority(node),
                        estimated_cost: node.cost,
                        created_at: Instant::now(),
                    });
                }
            }
        }
        
        // Spawn worker threads
        let mut workers = Vec::new();
        let worker_steal_queues = Arc::new(Mutex::new(Vec::new()));
        
        for worker_id in 0..self.config.worker_count {
            let shared_state = Arc::clone(&shared_state);
            let config = self.config.clone();
            let runtime_context = self.runtime_context.clone();
            let steal_queues = Arc::clone(&worker_steal_queues);
            
            let handle = thread::spawn(move || {
                Self::worker_loop(worker_id, shared_state, config, runtime_context, steal_queues)
            });
            
            workers.push(handle);
        }
        
        // Wait for completion or timeout
        let total_nodes = shared_state.teg.nodes.len();
        let (completion_lock, completion_cvar) = &*shared_state.completion_signal;
        
        loop {
            let mut completed = completion_lock.lock().unwrap();
            
            // Check if all nodes are completed
            let status_map = shared_state.node_status.lock().unwrap();
            let completed_count = status_map.values()
                .filter(|status| **status == NodeStatus::Completed)
                .count();
            drop(status_map);
            
            if completed_count == total_nodes {
                *completed = true;
                completion_cvar.notify_all();
                break;
            }
            
            // Check for errors
            let errors = shared_state.execution_errors.lock().unwrap();
            if !errors.is_empty() {
                drop(errors);
                *completed = true;
                completion_cvar.notify_all();
                break;
            }
            drop(errors);
            
            // Wait with timeout
            let timeout = Duration::from_millis(self.config.node_timeout_ms);
            let (mut new_completed, timeout_result) = completion_cvar.wait_timeout(completed, timeout).unwrap();
            
            if timeout_result.timed_out() {
                *new_completed = true;
                completion_cvar.notify_all();
                break;
            }
        }
        
        // Signal workers to stop and wait for them
        {
            let (queue_lock, queue_cvar) = &*shared_state.ready_queue;
            let _queue = queue_lock.lock().unwrap();
            queue_cvar.notify_all();
        }
        
        let mut worker_stats = Vec::new();
        for handle in workers {
            if let Ok(stats) = handle.join() {
                worker_stats.push(stats);
            }
        }
        
        // Collect results
        let results = shared_state.node_results.lock().unwrap().clone();
        let errors = shared_state.execution_errors.lock().unwrap().clone();
        
        let total_time = start_time.elapsed();
        let parallel_nodes = worker_stats.iter().map(|s| s.completed_nodes).sum();
        let actual_parallelization = if total_time.as_millis() > 0 {
            // Use simple integer ratio: parallel_nodes * 1000 / time_seconds for 3 decimal precision
            let time_seconds = total_time.as_millis() as i64 / 1000;
            if time_seconds > 0 {
                (parallel_nodes as i64 * 1000) / time_seconds
            } else {
                1000 // 1.0 scaled by 1000
            }
        } else {
            1000 // 1.0 scaled by 1000
        };
        
        let stats = ExecutionStats {
            total_time_ms: total_time.as_millis() as u64,
            parallel_nodes,
            actual_parallelization,
            critical_path_time_ms: shared_state.teg.metadata.critical_path_length,
        };
        
        Ok(TegResult {
            results,
            stats,
            errors,
        })
    }
    
    /// Worker thread main loop
    fn worker_loop(
        worker_id: usize,
        shared_state: Arc<SharedExecutionState>,
        config: TegExecutorConfig,
        runtime_context: RuntimeContext,
        steal_queues: Arc<Mutex<Vec<Arc<Mutex<VecDeque<WorkItem>>>>>>,
    ) -> WorkerState {
        let mut worker_state = WorkerState::new(worker_id);
        let mut interpreter = Interpreter::new(runtime_context);
        
        // Register our steal queue
        {
            let mut queues = steal_queues.lock().unwrap();
            if queues.len() <= worker_id {
                queues.resize(worker_id + 1, Arc::new(Mutex::new(VecDeque::new())));
            }
            queues[worker_id] = worker_state.queue.steal_queue();
        }
        
        let steal_timeout = Duration::from_millis(config.steal_timeout_ms);
        
        loop {
            // Check for completion signal
            let (completion_lock, _) = &*shared_state.completion_signal;
            if let Ok(completed) = completion_lock.try_lock() {
                if *completed {
                    break;
                }
            }
            
            // Try to get work from local queue first
            let work_item = if let Some(item) = worker_state.queue.pop() {
                Some(item)
            } else {
                // Try to get work from shared ready queue
                Self::try_get_shared_work(&shared_state, steal_timeout)
                    .or_else(|| Self::try_steal_work(&mut worker_state, &steal_queues, worker_id))
            };
            
            if let Some(item) = work_item {
                let execution_start = Instant::now();
                
                if let Some(result) = Self::execute_node(
                    &shared_state,
                    &mut interpreter,
                    item.node_id,
                    config.node_timeout_ms,
                ) {
                    // Update node status and results
                    {
                        let mut status_map = shared_state.node_status.lock().unwrap();
                        let mut results_map = shared_state.node_results.lock().unwrap();
                        
                        match result {
                            Ok(value) => {
                                status_map.insert(item.node_id, NodeStatus::Completed);
                                results_map.insert(item.node_id, value);
                                worker_state.completed_nodes += 1;
                            }
                            Err(error) => {
                                status_map.insert(item.node_id, NodeStatus::Failed(error.clone()));
                                let mut errors = shared_state.execution_errors.lock().unwrap();
                                errors.push((item.node_id, error));
                            }
                        }
                    }
                    
                    // Find newly ready nodes
                    let newly_ready = Self::find_newly_ready_nodes(&shared_state, item.node_id);
                    Self::enqueue_ready_nodes(&shared_state, &mut worker_state, newly_ready);
                }
                
                worker_state.execution_time += execution_start.elapsed();
                
                // Make work available for stealing if we have too much
                if worker_state.queue.local_queue.len() > config.load_balance_threshold {
                    worker_state.queue.make_stealable();
                }
            } else {
                // No work found, short sleep to avoid busy waiting
                thread::sleep(Duration::from_millis(1));
            }
        }
        
        worker_state
    }
    
    /// Try to get work from the shared ready queue
    fn try_get_shared_work(
        shared_state: &SharedExecutionState,
        timeout: Duration,
    ) -> Option<WorkItem> {
        let (queue_lock, queue_cvar) = &*shared_state.ready_queue;
        
        if let Ok(mut queue) = queue_lock.try_lock() {
            queue.pop_front()
        } else {
            // Wait for work with timeout
            let queue_result = queue_lock.lock();
            if let Ok(mut queue) = queue_result {
                if queue.is_empty() {
                    let (mut queue, timeout_result) = queue_cvar.wait_timeout(queue, timeout).unwrap();
                    if !timeout_result.timed_out() {
                        queue.pop_front()
                    } else {
                        None
                    }
                } else {
                    queue.pop_front()
                }
            } else {
                None
            }
        }
    }
    
    /// Try to steal work from other workers
    fn try_steal_work(
        worker_state: &mut WorkerState,
        steal_queues: &Arc<Mutex<Vec<Arc<Mutex<VecDeque<WorkItem>>>>>>,
        worker_id: usize,
    ) -> Option<WorkItem> {
        if let Ok(queues) = steal_queues.try_lock() {
            for (other_id, queue) in queues.iter().enumerate() {
                if other_id != worker_id {
                    if let Some(item) = worker_state.queue.steal_from(queue) {
                        return Some(item);
                    }
                }
            }
        }
        None
    }
    
    /// Execute a single node
    fn execute_node(
        shared_state: &SharedExecutionState,
        interpreter: &mut Interpreter,
        node_id: NodeId,
        timeout_ms: u64,
    ) -> Option<Result<Value, String>> {
        // Update status to executing
        {
            let mut status_map = shared_state.node_status.lock().unwrap();
            status_map.insert(node_id, NodeStatus::Executing);
        }
        
        if let Some(node) = shared_state.teg.nodes.get(&node_id) {
            let start_time = Instant::now();
            let timeout = Duration::from_millis(timeout_ms);
            
            // Execute the effect with timeout - use serde_json::Value as intermediate
            match interpreter.execute::<serde_json::Value>(node.effect.clone()) {
                Ok(json_value) => {
                    if start_time.elapsed() < timeout {
                        // Convert serde_json::Value to causality_core::Value
                        use causality_core::system::Str;
                        let value = Value::Symbol(Str::new(&json_value.to_string()));
                        Some(Ok(value))
                    } else {
                        Some(Err("Node execution timeout".to_string()))
                    }
                }
                Err(e) => Some(Err(e.to_string())),
            }
        } else {
            Some(Err("Node not found".to_string()))
        }
    }
    
    /// Find newly ready nodes after a node completes
    fn find_newly_ready_nodes(
        shared_state: &SharedExecutionState,
        _completed_node: NodeId,
    ) -> Vec<NodeId> {
        let mut newly_ready = Vec::new();
        let status_map = shared_state.node_status.lock().unwrap();
        
        // Check all nodes to see if any became ready
        for (node_id, node) in &shared_state.teg.nodes {
            if status_map.get(node_id) == Some(&NodeStatus::Pending) {
                let deps_satisfied = node.dependencies.iter().all(|dep_id| {
                    status_map.get(dep_id) == Some(&NodeStatus::Completed)
                });
                
                if deps_satisfied {
                    newly_ready.push(*node_id);
                }
            }
        }
        
        newly_ready
    }
    
    /// Enqueue newly ready nodes for execution
    fn enqueue_ready_nodes(
        shared_state: &SharedExecutionState,
        worker_state: &mut WorkerState,
        ready_nodes: Vec<NodeId>,
    ) {
        for node_id in ready_nodes {
            if let Some(node) = shared_state.teg.nodes.get(&node_id) {
                let work_item = WorkItem {
                    node_id,
                    priority: Self::calculate_priority_static(node),
                    estimated_cost: node.cost,
                    created_at: Instant::now(),
                };
                
                // Add to local queue for cache locality
                worker_state.queue.push(work_item);
            }
        }
    }
    
    /// Calculate priority for a node (higher = more important)
    fn calculate_priority(&self, node: &EffectNode) -> u32 {
        Self::calculate_priority_static(node)
    }
    
    /// Static priority calculation
    fn calculate_priority_static(node: &EffectNode) -> u32 {
        // Higher priority for:
        // - Nodes with more dependents (critical path)
        // - Nodes with higher cost (get them started early)  
        // - Nodes that produce resources needed by others
        
        let base_priority = 1000;
        let cost_bonus = (node.cost / 100).min(500) as u32; // Max 500 bonus for cost
        let resource_bonus = node.resource_productions.len() as u32 * 50; // 50 per produced resource
        
        base_priority + cost_bonus + resource_bonus
    }
}

/// Statistics for worker performance
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub worker_id: usize,
    pub completed_nodes: u64,
    pub stolen_work: u64,
    pub provided_work: u64,
    pub execution_time: Duration,
    pub efficiency: i64, // Nodes per second * 1000 for precision
}

impl From<WorkerState> for WorkerStats {
    fn from(state: WorkerState) -> Self {
        let efficiency = if state.execution_time.as_millis() > 0 {
            // Use simple integer ratio: completed_nodes / execution_time_seconds
            let time_seconds = state.execution_time.as_millis() as i64 / 1000;
            if time_seconds > 0 {
                (state.completed_nodes as i64 * 1000) / time_seconds
            } else {
                1000 // 1.0 scaled by 1000
            }
        } else {
            1000 // 1.0 scaled by 1000
        };
        
        Self {
            worker_id: state.id,
            completed_nodes: state.completed_nodes,
            stolen_work: state.stolen_work,
            provided_work: state.provided_work,
            execution_time: state.execution_time,
            efficiency,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::effect::{EffectExpr, EffectExprKind};
    
    use causality_core::system::content_addressing::EntityId;
    
    #[test]
    fn test_teg_executor_creation() {
        let context = RuntimeContext::new();
        let executor = TegExecutor::new(context);
        assert_eq!(executor.config.worker_count, num_cpus::get().max(1));
    }
    
    #[test]
    fn test_work_item_ordering() {
        let item1 = WorkItem {
            node_id: EntityId::from_bytes([1; 32]),
            priority: 100,
            estimated_cost: 200,
            created_at: Instant::now(),
        };
        
        let item2 = WorkItem {
            node_id: EntityId::from_bytes([2; 32]),
            priority: 200,
            estimated_cost: 100,
            created_at: Instant::now(),
        };
        
        // Higher priority and lower cost should come first
        // item2 has priority 200 vs item1's 100, so item2 > item1
        assert!(item2 > item1);
        
        // Test cost tiebreaker with same priority
        let item3 = WorkItem {
            node_id: EntityId::from_bytes([3; 32]),
            priority: 100,
            estimated_cost: 50,  // Lower cost than item1
            created_at: Instant::now(),
        };
        
        // Same priority, but item3 has lower cost, so item3 > item1
        assert!(item3 > item1);
    }
    
    #[test]
    fn test_simple_teg_execution() {
        let context = RuntimeContext::new();
        let mut executor = TegExecutor::with_config(
            TegExecutorConfig {
                worker_count: 1,  // Use single thread for deterministic testing
                node_timeout_ms: 5000,  // 5 seconds timeout
                ..Default::default()
            },
            context
        );
        
        // Create a simple TEG with one effect using a literal value instead of unbound variable
        use causality_core::lambda::{Term, TermKind, Literal};
        let term = Term::new(TermKind::Literal(Literal::Int(42)));
        let effect = EffectExpr::new(EffectExprKind::Pure(term));
        let effects = vec![effect];
        let teg = TemporalEffectGraph::from_effect_sequence(effects).unwrap();
        
        // Verify TEG has one node
        assert_eq!(teg.nodes.len(), 1);
        
        let result = executor.execute(teg);
        assert!(result.is_ok(), "TEG execution should succeed");
        
        let teg_result = result.unwrap();
        
        // Check for execution errors first
        if !teg_result.errors.is_empty() {
            eprintln!("TEG execution errors: {:?}", teg_result.errors);
            eprintln!("Stats: {:?}", teg_result.stats);
        }
        assert!(teg_result.errors.is_empty(), "No execution errors should occur");
        
        // Now we should have results
        assert_eq!(teg_result.results.len(), 1, "Should have one result from the executed node");
    }
} 