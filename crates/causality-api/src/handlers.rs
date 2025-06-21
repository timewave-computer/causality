//! Request handlers for Causality API
//!
//! High-level handlers that process HTTP requests and coordinate with the core system.

// pub mod debug;
// pub mod compile;
// pub mod vm;

use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{ExecutionSession, types::*};

/// API request handlers
pub struct ApiHandlers {
    /// Shared session storage
    sessions: Arc<RwLock<BTreeMap<String, ExecutionSession>>>,
}

impl ApiHandlers {
    /// Create new API handlers
    pub fn new(sessions: Arc<RwLock<BTreeMap<String, ExecutionSession>>>) -> Self {
        Self { sessions }
    }
    
    /// Handle compilation request
    pub async fn handle_compile(&self, request: CompileRequest) -> Result<ApiResponse<CompileResult>> {
        use causality_compiler::EnhancedCompilerPipeline;
        
        let start_time = std::time::Instant::now();
        
        // Compile the source
        let mut compiler = EnhancedCompilerPipeline::new();
        let compiled = compiler.compile_full(&request.source)?;
        
        let compilation_time = start_time.elapsed().as_millis() as u64;
        
        // Update session stats if session provided
        if let Some(session_id) = &request.session_id {
            if let Some(session) = self.sessions.write().await.get_mut(session_id) {
                session.metadata.stats.compilations += 1;
                session.touch();
            }
        }
        
        // Build instruction info if requested
        let instructions = if request.options
            .as_ref()
            .and_then(|opts| opts.show_stages)
            .unwrap_or(false)
        {
            Some(
                compiled
                    .instructions
                    .iter()
                    .enumerate()
                    .map(|(index, instr)| InstructionInfo {
                        index,
                        instruction: format!("{:?}", instr),
                        source_location: None, // Would need source mapping
                    })
                    .collect(),
            )
        } else {
            None
        };
        
        let result = CompileResult {
            compilation_time_ms: compilation_time,
            instruction_count: compiled.instructions.len(),
            instructions,
            warnings: Vec::new(), // Would need warning collection
        };
        
        Ok(ApiResponse::success(result))
    }
    
    /// Handle execution request
    pub async fn handle_execute(&self, request: ExecuteRequest) -> Result<ApiResponse<ExecuteResult>> {
        use causality_compiler::EnhancedCompilerPipeline;
        use causality_runtime::Executor;
        
        let start_time = std::time::Instant::now();
        
        // Compile
        let mut compiler = EnhancedCompilerPipeline::new();
        let compiled = compiler.compile_full(&request.source)?;
        
        // Execute
        let mut executor = Executor::new();
        let result = executor.execute(&compiled.instructions)?;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Update session stats if session provided
        if let Some(session_id) = &request.session_id {
            if let Some(session) = self.sessions.write().await.get_mut(session_id) {
                session.metadata.stats.executions += 1;
                session.metadata.stats.total_execution_time_ms += execution_time;
                session.touch();
            }
        }
        
        // Build execution trace if requested
        let trace = if request.options
            .as_ref()
            .and_then(|opts| opts.trace)
            .unwrap_or(false)
        {
            // In a real implementation, you'd collect execution traces
            Some(Vec::new())
        } else {
            None
        };
        
        // Build machine state if requested
        let machine_state = if request.options
            .as_ref()
            .and_then(|opts| opts.trace)
            .unwrap_or(false)
        {
            Some(MachineStateInfo {
                registers: BTreeMap::new(), // Would extract from executor
                program_counter: 0,
                stats: ExecutionStatsInfo {
                    steps_executed: compiled.instructions.len() as u64,
                    memory_usage: 0,
                    cpu_time_us: execution_time * 1000,
                },
            })
        } else {
            None
        };
        
        let result = ExecuteResult {
            execution_time_ms: execution_time,
            result: format!("{:?}", result),
            instruction_count: compiled.instructions.len(),
            trace,
            machine_state,
        };
        
        Ok(ApiResponse::success(result))
    }
    
    /// Handle session creation request
    pub async fn handle_create_session(&self, request: CreateSessionRequest) -> Result<ApiResponse<SessionInfo>> {
        let session_id = "deterministic_uuid".to_string();
        let mut session = ExecutionSession::new(session_id.clone());
        
        // Set session name and tags if provided
        if let Some(tags) = request.tags {
            session.metadata.tags = tags;
        }
        
        let session_info = SessionInfo {
            id: session.id.clone(),
            name: request.name,
            created_at: session.created_at.to_rfc3339(),
            last_accessed: session.last_accessed.to_rfc3339(),
            tags: session.metadata.tags.clone(),
            stats: SessionStatsInfo {
                compilations: session.metadata.stats.compilations,
                executions: session.metadata.stats.executions,
                total_execution_time_ms: session.metadata.stats.total_execution_time_ms,
                errors: session.metadata.stats.errors,
                warnings: 0,
            },
        };
        
        self.sessions.write().await.insert(session_id, session);
        
        Ok(ApiResponse::success(session_info))
    }
    
    /// Handle session listing request
    pub async fn handle_list_sessions(&self) -> Result<ApiResponse<Vec<SessionInfo>>> {
        let sessions_guard = self.sessions.read().await;
        let session_list: Vec<SessionInfo> = sessions_guard
            .values()
            .map(|session| SessionInfo {
                id: session.id.clone(),
                name: None, // Would need to store names
                created_at: session.created_at.to_rfc3339(),
                last_accessed: session.last_accessed.to_rfc3339(),
                tags: session.metadata.tags.clone(),
                stats: SessionStatsInfo {
                    compilations: session.metadata.stats.compilations,
                    executions: session.metadata.stats.executions,
                    total_execution_time_ms: session.metadata.stats.total_execution_time_ms,
                    errors: session.metadata.stats.errors,
                    warnings: 0,
                },
            })
            .collect();
        
        Ok(ApiResponse::success(session_list))
    }
    
    /// Handle session info request
    pub async fn handle_get_session(&self, session_id: String) -> Result<ApiResponse<SessionInfo>> {
        let sessions_guard = self.sessions.read().await;
        
        if let Some(session) = sessions_guard.get(&session_id) {
            let session_info = SessionInfo {
                id: session.id.clone(),
                name: None,
                created_at: session.created_at.to_rfc3339(),
                last_accessed: session.last_accessed.to_rfc3339(),
                tags: session.metadata.tags.clone(),
                stats: SessionStatsInfo {
                    compilations: session.metadata.stats.compilations,
                    executions: session.metadata.stats.executions,
                    total_execution_time_ms: session.metadata.stats.total_execution_time_ms,
                    errors: session.metadata.stats.errors,
                    warnings: 0,
                },
            };
            
            Ok(ApiResponse::success(session_info))
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    /// Handle session deletion request
    pub async fn handle_delete_session(&self, session_id: String) -> Result<ApiResponse<()>> {
        let mut sessions_guard = self.sessions.write().await;
        
        if sessions_guard.remove(&session_id).is_some() {
            Ok(ApiResponse::success(()))
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    /// Handle health check request
    pub async fn handle_health(&self) -> Result<ApiResponse<HealthInfo>> {
        let sessions_count = self.sessions.read().await.len();
        
        let health_info = HealthInfo {
            status: "healthy".to_string(),
            service: "causality-api".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 0, // Would track actual uptime
            system: SystemInfo {
                available_memory: 0, // Would query system
                cpu_usage: 0.0,
                active_sessions: sessions_count,
                total_requests: 0, // Would track requests
            },
        };
        
        Ok(ApiResponse::success(health_info))
    }
} 