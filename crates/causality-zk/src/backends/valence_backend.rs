//! Valence Coprocessor backend implementation
//!
//! This module provides integration with the Valence coprocessor system
//! for remote ZK proof generation and verification using the causality-api client.

use crate::{
    ZkCircuit,
    error::{ProofResult, VerificationError, ProofError},
    backends::ZkBackend,
    proof_generation::{ZkProof, ZkWitness},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use log::{info, warn, error, debug, trace};
use causality_core::machine::instruction::{Instruction, RegisterId};

/// Performance metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total number of proofs generated
    pub total_proofs_generated: u64,
    
    /// Total number of verification operations
    pub total_verifications: u64,
    
    /// Average proof generation time in milliseconds
    pub avg_proof_generation_time_ms: f64,
    
    /// Average verification time in milliseconds
    pub avg_verification_time_ms: f64,
    
    /// Number of cache hits
    pub cache_hits: u64,
    
    /// Number of cache misses
    pub cache_misses: u64,
    
    /// Number of failed operations
    pub failed_operations: u64,
    
    /// Number of retry attempts
    pub retry_attempts: u64,
    
    /// Circuit compilation times
    pub circuit_compilation_times: Vec<u64>,
    
    /// Service health status
    pub service_healthy: bool,
    
    /// Last health check timestamp
    pub last_health_check: u64,
}

impl PerformanceMetrics {
    /// Record a successful proof generation
    pub fn record_proof_generation(&mut self, duration_ms: u64) {
        self.total_proofs_generated += 1;
        self.update_avg_proof_time(duration_ms as f64);
        info!("Proof generated successfully in {}ms", duration_ms);
    }
    
    /// Record a successful verification
    pub fn record_verification(&mut self, duration_ms: u64) {
        self.total_verifications += 1;
        self.update_avg_verification_time(duration_ms as f64);
        debug!("Proof verified in {}ms", duration_ms);
    }
    
    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
        trace!("Cache hit recorded");
    }
    
    /// Record a cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
        trace!("Cache miss recorded");
    }
    
    /// Record a failed operation
    pub fn record_failure(&mut self, operation: &str, error: &str) {
        self.failed_operations += 1;
        error!("Operation '{}' failed: {}", operation, error);
    }
    
    /// Record a retry attempt
    pub fn record_retry(&mut self, attempt: u32, operation: &str) {
        self.retry_attempts += 1;
        warn!("Retry attempt {} for operation '{}'", attempt, operation);
    }
    
    /// Record circuit compilation time
    pub fn record_circuit_compilation(&mut self, duration_ms: u64) {
        self.circuit_compilation_times.push(duration_ms);
        // Keep only last 100 compilation times for memory efficiency
        if self.circuit_compilation_times.len() > 100 {
            self.circuit_compilation_times.remove(0);
        }
        debug!("Circuit compiled in {}ms", duration_ms);
    }
    
    /// Update service health status
    pub fn update_health_status(&mut self, healthy: bool) {
        self.service_healthy = healthy;
        self.last_health_check = chrono::Utc::now().timestamp() as u64;
        if healthy {
            info!("Service health check: HEALTHY");
        } else {
            error!("Service health check: UNHEALTHY");
        }
    }
    
    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64 * 100.0
        }
    }
    
    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        let total_operations = self.total_proofs_generated + self.total_verifications + self.failed_operations;
        if total_operations == 0 {
            0.0
        } else {
            self.failed_operations as f64 / total_operations as f64 * 100.0
        }
    }
    
    /// Get average circuit compilation time
    pub fn avg_circuit_compilation_time_ms(&self) -> f64 {
        if self.circuit_compilation_times.is_empty() {
            0.0
        } else {
            let sum: u64 = self.circuit_compilation_times.iter().sum();
            sum as f64 / self.circuit_compilation_times.len() as f64
        }
    }
    
    /// Update average proof generation time
    fn update_avg_proof_time(&mut self, new_time: f64) {
        if self.total_proofs_generated == 1 {
            self.avg_proof_generation_time_ms = new_time;
        } else {
            // Exponential moving average
            let alpha = 0.1;
            self.avg_proof_generation_time_ms = 
                alpha * new_time + (1.0 - alpha) * self.avg_proof_generation_time_ms;
        }
    }
    
    /// Update average verification time
    fn update_avg_verification_time(&mut self, new_time: f64) {
        if self.total_verifications == 1 {
            self.avg_verification_time_ms = new_time;
        } else {
            // Exponential moving average
            let alpha = 0.1;
            self.avg_verification_time_ms = 
                alpha * new_time + (1.0 - alpha) * self.avg_verification_time_ms;
        }
    }
}

