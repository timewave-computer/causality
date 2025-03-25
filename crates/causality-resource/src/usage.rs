// Resource usage tracking and metrics
// Original file: src/resource/usage.rs

// Resource usage module for Causality Content-Addressed Code System
//
// This module provides types for tracking resource usage during execution.

use std::fmt;
use serde::{Serialize, Deserialize};

/// Current resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_millis: usize,
    /// I/O operations performed
    pub io_operations: usize,
    /// Effects applied
    pub effect_count: usize,
}

impl ResourceUsage {
    /// Create a new resource usage
    pub fn new() -> Self {
        ResourceUsage {
            memory_bytes: 0,
            cpu_millis: 0,
            io_operations: 0,
            effect_count: 0,
        }
    }
    
    /// Create a resource usage with specific values
    pub fn with_values(
        memory_bytes: usize,
        cpu_millis: usize,
        io_operations: usize,
        effect_count: usize,
    ) -> Self {
        ResourceUsage {
            memory_bytes,
            cpu_millis,
            io_operations,
            effect_count,
        }
    }
    
    /// Add another usage to this one
    pub fn add(&mut self, other: &ResourceUsage) {
        self.memory_bytes += other.memory_bytes;
        self.cpu_millis += other.cpu_millis;
        self.io_operations += other.io_operations;
        self.effect_count += other.effect_count;
    }
    
    /// Create a new usage by adding two usages together
    pub fn combine(a: &ResourceUsage, b: &ResourceUsage) -> ResourceUsage {
        ResourceUsage {
            memory_bytes: a.memory_bytes + b.memory_bytes,
            cpu_millis: a.cpu_millis + b.cpu_millis,
            io_operations: a.io_operations + b.io_operations,
            effect_count: a.effect_count + b.effect_count,
        }
    }
    
    /// Check if this usage exceeds another
    pub fn exceeds(&self, other: &ResourceUsage) -> bool {
        self.memory_bytes > other.memory_bytes
            || self.cpu_millis > other.cpu_millis
            || self.io_operations > other.io_operations
            || self.effect_count > other.effect_count
    }
    
    /// Calculate the percentage of resources used
    pub fn percentage_of(&self, limit: &ResourceUsage) -> ResourcePercentage {
        ResourcePercentage {
            memory_percent: if limit.memory_bytes > 0 {
                (self.memory_bytes as f64 / limit.memory_bytes as f64) * 100.0
            } else {
                0.0
            },
            cpu_percent: if limit.cpu_millis > 0 {
                (self.cpu_millis as f64 / limit.cpu_millis as f64) * 100.0
            } else {
                0.0
            },
            io_percent: if limit.io_operations > 0 {
                (self.io_operations as f64 / limit.io_operations as f64) * 100.0
            } else {
                0.0
            },
            effect_percent: if limit.effect_count > 0 {
                (self.effect_count as f64 / limit.effect_count as f64) * 100.0
            } else {
                0.0
            },
        }
    }
    
    /// Reset all usage counters to zero
    pub fn reset(&mut self) {
        self.memory_bytes = 0;
        self.cpu_millis = 0;
        self.io_operations = 0;
        self.effect_count = 0;
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResourceUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Memory: {} bytes, CPU: {} ms, I/O: {} ops, Effects: {}",
            self.memory_bytes, self.cpu_millis, self.io_operations, self.effect_count
        )
    }
}

/// Resource usage as percentages of limits
#[derive(Debug, Clone)]
pub struct ResourcePercentage {
    /// Memory usage percentage
    pub memory_percent: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// I/O operations percentage
    pub io_percent: f64,
    /// Effect count percentage
    pub effect_percent: f64,
}

impl ResourcePercentage {
    /// Get the maximum percentage across all resource types
    pub fn max_percentage(&self) -> f64 {
        self.memory_percent
            .max(self.cpu_percent)
            .max(self.io_percent)
            .max(self.effect_percent)
    }
    
    /// Check if any resource is over a specific percentage
    pub fn any_over(&self, percentage: f64) -> bool {
        self.memory_percent > percentage
            || self.cpu_percent > percentage
            || self.io_percent > percentage
            || self.effect_percent > percentage
    }
}

impl fmt::Display for ResourcePercentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Memory: {:.1}%, CPU: {:.1}%, I/O: {:.1}%, Effects: {:.1}%",
            self.memory_percent, self.cpu_percent, self.io_percent, self.effect_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_usage_creation() {
        let usage = ResourceUsage::new();
        
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.cpu_millis, 0);
        assert_eq!(usage.io_operations, 0);
        assert_eq!(usage.effect_count, 0);
        
        let usage2 = ResourceUsage::with_values(1024, 1000, 100, 50);
        
        assert_eq!(usage2.memory_bytes, 1024);
        assert_eq!(usage2.cpu_millis, 1000);
        assert_eq!(usage2.io_operations, 100);
        assert_eq!(usage2.effect_count, 50);
    }
    
    #[test]
    fn test_resource_usage_add() {
        let mut usage1 = ResourceUsage::with_values(1024, 1000, 100, 50);
        let usage2 = ResourceUsage::with_values(2048, 500, 50, 25);
        
        usage1.add(&usage2);
        
        assert_eq!(usage1.memory_bytes, 3072);
        assert_eq!(usage1.cpu_millis, 1500);
        assert_eq!(usage1.io_operations, 150);
        assert_eq!(usage1.effect_count, 75);
    }
    
    #[test]
    fn test_resource_usage_combine() {
        let usage1 = ResourceUsage::with_values(1024, 1000, 100, 50);
        let usage2 = ResourceUsage::with_values(2048, 500, 50, 25);
        
        let combined = ResourceUsage::combine(&usage1, &usage2);
        
        assert_eq!(combined.memory_bytes, 3072);
        assert_eq!(combined.cpu_millis, 1500);
        assert_eq!(combined.io_operations, 150);
        assert_eq!(combined.effect_count, 75);
    }
    
    #[test]
    fn test_resource_usage_exceeds() {
        let usage1 = ResourceUsage::with_values(1024, 1000, 100, 50);
        let usage2 = ResourceUsage::with_values(2048, 500, 50, 25);
        
        assert!(usage2.exceeds(&usage1)); // higher memory
        assert!(!usage1.exceeds(&usage2)); // lower memory but higher CPU/IO/effects
        
        let usage3 = ResourceUsage::with_values(512, 2000, 100, 50);
        assert!(usage3.exceeds(&usage1)); // higher CPU
    }
    
    #[test]
    fn test_resource_percentage() {
        let usage = ResourceUsage::with_values(512, 500, 50, 25);
        let limit = ResourceUsage::with_values(1024, 1000, 100, 100);
        
        let percentage = usage.percentage_of(&limit);
        
        assert_eq!(percentage.memory_percent, 50.0);
        assert_eq!(percentage.cpu_percent, 50.0);
        assert_eq!(percentage.io_percent, 50.0);
        assert_eq!(percentage.effect_percent, 25.0);
        
        assert_eq!(percentage.max_percentage(), 50.0);
        assert!(percentage.any_over(40.0));
        assert!(!percentage.any_over(60.0));
    }
    
    #[test]
    fn test_resource_usage_reset() {
        let mut usage = ResourceUsage::with_values(1024, 1000, 100, 50);
        
        usage.reset();
        
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.cpu_millis, 0);
        assert_eq!(usage.io_operations, 0);
        assert_eq!(usage.effect_count, 0);
    }
} 