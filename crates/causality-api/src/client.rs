//! HTTP client for Causality API
//!
//! Provides a convenient Rust client for interacting with the Causality API server.

use anyhow::Result;
use std::collections::BTreeMap;

use crate::types::*;

/// HTTP client for Causality API
#[derive(Debug, Clone)]
pub struct CausalityClient {
    /// Base URL of the API server
    base_url: String,
    
    /// HTTP client
    client: reqwest::Client,
    
    /// Default session ID (if any)
    default_session: Option<String>,
}

impl CausalityClient {
    /// Create a new client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            default_session: None,
        }
    }
    
    /// Create a client with a default session
    pub fn with_session(base_url: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            default_session: Some(session_id.into()),
        }
    }
    
    /// Set the default session ID
    pub fn set_default_session(&mut self, session_id: impl Into<String>) {
        self.default_session = Some(session_id.into());
    }
    
    /// Clear the default session ID
    pub fn clear_default_session(&mut self) {
        self.default_session = None;
    }
    
    /// Health check
    pub async fn health(&self) -> Result<HealthInfo> {
        let url = format!("{}/health", self.base_url);
        let response: ApiResponse<HealthInfo> = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }
    
    /// Compile source code
    pub async fn compile(&self, source: impl Into<String>) -> Result<CompileResult> {
        self.compile_with_options(source, None).await
    }
    
    /// Compile source code with options
    pub async fn compile_with_options(
        &self,
        source: impl Into<String>,
        options: Option<CompileOptions>,
    ) -> Result<CompileResult> {
        let url = format!("{}/compile", self.base_url);
        let request = CompileRequest {
            source: source.into(),
            session_id: self.default_session.clone(),
            options,
        };
        
        let response: ApiResponse<CompileResult> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }
    
    /// Execute source code
    pub async fn execute(&self, source: impl Into<String>) -> Result<ExecuteResult> {
        self.execute_with_options(source, None).await
    }
    
    /// Execute source code with options
    pub async fn execute_with_options(
        &self,
        source: impl Into<String>,
        options: Option<ExecuteOptions>,
    ) -> Result<ExecuteResult> {
        let url = format!("{}/execute", self.base_url);
        let request = ExecuteRequest {
            source: source.into(),
            session_id: self.default_session.clone(),
            options,
        };
        
        let response: ApiResponse<ExecuteResult> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }
    
    /// Create a new session
    pub async fn create_session(&self) -> Result<SessionInfo> {
        self.create_session_with_options(None, None).await
    }
    
    /// Create a new session with name and tags
    pub async fn create_session_with_options(
        &self,
        name: Option<String>,
        tags: Option<BTreeMap<String, String>>,
    ) -> Result<SessionInfo> {
        let url = format!("{}/sessions", self.base_url);
        let request = CreateSessionRequest { name, tags };
        
        let response: ApiResponse<SessionInfo> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }
    
    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let url = format!("{}/sessions", self.base_url);
        let response: ApiResponse<Vec<SessionInfo>> = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }
    
    /// Get session information
    pub async fn get_session(&self, session_id: impl Into<String>) -> Result<SessionInfo> {
        let url = format!("{}/sessions/{}", self.base_url, session_id.into());
        let response: ApiResponse<SessionInfo> = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }
    
    /// Delete a session
    pub async fn delete_session(&self, session_id: impl Into<String>) -> Result<()> {
        let url = format!("{}/sessions/{}", self.base_url, session_id.into());
        let _response: ApiResponse<()> = self.client
            .delete(&url)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(())
    }
}

/// Builder for compile options
#[derive(Debug, Default)]
pub struct CompileOptionsBuilder {
    optimize: Option<bool>,
    show_stages: Option<bool>,
    target: Option<String>,
}

impl CompileOptionsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Enable optimizations
    pub fn optimize(mut self, optimize: bool) -> Self {
        self.optimize = Some(optimize);
        self
    }
    
    /// Show compilation stages
    pub fn show_stages(mut self, show_stages: bool) -> Self {
        self.show_stages = Some(show_stages);
        self
    }
    
    /// Set target platform
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
    
    /// Build the options
    pub fn build(self) -> CompileOptions {
        CompileOptions {
            optimize: self.optimize,
            show_stages: self.show_stages,
            target: self.target,
        }
    }
}

/// Builder for execute options
#[derive(Debug, Default)]
pub struct ExecuteOptionsBuilder {
    max_steps: Option<u64>,
    trace: Option<bool>,
    timeout_seconds: Option<u64>,
}

impl ExecuteOptionsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set maximum execution steps
    pub fn max_steps(mut self, max_steps: u64) -> Self {
        self.max_steps = Some(max_steps);
        self
    }
    
    /// Enable execution trace
    pub fn trace(mut self, trace: bool) -> Self {
        self.trace = Some(trace);
        self
    }
    
    /// Set timeout in seconds
    pub fn timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
    
    /// Build the options
    pub fn build(self) -> ExecuteOptions {
        ExecuteOptions {
            max_steps: self.max_steps,
            trace: self.trace,
            timeout_seconds: self.timeout_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = CausalityClient::new("http://localhost:3000");
        assert_eq!(client.base_url, "http://localhost:3000");
        assert!(client.default_session.is_none());
    }
    
    #[test]
    fn test_options_builders() {
        let compile_opts = CompileOptionsBuilder::new()
            .optimize(true)
            .show_stages(true)
            .target("wasm")
            .build();
        
        assert_eq!(compile_opts.optimize, Some(true));
        assert_eq!(compile_opts.show_stages, Some(true));
        assert_eq!(compile_opts.target, Some("wasm".to_string()));
        
        let execute_opts = ExecuteOptionsBuilder::new()
            .max_steps(1000)
            .trace(true)
            .timeout_seconds(30)
            .build();
        
        assert_eq!(execute_opts.max_steps, Some(1000));
        assert_eq!(execute_opts.trace, Some(true));
        assert_eq!(execute_opts.timeout_seconds, Some(30));
    }
} 