/// Valence coprocessor backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceConfig {
    /// Coprocessor endpoint URL
    pub endpoint: String,
    
    /// API key for authentication
    pub api_key: Option<String>,
    
    /// Controller ID for the deployed circuit
    pub controller_id: Option<String>,
    
    /// Circuit name for deployment
    pub circuit_name: String,
    
    /// Maximum proof generation timeout
    pub timeout: Duration,
    
    /// Whether to auto-deploy circuits
    pub auto_deploy: bool,
    
    /// Polling interval for async proof generation
    pub polling_interval: Duration,
    
    /// Maximum number of retry attempts
    pub max_retries: u32,
    
    /// Connection timeout for HTTP requests
    pub connection_timeout: Duration,
}

impl Default for ValenceConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://prover.timewave.computer:37281".to_string(),
            api_key: None,
            controller_id: None,
            circuit_name: "causality-circuit".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes default
            auto_deploy: true,
            polling_interval: Duration::from_secs(5), // Poll every 5 seconds
            max_retries: 3,
            connection_timeout: Duration::from_secs(30),
        }
    }
}

/// Circuit information response from Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub name: String,
    pub id: String,
    pub description: Option<String>,
    pub created_at: String,
    pub parameters: Option<HashMap<String, String>>,
    pub status: CircuitStatus,
    pub verification_key: Option<String>,
}

/// Circuit deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitStatus {
    Deploying,
    Ready,
    Failed,
    Unknown,
}

/// Circuit deployment request to Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployCircuitRequest {
    pub name: String,
    pub wasm_bytecode: String,
    pub description: Option<String>,
    pub parameters: Option<HashMap<String, String>>,
}

/// ZK proof request to Valence coprocessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofRequest {
    pub circuit_name: String,
    pub public_inputs: Vec<String>,
    pub private_inputs: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub priority: Option<ProofPriority>,
}

/// Proof generation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Async proof generation job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofJob {
    pub job_id: String,
    pub status: JobStatus,
    pub created_at: String,
    pub estimated_completion_time: Option<String>,
    pub progress: Option<f64>,
}

/// Proof generation job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// ZK proof response from Valence coprocessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofResponse {
    pub proof: String,
    pub public_inputs: Vec<String>,
    pub generation_time_ms: u64,
    pub circuit_info: CircuitInfo,
    pub job_id: Option<String>,
    pub verification_key_id: Option<String>,
}

/// Proof verification request to Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofRequest {
    pub proof: String,
    pub public_inputs: Vec<String>,
    pub circuit_name: String,
    pub verification_key_id: Option<String>,
}

/// Error response from Valence service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceError {
    pub error: String,
    pub code: u32,
    pub details: Option<HashMap<String, String>>,
}

/// Enhanced Valence client for ZK operations with async support
#[derive(Debug, Clone)]
pub struct ValenceClient {
    endpoint: String,
    client: reqwest::Client,
    api_key: Option<String>,
    config: ValenceConfig,
}

impl ValenceClient {
    /// Create a new Valence client
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        let config = ValenceConfig::default();
        let client = reqwest::Client::builder()
            .timeout(config.connection_timeout)
            .build()
            .unwrap();
            
