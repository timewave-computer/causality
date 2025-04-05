// Security measures for execution
// Original file: src/execution/security.rs

// Security module for Causality Content-Addressed Code System
//
// This module provides security mechanisms for execution, including resource limits,
// effect permissions, and isolation of execution contexts.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Instant;

use causality_core::effect::EffectType;
use causality_types::Error;

/// A resource limit specification
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum CPU time in milliseconds
    pub max_cpu_millis: usize,
    /// Maximum I/O operations
    pub max_io_operations: usize,
    /// Maximum effect count
    pub max_effect_count: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_millis: 10000,               // 10 seconds
            max_io_operations: 1000,
            max_effect_count: 1000,
        }
    }
}

/// Current resource usage
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: usize,
    /// CPU time used in milliseconds
    pub cpu_millis: usize,
    /// I/O operations performed
    pub io_operations: usize,
    /// Effects applied
    pub effect_count: usize,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        ResourceUsage {
            memory_bytes: 0,
            cpu_millis: 0,
            io_operations: 0,
            effect_count: 0,
        }
    }
}

impl ResourceUsage {
    /// Check if this usage exceeds the given limits
    pub fn exceeds_limits(&self, limits: &ResourceLimits) -> bool {
        self.memory_bytes > limits.max_memory_bytes
            || self.cpu_millis > limits.max_cpu_millis
            || self.io_operations > limits.max_io_operations
            || self.effect_count > limits.max_effect_count
    }
    
    /// Add another usage to this one
    pub fn add(&mut self, other: &ResourceUsage) {
        self.memory_bytes += other.memory_bytes;
        self.cpu_millis += other.cpu_millis;
        self.io_operations += other.io_operations;
        self.effect_count += other.effect_count;
    }
}

/// A security sandbox for execution
pub struct SecuritySandbox {
    /// Allowed effect types
    allowed_effects: HashSet<EffectType>,
    /// Resource limits
    limits: ResourceLimits,
    /// Current resource usage
    usage: Mutex<ResourceUsage>,
    /// Start time of execution
    start_time: Instant,
    /// Whether the sandbox is currently active
    active: Mutex<bool>,
}

impl SecuritySandbox {
    /// Create a new security sandbox with default settings
    pub fn new() -> Self {
        SecuritySandbox {
            allowed_effects: HashSet::new(),
            limits: ResourceLimits::default(),
            usage: Mutex::new(ResourceUsage::default()),
            start_time: Instant::now(),
            active: Mutex::new(false),
        }
    }
    
    /// Create a new security sandbox with specific limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        SecuritySandbox {
            allowed_effects: HashSet::new(),
            limits,
            usage: Mutex::new(ResourceUsage::default()),
            start_time: Instant::now(),
            active: Mutex::new(false),
        }
    }
    
    /// Allow a specific effect type
    pub fn allow_effect(mut self, effect_type: EffectType) -> Self {
        self.allowed_effects.insert(effect_type);
        self
    }
    
    /// Allow multiple effect types
    pub fn allow_effects(mut self, effect_types: Vec<EffectType>) -> Self {
        for effect_type in effect_types {
            self.allowed_effects.insert(effect_type);
        }
        self
    }
    
    /// Check if an effect is allowed
    pub fn is_effect_allowed(&self, effect_type: &EffectType) -> bool {
        self.allowed_effects.contains(effect_type)
    }
    
    /// Activate the sandbox
    pub fn activate(&self) -> std::result::Result<SandboxGuard, Error> {
        let mut active = self.active.lock().map_err(|_| Error::LockError("Failed to acquire lock for active state".to_string()))?;
        if *active {
            return Err(Error::SystemError("Sandbox is already active".to_string()));
        }
        
        *active = true;
        
        Ok(SandboxGuard {
            sandbox: self,
            start_time: Instant::now(),
        })
    }
    
    /// Check if the sandbox is active
    pub fn is_active(&self) -> std::result::Result<bool, Error> {
        let active = self.active.lock().map_err(|_| Error::LockError("Failed to acquire lock for active state".to_string()))?;
        Ok(*active)
    }
    
    /// Deactivate the sandbox
    fn deactivate(&self) -> std::result::Result<(), Error> {
        let mut active = self.active.lock().map_err(|_| Error::LockError("Failed to acquire lock for active state".to_string()))?;
        *active = false;
        Ok(())
    }
    
    /// Allocate memory in the sandbox
    pub fn allocate_memory(&self, bytes: usize) -> std::result::Result<MemoryGuard, Error> {
        // Check if active
        if !self.is_active()? {
            return Err(Error::SystemError("Sandbox is not active".to_string()));
        }
        
        // Check limits
        let mut usage = self.usage.lock().map_err(|_| Error::LockError("Failed to acquire lock for usage".to_string()))?;
        if usage.memory_bytes + bytes > self.limits.max_memory_bytes {
            return Err(Error::OutOfMemory("Memory limit exceeded".to_string()));
        }
        
        // Allocate memory
        usage.memory_bytes += bytes;
        
        Ok(MemoryGuard {
            sandbox: self,
            bytes,
        })
    }
    
    /// Record an I/O operation
    pub fn record_io_operation(&self) -> std::result::Result<(), Error> {
        // Check if active
        if !self.is_active()? {
            return Err(Error::SystemError("Sandbox is not active".to_string()));
        }
        
        // Check limits
        let mut usage = self.usage.lock().map_err(|_| Error::LockError("Failed to acquire lock for usage".to_string()))?;
        if usage.io_operations + 1 > self.limits.max_io_operations {
            return Err(Error::OperationFailed("I/O operation limit exceeded".to_string()));
        }
        
        // Record operation
        usage.io_operations += 1;
        
        Ok(())
    }
    
    /// Record an effect application
    pub fn record_effect_application(&self, effect_type: &EffectType) -> std::result::Result<(), Error> {
        // Check if active
        if !self.is_active()? {
            return Err(Error::SystemError("Sandbox is not active".to_string()));
        }
        
        // Check if effect is allowed
        if !self.is_effect_allowed(effect_type) {
            return Err(Error::SystemError(format!("Effect {:?} is not allowed", effect_type)));
        }
        
        // Check limits
        let mut usage = self.usage.lock().map_err(|_| Error::LockError("Failed to acquire lock for usage".to_string()))?;
        if usage.effect_count + 1 > self.limits.max_effect_count {
            return Err(Error::OperationFailed("Effect limit exceeded".to_string()));
        }
        
        // Record application
        usage.effect_count += 1;
        
        Ok(())
    }
    
    /// Check current resource usage
    pub fn current_usage(&self) -> std::result::Result<ResourceUsage, Error> {
        let usage = self.usage.lock().map_err(|_| Error::LockError("Failed to acquire lock for usage".to_string()))?;
        Ok(usage.clone())
    }
    
    /// Check if CPU time limit has been exceeded
    pub fn check_cpu_time(&self) -> std::result::Result<(), Error> {
        let elapsed = self.start_time.elapsed().as_millis() as usize;
        let mut usage = self.usage.lock().map_err(|_| Error::LockError("Failed to acquire lock for usage".to_string()))?;
        
        usage.cpu_millis = elapsed;
        
        if elapsed > self.limits.max_cpu_millis {
            return Err(Error::OperationFailed("CPU time limit exceeded".to_string()));
        }
        
        Ok(())
    }
}

