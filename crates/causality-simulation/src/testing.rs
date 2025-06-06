//! Testing utilities for Causality protocols

/// Test case runner for effect simulations
pub struct TestRunner {
    pub name: String,
    pub iterations: usize,
}

impl TestRunner {
    pub fn new(name: impl Into<String>, iterations: usize) -> Self {
        Self {
            name: name.into(),
            iterations,
        }
    }
    
    pub fn run<F>(&self, test_fn: F) -> Result<(), anyhow::Error>
    where
        F: Fn(usize) -> Result<(), anyhow::Error>,
    {
        for i in 0..self.iterations {
            test_fn(i)?;
        }
        Ok(())
    }
} 