        Self {
            endpoint,
            client,
            api_key,
            config,
        }
    }
    
    /// Create a new Valence client with custom configuration
    pub fn with_config(config: ValenceConfig) -> Self {
        let endpoint = config.endpoint.clone();
        let api_key = config.api_key.clone();
        
        let client = reqwest::Client::builder()
            .timeout(config.connection_timeout)
            .build()
            .unwrap();
            
        Self {
            endpoint,
            client,
            api_key,
            config,
        }
    }
    
    /// Deploy a circuit to the Valence coprocessor
    pub async fn deploy_circuit(&self, request: DeployCircuitRequest) -> Result<CircuitInfo> {
        let url = format!("{}/api/v1/circuits", self.endpoint);
        
        let mut req = self.client.post(&url).json(&request);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Circuit deployment failed: {}", error_text));
        }
        
        let circuit_info: CircuitInfo = response.json().await?;
        
        // Wait for circuit to be ready if auto-deploy is enabled
        if self.config.auto_deploy {
            self.wait_for_circuit_ready(&circuit_info.id).await?;
        }
        
        Ok(circuit_info)
    }
    
    /// Start asynchronous proof generation
    pub async fn start_proof_generation(&self, request: ZkProofRequest) -> Result<ProofJob> {
        let url = format!("{}/api/v1/proofs/async", self.endpoint);
        
        let mut req = self.client.post(&url).json(&request);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Proof generation start failed: {}", error_text));
        }
        
        let job: ProofJob = response.json().await?;
        Ok(job)
    }
    
    /// Check the status of a proof generation job
    pub async fn get_job_status(&self, job_id: &str) -> Result<ProofJob> {
        let url = format!("{}/api/v1/jobs/{}", self.endpoint, job_id);
        
        let mut req = self.client.get(&url);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Job status check failed: {}", error_text));
        }
        
        let job: ProofJob = response.json().await?;
        Ok(job)
    }
    
    /// Get the result of a completed proof generation job
    pub async fn get_proof_result(&self, job_id: &str) -> Result<ZkProofResponse> {
        let url = format!("{}/api/v1/jobs/{}/result", self.endpoint, job_id);
        
        let mut req = self.client.get(&url);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Get proof result failed: {}", error_text));
        }
        
        let proof_response: ZkProofResponse = response.json().await?;
        Ok(proof_response)
    }
    
    /// Generate a ZK proof with polling (convenience method)
    pub async fn generate_proof_with_polling(&self, request: ZkProofRequest) -> Result<ZkProofResponse> {
        // Start the proof generation job
        let job = self.start_proof_generation(request).await?;
        
        // Poll until completion or timeout
        let start_time = std::time::Instant::now();
        
        loop {
            // Check if we've exceeded the timeout
            if start_time.elapsed() > self.config.timeout {
                return Err(anyhow::anyhow!("Proof generation timed out after {:?}", self.config.timeout));
            }
            
            // Check job status
            let job_status = self.get_job_status(&job.job_id).await?;
            
            match job_status.status {
                JobStatus::Completed => {
                    // Get the proof result
                    return self.get_proof_result(&job.job_id).await;
                }
                JobStatus::Failed => {
                    return Err(anyhow::anyhow!("Proof generation failed for job {}", job.job_id));
                }
                JobStatus::Cancelled => {
                    return Err(anyhow::anyhow!("Proof generation was cancelled for job {}", job.job_id));
                }
                JobStatus::Queued | JobStatus::InProgress => {
                    // Continue polling
                    tokio::time::sleep(self.config.polling_interval).await;
                }
            }
        }
    }
    
    /// Generate a ZK proof using the Valence coprocessor (legacy sync method)
    pub async fn generate_proof(&self, request: ZkProofRequest) -> Result<ZkProofResponse> {
        // Try async generation with polling first, fall back to sync if not supported
        match self.generate_proof_with_polling(request.clone()).await {
            Ok(response) => Ok(response),
            Err(_) => {
                // Fall back to synchronous generation
                let url = format!("{}/api/v1/proofs", self.endpoint);
                
                let mut req = self.client.post(&url).json(&request);
                if let Some(api_key) = &self.api_key {
                    req = req.header("Authorization", format!("Bearer {}", api_key));
                }
                
                let response = req.send().await?;
                
                if !response.status().is_success() {
                    let error_text = response.text().await?;
                    return Err(anyhow::anyhow!("Proof generation failed: {}", error_text));
                }
                
                let proof_response: ZkProofResponse = response.json().await?;
                Ok(proof_response)
            }
        }
    }
    
    /// Verify a ZK proof using the Valence coprocessor
    pub async fn verify_proof(&self, request: VerifyProofRequest) -> Result<bool> {
        let url = format!("{}/api/v1/verify", self.endpoint);
        
        let mut req = self.client.post(&url).json(&request);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Proof verification failed: {}", error_text));
        }
        
        let is_valid: bool = response.json().await?;
        Ok(is_valid)
    }
    
    /// List available circuits
    pub async fn list_circuits(&self) -> Result<Vec<CircuitInfo>> {
        let url = format!("{}/api/v1/circuits", self.endpoint);
        
        let mut req = self.client.get(&url);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("List circuits failed: {}", error_text));
        }
        
        let circuits: Vec<CircuitInfo> = response.json().await?;
        Ok(circuits)
    }
    
    /// Wait for a circuit to become ready
    async fn wait_for_circuit_ready(&self, circuit_id: &str) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        loop {
            if start_time.elapsed() > self.config.timeout {
                return Err(anyhow::anyhow!("Circuit deployment timed out"));
            }
            
            let circuits = self.list_circuits().await?;
            if let Some(circuit) = circuits.iter().find(|c| c.id == circuit_id) {
                match circuit.status {
                    CircuitStatus::Ready => return Ok(()),
                    CircuitStatus::Failed => return Err(anyhow::anyhow!("Circuit deployment failed")),
                    CircuitStatus::Deploying => {
                        tokio::time::sleep(self.config.polling_interval).await;
                        continue;
                    }
                    CircuitStatus::Unknown => {
                        return Err(anyhow::anyhow!("Circuit status unknown"));
                    }
                }
            }
            
            tokio::time::sleep(self.config.polling_interval).await;
        }
    }
    
    /// Health check for the Valence service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.endpoint);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Valence backend for ZK proof generation and verification