impl Default for SecuritySandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// A guard for a security sandbox that deactivates on drop
pub struct SandboxGuard<'a> {
    sandbox: &'a SecuritySandbox,
    start_time: Instant,
}

impl<'a> Drop for SandboxGuard<'a> {
    fn drop(&mut self) {
        // Record CPU time used
        let elapsed = self.start_time.elapsed().as_millis() as usize;
        if let Ok(mut usage) = self.sandbox.usage.lock() {
            usage.cpu_millis += elapsed;
        }
        
        // Deactivate the sandbox
        let _ = self.sandbox.deactivate();
    }
}

/// A guard for allocated memory that frees on drop
pub struct MemoryGuard<'a> {
    sandbox: &'a SecuritySandbox,
    bytes: usize,
}

impl<'a> Drop for MemoryGuard<'a> {
    fn drop(&mut self) {
        // Free memory
        if let Ok(mut usage) = self.sandbox.usage.lock() {
            usage.memory_bytes = usage.memory_bytes.saturating_sub(self.bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits::default();
        let usage = ResourceUsage::default();
        
        assert!(!usage.exceeds_limits(&limits));
        
        let excessive_usage = ResourceUsage {
            memory_bytes: limits.max_memory_bytes + 1,
            ..ResourceUsage::default()
        };
        
        assert!(excessive_usage.exceeds_limits(&limits));
    }
    
    #[test]
    fn test_sandbox_activation() -> std::result::Result<(), Error> {
        let sandbox = SecuritySandbox::new();
        
        assert!(!sandbox.is_active()?);
        
        let guard = sandbox.activate()?;
        assert!(sandbox.is_active()?);
        
        // The guard will deactivate the sandbox when dropped
        drop(guard);
        assert!(!sandbox.is_active()?);
        
        Ok(())
    }
    
    #[test]
    fn test_effect_permissions() {
        let sandbox = SecuritySandbox::new()
            .allow_effect(EffectType::Custom("file.read".to_string()));
        
        assert!(sandbox.is_effect_allowed(&EffectType::Custom("file.read".to_string())));
        assert!(!sandbox.is_effect_allowed(&EffectType::Custom("file.write".to_string())));
    }
    
    #[test]
    fn test_memory_allocation() -> std::result::Result<(), Error> {
        let sandbox = SecuritySandbox::with_limits(ResourceLimits {
            max_memory_bytes: 1024,
            ..ResourceLimits::default()
        });
        
        let _guard = sandbox.activate()?;
        
        let usage_before = sandbox.current_usage()?;
        assert_eq!(usage_before.memory_bytes, 0);
        
        let _memory = sandbox.allocate_memory(512)?;
        
        let usage_after = sandbox.current_usage()?;
        assert_eq!(usage_after.memory_bytes, 512);
        
        // Should succeed
        let _more_memory = sandbox.allocate_memory(512)?;
        
        // Should fail - exceeds limit
        let result = sandbox.allocate_memory(1);
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_memory_guard() -> std::result::Result<(), Error> {
        let sandbox = SecuritySandbox::new();
        let _guard = sandbox.activate()?;
        
        let usage_before = sandbox.current_usage()?;
        
        {
            let _memory = sandbox.allocate_memory(1024)?;
            let usage_during = sandbox.current_usage()?;
            assert!(usage_during.memory_bytes > usage_before.memory_bytes);
        }
        
        // Memory should be freed after guard is dropped
        let usage_after = sandbox.current_usage()?;
        assert_eq!(usage_after.memory_bytes, usage_before.memory_bytes);
        
        Ok(())
    }
    
    #[test]
    fn test_cpu_time_tracking() -> std::result::Result<(), Error> {
        let sandbox = SecuritySandbox::with_limits(ResourceLimits {
            max_cpu_millis: 500,
            ..ResourceLimits::default()
        });
        
        let _guard = sandbox.activate()?;
        
        // Should succeed immediately
        sandbox.check_cpu_time()?;
        
        // CPU time test is hard to reliably test in a unit test
        // without sleeping, which would slow down the test suite
        
        Ok(())
    }
} 