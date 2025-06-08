//! HTTP server implementation for Causality API
//!
//! Provides a REST API server with endpoints for compilation, execution, and management.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{ApiConfig, ExecutionSession};

/// Start the HTTP server
pub async fn start_server(
    config: ApiConfig,
    _sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
) -> Result<()> {
    use colored::Colorize;
    
    println!("{}", "🌐 Starting Causality API Server...".cyan().bold());
    println!("📡 Server would run on http://{}:{}", config.host, config.port);
    println!("🔒 CORS: {}", if config.enable_cors { "enabled" } else { "disabled" });
    println!("📦 Max request size: {} bytes", config.max_request_size);
    
    println!("\n{}", "📋 Available endpoints:".yellow());
    println!("  GET  /health           - Health check");
    println!("  POST /compile          - Compile Lisp code");
    println!("  POST /execute          - Execute Lisp code");
    println!("  GET  /sessions         - List active sessions");
    println!("  POST /sessions         - Create new session");
    println!("  GET  /sessions/:id     - Get session info");
    println!("  DELETE /sessions/:id   - Delete session");
    println!("  GET  /docs             - API documentation");
    
    // In a real implementation, you would start an actual HTTP server here
    // For now, we'll just simulate it
    println!("\n{}", "💡 API Features:".green());
    println!("  • RESTful endpoints for all Causality operations");
    println!("  • Session-based execution state management");
    println!("  • JSON request/response format");
    println!("  • Error handling and validation");
    println!("  • Real-time compilation and execution");
    
    // Simulate server running
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("\n{} API server ready! (demo mode)", "✅".green());
    
    Ok(())
}

/// Health check endpoint
pub async fn health_check() -> Result<serde_json::Value> {
    Ok(json!({
        "status": "ok",
        "service": "causality-api",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Create a new session
pub async fn create_session(
    _sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
) -> Result<serde_json::Value> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = ExecutionSession::new(session_id.clone());
    
    _sessions.write().await.insert(session_id.clone(), session);
    
    Ok(json!({
        "session_id": session_id,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "status": "created"
    }))
}

/// List all sessions
pub async fn list_sessions(
    _sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
) -> Result<serde_json::Value> {
    let sessions_guard = _sessions.read().await;
    let session_list: Vec<serde_json::Value> = sessions_guard
        .values()
        .map(|session| {
            json!({
                "id": session.id,
                "created_at": session.created_at.to_rfc3339(),
                "last_accessed": session.last_accessed.to_rfc3339(),
                "stats": {
                    "compilations": session.metadata.stats.compilations,
                    "executions": session.metadata.stats.executions,
                    "errors": session.metadata.stats.errors
                }
            })
        })
        .collect();
    
    Ok(json!({
        "sessions": session_list,
        "total": session_list.len()
    }))
}

/// Compile Lisp source code
pub async fn compile_source(
    source: String,
    _sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
    session_id: Option<String>,
) -> Result<serde_json::Value> {
    use causality_compiler::EnhancedCompilerPipeline;
    
    let start_time = std::time::Instant::now();
    
    // Compile the source
    let mut compiler = EnhancedCompilerPipeline::new();
    let compiled = compiler.compile_full(&source)?;
    
    let compilation_time = start_time.elapsed().as_millis() as u64;
    
    // Update session stats if session provided
    if let Some(sid) = session_id {
        if let Some(session) = _sessions.write().await.get_mut(&sid) {
            session.metadata.stats.compilations += 1;
            session.touch();
        }
    }
    
    Ok(json!({
        "status": "success",
        "compilation_time_ms": compilation_time,
        "instruction_count": compiled.instructions.len(),
        "instructions": compiled.instructions.iter()
            .enumerate()
            .map(|(i, instr)| json!({
                "index": i,
                "instruction": format!("{:?}", instr)
            }))
            .collect::<Vec<_>>()
    }))
}

/// Execute Lisp source code
pub async fn execute_source(
    source: String,
    _sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
    session_id: Option<String>,
) -> Result<serde_json::Value> {
    use causality_compiler::EnhancedCompilerPipeline;
    use causality_runtime::Executor;
    
    let start_time = std::time::Instant::now();
    
    // Compile
    let mut compiler = EnhancedCompilerPipeline::new();
    let compiled = compiler.compile_full(&source)?;
    
    // Execute
    let mut executor = Executor::new();
    let result = executor.execute(&compiled.instructions)?;
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    // Update session stats if session provided
    if let Some(sid) = session_id {
        if let Some(session) = _sessions.write().await.get_mut(&sid) {
            session.metadata.stats.executions += 1;
            session.metadata.stats.total_execution_time_ms += execution_time;
            session.touch();
        }
    }
    
    Ok(json!({
        "status": "success",
        "execution_time_ms": execution_time,
        "result": format!("{:?}", result),
        "instruction_count": compiled.instructions.len()
    }))
} 