pub struct ValenceBackend {
    config: ValenceConfig,
    client: ValenceClient,
    runtime: tokio::runtime::Runtime,
    key_manager: VerificationKeyManager,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl ValenceBackend {
    /// Create a new Valence backend with default configuration
    pub fn new() -> Self {
        let config = ValenceConfig::default();
        let client = ValenceClient::with_config(config.clone());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let key_manager = VerificationKeyManager::default();
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        
        Self { config, client, runtime, key_manager, metrics }
    }
    
    /// Create a new Valence backend with custom configuration
    pub fn with_config(config: ValenceConfig) -> Self {
        let client = ValenceClient::with_config(config.clone());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let key_manager = VerificationKeyManager::default();
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        
        Self { config, client, runtime, key_manager, metrics }
    }
    
    /// Create a new Valence backend with custom key manager
    pub fn with_key_manager(config: ValenceConfig, key_manager: VerificationKeyManager) -> Self {
        let client = ValenceClient::with_config(config.clone());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        
        Self { config, client, runtime, key_manager, metrics }
    }
    
    /// Store a verification key for a circuit
    pub fn store_verification_key(
        &mut self,
        circuit_id: String,
        key_data: Vec<u8>,
        metadata: VerificationKeyMetadata,
    ) -> Result<String, String> {
        self.key_manager.store_verification_key(circuit_id, key_data, metadata)
    }
    
    /// Get verification key for a circuit
    pub fn get_verification_key(&mut self, circuit_id: &str) -> Option<VerificationKeyEntry> {
        self.key_manager.get_verification_key(circuit_id)
    }
    
    /// List all verification keys for a circuit
    pub fn list_circuit_keys(&self, circuit_id: &str) -> Vec<VerificationKeyEntry> {
        self.key_manager.list_circuit_keys(circuit_id)
    }
    
    /// Check if the Valence service is available
    pub fn check_service_health(&self) -> bool {
        let start_time = Instant::now();
        let is_healthy = self.runtime.block_on(async {
            self.client.health_check().await.unwrap_or(false)
        });
        
        // Update metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.update_health_status(is_healthy);
        }
        
        let duration = start_time.elapsed();
        debug!("Health check completed in {:?}, status: {}", duration, is_healthy);
        
        is_healthy
    }
    
    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        match self.metrics.lock() {
            Ok(metrics) => metrics.clone(),
            Err(_) => {
                error!("Failed to lock metrics mutex");
                PerformanceMetrics::default()
            }
        }
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            *metrics = PerformanceMetrics::default();
            info!("Performance metrics reset");
        } else {
            error!("Failed to reset metrics - mutex lock failed");
        }
    }
    
    /// Generate multiple proofs concurrently
    pub fn generate_proofs_concurrent(&self, requests: Vec<(ZkCircuit, ZkWitness)>) -> Vec<ProofResult<ZkProof>> {
        info!("Starting concurrent proof generation for {} circuits", requests.len());
        
        let mut handles = Vec::new();
        
        for (circuit, witness) in requests {
            let backend_clone = self.clone();
            let handle = std::thread::spawn(move || {
                backend_clone.generate_proof(&circuit, &witness)
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(result) => {
                    results.push(result);
                    debug!("Concurrent proof generation {} completed", i);
                }
                Err(_) => {
                    let error = ProofError::GenerationFailed("Thread panic during proof generation".to_string());
                    results.push(Err(error));
                    error!("Concurrent proof generation {} failed due to thread panic", i);
                }
            }
        }
        
        info!("Concurrent proof generation completed. Success rate: {}/{}", 
              results.iter().filter(|r| r.is_ok()).count(), 
              results.len());
        
        results
    }
    
    /// Enhanced circuit compilation with performance monitoring
    fn compile_circuit_to_wasm(&self, circuit: &ZkCircuit) -> Result<Vec<u8>> {
        let start_time = Instant::now();
        
        info!("Starting circuit compilation for circuit: {}", circuit.id);
        
        // TODO: Placeholder implementation - in a real system this would:
        // 1. Convert causality instructions to SP1 circuit format
        // 2. Compile to WASM bytecode for Valence execution
        // 3. Include verification logic for the circuit constraints
        
        let mut wasm_data = Vec::new();
        wasm_data.extend_from_slice(b"wasm_header");
        wasm_data.extend_from_slice(&circuit.id.as_bytes());
        wasm_data.extend_from_slice(&(circuit.instructions.len() as u32).to_le_bytes());
        wasm_data.extend_from_slice(&(circuit.public_inputs.len() as u32).to_le_bytes());
        
        // Add circuit constraints as metadata
        for constraint in &circuit.constraints {
            wasm_data.extend_from_slice(constraint.as_bytes());
        }
        
        let compilation_time = start_time.elapsed().as_millis() as u64;
        
        // Record compilation metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_circuit_compilation(compilation_time);
        }
        
        info!("Circuit compilation completed for {} in {}ms", circuit.id, compilation_time);
        
        Ok(wasm_data)
    }
    
    /// Enhanced proof generation with retry logic and monitoring
    async fn generate_proof_with_retry(&mut self, circuit: &ZkCircuit, witness: &ZkWitness) -> Result<ZkProofResponse> {
        let mut last_error = None;
        
        info!("Starting proof generation with retry for circuit: {}", circuit.id);
        
        for attempt in 0..self.config.max_retries {
            // Record retry attempt if not first attempt
            if attempt > 0 {
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.record_retry(attempt + 1, "proof_generation");
                }
            }
            
            match self.generate_proof_internal(circuit, witness).await {
                Ok(response) => {
                    info!("Proof generation succeeded on attempt {} for circuit: {}", 
                          attempt + 1, circuit.id);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Proof generation attempt {} failed for circuit {}: {}", 
                          attempt + 1, circuit.id, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries - 1 {
                        let backoff_duration = Duration::from_secs(2_u64.pow(attempt));
                        info!("Retrying proof generation in {:?}", backoff_duration);
                        tokio::time::sleep(backoff_duration).await; // Exponential backoff
                    }
                }
            }
        }
        
        error!("Proof generation failed after {} attempts for circuit: {}", 
               self.config.max_retries, circuit.id);
        
        // Record the final failure
        if let Ok(mut metrics) = self.metrics.lock() {
            let error_msg = last_error.as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string());
            metrics.record_failure("proof_generation", &error_msg);
        }
        
        Err(last_error.unwrap())
    }
    
    /// Internal proof generation implementation with timing
    async fn generate_proof_internal(&mut self, circuit: &ZkCircuit, witness: &ZkWitness) -> Result<ZkProofResponse> {
        let start_time = Instant::now();
        
        debug!("Starting internal proof generation for circuit: {}", circuit.id);
        
        // Ensure circuit is deployed
        let _circuit_id = self.ensure_circuit_deployed(circuit).await?;
        
        // Prepare proof request
        let proof_request = self.prepare_proof_request(circuit, witness)?;
        
        // Generate proof using async method with polling
        let proof_response = self.client.generate_proof_with_polling(proof_request).await?;
        
        let generation_time = start_time.elapsed().as_millis() as u64;
        
        // Record successful proof generation
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_proof_generation(generation_time);
        }
        
        info!("Proof generation completed for circuit {} in {}ms", 
              circuit.id, generation_time);
        
        Ok(proof_response)
    }
    
    /// Prepare circuit deployment request from ZK circuit
    fn prepare_circuit_deployment(&self, circuit: &ZkCircuit) -> Result<DeployCircuitRequest> {
        // Convert ZK circuit to WASM bytecode for Valence deployment
        let wasm_bytecode = self.compile_circuit_to_wasm(circuit)?;
        
        Ok(DeployCircuitRequest {
            name: format!("{}_{}", self.config.circuit_name, circuit.id),
            wasm_bytecode: BASE64.encode(wasm_bytecode),
            description: Some(format!("Causality ZK circuit: {}", circuit.id)),
            parameters: Some([
                ("circuit_id".to_string(), circuit.id.clone()),
                ("backend".to_string(), "valence".to_string()),
                ("public_inputs".to_string(), circuit.public_inputs.len().to_string()),
                ("instructions".to_string(), circuit.instructions.len().to_string()),
            ].into_iter().collect()),
        })
    }
    
    /// Prepare proof request from circuit and witness
    fn prepare_proof_request(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> Result<ZkProofRequest> {
        // Convert public inputs to strings
        let public_inputs: Vec<String> = circuit.public_inputs
            .iter()
            .map(|i| i.to_string())
            .collect();
        
        // Convert private inputs from witness
        let private_inputs: Vec<String> = witness.private_inputs
            .iter()
            .map(|b| BASE64.encode([*b]))
            .collect();
        
        Ok(ZkProofRequest {
            circuit_name: format!("{}_{}", self.config.circuit_name, circuit.id),
            public_inputs,
            private_inputs,
            metadata: Some([
                ("circuit_id".to_string(), circuit.id.clone()),
                ("witness_id".to_string(), witness.id.clone()),
                ("backend".to_string(), "valence".to_string()),
            ].into_iter().collect()),
            priority: Some(ProofPriority::Normal),
        })
    }
    
    /// Ensure circuit is deployed and ready
    async fn ensure_circuit_deployed(&mut self, circuit: &ZkCircuit) -> Result<String> {
        // First check if circuit already exists
        let circuits = self.client.list_circuits().await?;
        let circuit_name = format!("{}_{}", self.config.circuit_name, circuit.id);
        
        if let Some(existing) = circuits.iter().find(|c| c.name == circuit_name) {
            match existing.status {
                CircuitStatus::Ready => return Ok(existing.id.clone()),
                CircuitStatus::Failed => {
                    // Circuit failed, need to redeploy
                }
                CircuitStatus::Deploying => {
                    // Wait for current deployment to complete
                    self.client.wait_for_circuit_ready(&existing.id).await?;
                    return Ok(existing.id.clone());
                }
                CircuitStatus::Unknown => {
                    // Status unknown, try to redeploy
                }
            }
        }
        
        // Deploy new circuit
        if self.config.auto_deploy {
            let deploy_request = self.prepare_circuit_deployment(circuit)?;
            let circuit_info = self.client.deploy_circuit(deploy_request).await?;
            Ok(circuit_info.id)
        } else {
            Err(anyhow::anyhow!("Circuit not deployed and auto-deploy is disabled"))
        }
    }
}

