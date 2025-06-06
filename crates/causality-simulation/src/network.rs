//! Network simulation utilities

/// Simple network simulator
pub struct NetworkSimulator {
    pub latency_ms: u64,
    pub bandwidth_mbps: u32,
}

impl NetworkSimulator {
    pub fn new(latency_ms: u64, bandwidth_mbps: u32) -> Self {
        Self { latency_ms, bandwidth_mbps }
    }
    
    pub async fn simulate_delay(&self) {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;
    }
} 