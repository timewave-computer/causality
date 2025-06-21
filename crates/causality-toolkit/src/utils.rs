//! Utility functions for Causality development

/// Helper function for generating unique identifiers
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = std::time::UNIX_EPOCH
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("id_{}", timestamp)
}

/// Format bytes as hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Parse hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex)
}

/// Test harness for running and managing tests
#[derive(Debug, Clone)]
pub struct TestHarness {
    test_results: std::collections::BTreeMap<String, bool>,
    test_count: usize,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self {
            test_results: std::collections::BTreeMap::new(),
            test_count: 0,
        }
    }
    
    /// Run a test and record the result
    pub fn run_test<F>(&mut self, name: String, test_fn: F) -> bool 
    where
        F: FnOnce() -> bool,
    {
        let result = test_fn();
        self.test_results.insert(name, result);
        self.test_count += 1;
        result
    }
    
    /// Get test result by name
    pub fn get_result(&self, name: &str) -> Option<bool> {
        self.test_results.get(name).copied()
    }
    
    /// Get all test results
    pub fn get_all_results(&self) -> &std::collections::BTreeMap<String, bool> {
        &self.test_results
    }
    
    /// Get the number of tests run
    pub fn test_count(&self) -> usize {
        self.test_count
    }
    
    /// Get the number of passed tests
    pub fn passed_count(&self) -> usize {
        self.test_results.values().filter(|&&result| result).count()
    }
    
    /// Get the number of failed tests
    pub fn failed_count(&self) -> usize {
        self.test_results.values().filter(|&&result| !result).count()
    }
    
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.test_results.values().all(|&result| result)
    }
    
    /// Clear all test results
    pub fn clear(&mut self) {
        self.test_results.clear();
        self.test_count = 0;
    }
    
    /// Run a series of compilation tests
    pub fn run_compilation_tests(&mut self, test_programs: Vec<String>) -> Vec<bool> {
        let mut results = Vec::new();
        
        for (i, program) in test_programs.iter().enumerate() {
            let test_name = format!("compilation_test_{}", i);
            let result = self.run_test(test_name, || {
                // Mock compilation test - just check if the program is not empty
                !program.trim().is_empty()
            });
            results.push(result);
        }
        
        results
    }
    
    /// Run a series of effect tests
    pub fn run_effect_tests(&mut self, test_effects: Vec<(String, Vec<String>)>) -> Vec<bool> {
        let mut results = Vec::new();
        
        for (i, (effect_name, args)) in test_effects.iter().enumerate() {
            let test_name = format!("effect_test_{}_{}", i, effect_name);
            let result = self.run_test(test_name, || {
                // Mock effect test - just check if effect name and args are valid
                !effect_name.is_empty() && !args.is_empty()
            });
            results.push(result);
        }
        
        results
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
} 