impl Default for ValenceBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkBackend for ValenceBackend {
    fn generate_proof(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> ProofResult<ZkProof> {
        let start_time = Instant::now();
        
        info!("Starting ZK proof generation for circuit: {}", circuit.id);
        
        // Use the async runtime to execute proof generation
        let result = self.runtime.block_on(async {
            // Clone self to get mutable access in the async context
            let mut backend = self.clone();
            
            match backend.generate_proof_with_retry(circuit, witness).await {
                Ok(valence_response) => {
                    // Convert Valence response to our ZkProof format
                    let proof_data = BASE64.decode(&valence_response.proof)
                        .map_err(|e| {
                            let error_msg = format!("Failed to decode proof: {}", e);
                            error!("Proof decoding failed: {}", error_msg);
                            ProofError::GenerationFailed(error_msg)
                        })?;
                    
                    // Convert u32 public inputs to u8 format
                    let public_inputs_bytes: Vec<u8> = circuit.public_inputs
                        .iter()
                        .flat_map(|&input| input.to_le_bytes())
                        .collect();
                    
                    let zk_proof = ZkProof::new(
                        format!("valence_{}_{}", circuit.id, chrono::Utc::now().timestamp()),
                        proof_data,
                        public_inputs_bytes,
                    );
                    
                    info!("Successfully generated ZK proof for circuit: {}", circuit.id);
                    Ok(zk_proof)
                }
                Err(e) => {
                    let error_msg = format!("Valence proof generation failed: {}", e);
                    error!("Proof generation failed for circuit {}: {}", circuit.id, error_msg);
                    Err(ProofError::GenerationFailed(error_msg))
                }
            }
        });
        
        let generation_time = start_time.elapsed().as_millis() as u64;
        
        // Record metrics regardless of success/failure
        if let Ok(mut metrics) = self.metrics.lock() {
            match &result {
                Ok(_) => {
                    metrics.record_proof_generation(generation_time);
                    debug!("Recorded successful proof generation metrics");
                }
                Err(e) => {
                    metrics.record_failure("zk_proof_generation", &e.to_string());
                    debug!("Recorded failed proof generation metrics");
                }
            }
        } else {
            warn!("Failed to update proof generation metrics - mutex lock failed");
        }
        
        result
    }
    
    fn verify_proof(&self, proof: &ZkProof, public_inputs: &[i64]) -> Result<bool, VerificationError> {
        let start_time = Instant::now();
        
        info!("Starting ZK proof verification for proof: {}", proof.id);
        debug!("Verifying proof with {} public inputs", public_inputs.len());
        
        let result = self.runtime.block_on(async {
            // Prepare verification request
            let verify_request = VerifyProofRequest {
                proof: BASE64.encode(&proof.proof_data),
                public_inputs: public_inputs.iter().map(|i| i.to_string()).collect(),
                circuit_name: self.config.circuit_name.clone(),
                verification_key_id: None,
            };
            
            match self.client.verify_proof(verify_request).await {
                Ok(is_valid) => {
                    if is_valid {
                        info!("Proof verification successful for proof: {}", proof.id);
                    } else {
                        warn!("Proof verification failed - proof is invalid: {}", proof.id);
                    }
                    Ok(is_valid)
                }
                Err(e) => {
                    let error_msg = format!("Valence verification failed: {}", e);
                    error!("Proof verification error for proof {}: {}", proof.id, error_msg);
                    Err(VerificationError::InvalidProof(error_msg))
                }
            }
        });
        
        let verification_time = start_time.elapsed().as_millis() as u64;
        
        // Record verification metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            match &result {
                Ok(_) => {
                    metrics.record_verification(verification_time);
                    debug!("Recorded successful verification metrics");
                }
                Err(e) => {
                    metrics.record_failure("zk_proof_verification", &e.to_string());
                    debug!("Recorded failed verification metrics");
                }
            }
        } else {
            warn!("Failed to update verification metrics - mutex lock failed");
        }
        
        result
    }
    
    fn backend_name(&self) -> &'static str {
        "valence"
    }
    
    fn is_available(&self) -> bool {
        let is_available = self.check_service_health();
        debug!("Backend availability check: {}", is_available);
        is_available
    }
}

// Add Clone implementation for ValenceBackend
impl Clone for ValenceBackend {
    fn clone(&self) -> Self {
        Self::with_config(self.config.clone())
    }
}

/// Verification key management for circuits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKeyManager {
    /// Verification keys storage
    pub keys: HashMap<String, VerificationKeyEntry>,
    
    /// Cache for frequently accessed keys
    pub key_cache: HashMap<String, CachedVerificationKey>,
    
    /// Maximum cache size
    max_cache_size: usize,
}

/// Verification key entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKeyEntry {
    /// Circuit identifier
    pub circuit_id: String,
    
    /// Verification key data
    pub key_data: Vec<u8>,
    
    /// Key version
    pub version: String,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last updated timestamp
    pub updated_at: u64,
    
    /// Whether this key is active
    pub is_active: bool,
    
    /// Key metadata
    pub metadata: VerificationKeyMetadata,
}

/// Verification key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKeyMetadata {
    /// Circuit complexity (gate count)
    pub gate_count: Option<u64>,
    
    /// Domain this key is for
    pub domain: String,
    
    /// Circuit type
    pub circuit_type: String,
    
    /// Verification algorithm
    pub algorithm: String,
    
    /// Trusted setup parameters hash
    pub setup_hash: Option<String>,
}

/// Cached verification key for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedVerificationKey {
    /// Key entry
    entry: VerificationKeyEntry,
    
    /// Cache timestamp
    cached_at: u64,
    
    /// Access count for LRU eviction
    access_count: u64,
}

impl VerificationKeyManager {
    /// Create a new verification key manager
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            keys: HashMap::new(),
            key_cache: HashMap::new(),
            max_cache_size,
        }
    }
    
    /// Store a verification key
    pub fn store_verification_key(
        &mut self,
        circuit_id: String,
        key_data: Vec<u8>,
        metadata: VerificationKeyMetadata,
    ) -> Result<String, String> {
        let key_id = self.generate_key_id(&circuit_id, &metadata);
        let now = chrono::Utc::now().timestamp() as u64;
        
        let entry = VerificationKeyEntry {
            circuit_id: circuit_id.clone(),
            key_data,
            version: "1.0".to_string(),
            created_at: now,
            updated_at: now,
            is_active: true,
            metadata,
        };
        
        // Deactivate any existing keys for this circuit
        for (_, existing_entry) in self.keys.iter_mut() {
            if existing_entry.circuit_id == circuit_id {
                existing_entry.is_active = false;
            }
        }
        
        self.keys.insert(key_id.clone(), entry);
        Ok(key_id)
    }
    
    /// Retrieve a verification key by circuit ID
    pub fn get_verification_key(&mut self, circuit_id: &str) -> Option<VerificationKeyEntry> {
        debug!("Retrieving verification key for circuit: {}", circuit_id);
        
        // Check cache first
        if let Some(cached_key) = self.get_from_cache(circuit_id) {
            info!("Cache hit for verification key: {}", circuit_id);
            return Some(cached_key.entry);
        }
        
        info!("Cache miss for verification key: {}", circuit_id);
        
        // Find active key for the circuit
        let mut found_entry: Option<(String, VerificationKeyEntry)> = None;
        for (key_id, entry) in &self.keys {
            if entry.circuit_id == circuit_id && entry.is_active {
                found_entry = Some((key_id.clone(), entry.clone()));
                break;
            }
        }
        
        if let Some((key_id, entry)) = found_entry {
            // Cache the key
            self.cache_key(key_id, entry.clone());
            debug!("Verification key found and cached for circuit: {}", circuit_id);
            Some(entry)
        } else {
            warn!("No verification key found for circuit: {}", circuit_id);
            None
        }
    }
    
    /// Retrieve a verification key by key ID
    pub fn get_verification_key_by_id(&mut self, key_id: &str) -> Option<VerificationKeyEntry> {
        debug!("Retrieving verification key by ID: {}", key_id);
        
        // Check cache first
        if let Some(cached_key) = self.get_from_cache(key_id) {
            info!("Cache hit for verification key ID: {}", key_id);
            return Some(cached_key.entry);
        }
        
        if let Some(entry) = self.keys.get(key_id).cloned() {
            self.cache_key(key_id.to_string(), entry.clone());
            debug!("Verification key found and cached for ID: {}", key_id);
            Some(entry)
        } else {
            warn!("No verification key found for ID: {}", key_id);
            None
        }
    }
    
    /// List all verification keys for a circuit
    pub fn list_circuit_keys(&self, circuit_id: &str) -> Vec<VerificationKeyEntry> {
        self.keys
            .values()
            .filter(|entry| entry.circuit_id == circuit_id)
            .cloned()
            .collect()
    }
    
    /// Update verification key metadata
    pub fn update_key_metadata(
        &mut self,
        key_id: &str,
        metadata: VerificationKeyMetadata,
    ) -> Result<(), String> {
        if let Some(entry) = self.keys.get_mut(key_id) {
            entry.metadata = metadata;
            entry.updated_at = chrono::Utc::now().timestamp() as u64;
            // Invalidate cache for this key
            self.invalidate_cache(key_id);
            Ok(())
        } else {
            Err(format!("Verification key not found: {}", key_id))
        }
    }
    
    /// Delete a verification key
    pub fn delete_verification_key(&mut self, key_id: &str) -> Result<(), String> {
        if self.keys.remove(key_id).is_some() {
            self.invalidate_cache(key_id);
            Ok(())
        } else {
            Err(format!("Verification key not found: {}", key_id))
        }
    }
    
    /// Generate a unique key ID
    fn generate_key_id(&self, circuit_id: &str, metadata: &VerificationKeyMetadata) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(circuit_id.as_bytes());
        hasher.update(&metadata.domain.as_bytes());
        hasher.update(&metadata.circuit_type.as_bytes());
        hasher.update(&metadata.algorithm.as_bytes());
        if let Some(setup_hash) = &metadata.setup_hash {
            hasher.update(setup_hash.as_bytes());
        }
        
        let hash = hasher.finalize();
        format!("vk_{}", hex::encode(&hash[..8]))
    }
    
    /// Get key from cache
    pub fn get_from_cache(&mut self, key_lookup: &str) -> Option<CachedVerificationKey> {
        // Try direct key ID lookup first
        if let Some(cached_key) = self.key_cache.get_mut(key_lookup) {
            cached_key.access_count += 1;
            return Some(cached_key.clone());
        }
        
        // Try circuit ID lookup
        for (_, cached_key) in self.key_cache.iter_mut() {
            if cached_key.entry.circuit_id == key_lookup && cached_key.entry.is_active {
                cached_key.access_count += 1;
                return Some(cached_key.clone());
            }
        }
        
        None
    }
    
    /// Cache a verification key
    pub fn cache_key(&mut self, key_id: String, entry: VerificationKeyEntry) {
        // If cache is full, remove least recently used entry
        if self.key_cache.len() >= self.max_cache_size {
            self.evict_lru_key();
        }
        
        let cached_key = CachedVerificationKey {
            entry,
            cached_at: chrono::Utc::now().timestamp() as u64,
            access_count: 1,
        };
        
        self.key_cache.insert(key_id, cached_key);
    }
    
    /// Invalidate cache entry
    fn invalidate_cache(&mut self, key_id: &str) {
        self.key_cache.remove(key_id);
    }
    
    /// Evict least recently used key from cache
    fn evict_lru_key(&mut self) {
        if let Some(lru_key) = self.key_cache
            .iter()
            .min_by_key(|(_, cached_key)| cached_key.access_count)
            .map(|(key_id, _)| key_id.clone())
        {
            self.key_cache.remove(&lru_key);
        }
    }
}

impl Default for VerificationKeyManager {
    fn default() -> Self {
        Self::new(100) // Default cache size of 100 keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZkCircuit, ZkWitness};
    
    #[test]
    fn test_valence_backend_creation() {
        let backend = ValenceBackend::new();
        assert_eq!(backend.backend_name(), "valence");
        assert_eq!(backend.config.endpoint, "http://prover.timewave.computer:37281");
    }
    
    #[test]
    fn test_valence_backend_with_config() {
        let config = ValenceConfig {
            endpoint: "http://localhost:8080".to_string(),
            circuit_name: "test-circuit".to_string(),
            timeout: Duration::from_secs(600),
            auto_deploy: false,
            ..Default::default()
        };
        
        let backend = ValenceBackend::with_config(config);
        assert_eq!(backend.config.endpoint, "http://localhost:8080");
        assert_eq!(backend.config.circuit_name, "test-circuit");
        assert_eq!(backend.config.timeout, Duration::from_secs(600));
        assert!(!backend.config.auto_deploy);
    }
    
    #[test]
    fn test_circuit_deployment_preparation() {
        let backend = ValenceBackend::new();
        let mut circuit = ZkCircuit::new(vec![], vec![42]);
        circuit.id = "test_circuit".to_string();
        
        let deploy_request = backend.prepare_circuit_deployment(&circuit);
        assert!(deploy_request.is_ok());
        
        let request = deploy_request.unwrap();
        assert_eq!(request.name, "causality-circuit_test_circuit");
        assert!(request.description.is_some());
        assert!(request.parameters.is_some());
    }
    
    #[test]
    fn test_proof_request_preparation() {
        let backend = ValenceBackend::new();
        let mut circuit = ZkCircuit::new(vec![], vec![42]);
        circuit.id = "test_circuit".to_string();
        
        let witness = ZkWitness::new("test_circuit".to_string(), vec![42], vec![1, 2, 3]);
        
        let proof_request = backend.prepare_proof_request(&circuit, &witness);
        assert!(proof_request.is_ok());
        
        let request = proof_request.unwrap();
        assert_eq!(request.circuit_name, "causality-circuit_test_circuit");
        assert!(!request.private_inputs.is_empty());
        assert!(request.metadata.is_some());
    }
    
    #[tokio::test]
    async fn test_circuit_and_witness_structure() {
        let backend = ValenceBackend::new();
        
        let circuit = ZkCircuit::new(
            vec![Instruction::Move { src: RegisterId::new(0), dst: RegisterId::new(1) }],
            vec![5, 6, 7, 8]
        );
        
        let witness = ZkWitness::new(
            "test_witness".to_string(),
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16]
        );
        
        // Test that circuit and witness are properly structured for Valence
        assert!(!circuit.instructions.is_empty());
        assert!(!witness.private_inputs.is_empty());
    }
} 