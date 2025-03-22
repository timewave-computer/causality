// Resource guard implementation for TEL
//
// This module provides the ResourceGuard which enables safe and controlled
// access to resources through the register-based model.

use std::sync::{Arc, RwLock};
use std::fmt;
use std::ops::{Deref, DerefMut, Drop};

use crate::tel::types::{ResourceId, Address};
use crate::tel::error::{TelError, TelResult};
use crate::tel::resource::model::{Register, RegisterId, RegisterContents, RegisterState};
use super::manager::ResourceManager;

/// Types of access to a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Read-only access
    ReadOnly,
    /// Read-write access
    ReadWrite,
}

/// Access control list entry
#[derive(Debug, Clone)]
pub struct AclEntry {
    /// Address that is granted access
    pub address: Address,
    /// Type of access
    pub access_mode: AccessMode,
    /// When the access expires (if temporary)
    pub expiry: Option<u64>,
}

/// Resource access control
pub struct ResourceAccessControl {
    /// The resource manager
    resource_manager: Arc<ResourceManager>,
    /// Access control lists for registers
    acls: RwLock<Vec<(RegisterId, Vec<AclEntry>)>>,
}

impl ResourceAccessControl {
    /// Create a new resource access control
    pub fn new(resource_manager: Arc<ResourceManager>) -> Self {
        Self {
            resource_manager,
            acls: RwLock::new(Vec::new()),
        }
    }
    
    /// Grant access to a register
    pub fn grant_access(
        &self,
        register_id: &RegisterId,
        address: Address,
        mode: AccessMode,
        expiry: Option<u64>,
    ) -> TelResult<()> {
        // First check if the register exists
        let _ = self.resource_manager.get_register(register_id)?;
        
        let mut acls = self.acls.write().map_err(|_| 
            TelError::InternalError("Failed to acquire ACL lock".to_string()))?;
            
        // Find or create ACL entry
        let mut found = false;
        for (id, entries) in acls.iter_mut() {
            if id == register_id {
                // Update existing entry if found
                let mut entry_found = false;
                for entry in entries.iter_mut() {
                    if entry.address == address {
                        entry.access_mode = mode;
                        entry.expiry = expiry;
                        entry_found = true;
                        break;
                    }
                }
                
                // Add new entry if not found
                if !entry_found {
                    entries.push(AclEntry {
                        address,
                        access_mode: mode,
                        expiry,
                    });
                }
                
                found = true;
                break;
            }
        }
        
        // Add new register ACL if not found
        if !found {
            acls.push((
                *register_id,
                vec![AclEntry {
                    address,
                    access_mode: mode,
                    expiry,
                }],
            ));
        }
        
        Ok(())
    }
    
    /// Revoke access to a register
    pub fn revoke_access(
        &self,
        register_id: &RegisterId,
        address: &Address,
    ) -> TelResult<()> {
        let mut acls = self.acls.write().map_err(|_| 
            TelError::InternalError("Failed to acquire ACL lock".to_string()))?;
            
        // Find register ACL
        for (id, entries) in acls.iter_mut() {
            if id == register_id {
                // Remove matching entries
                entries.retain(|entry| &entry.address != address);
                return Ok(());
            }
        }
        
        // Register not found in ACL
        Err(TelError::ResourceError(format!(
            "Register {:?} not found in ACL", register_id
        )))
    }
    
    /// Check if an address has access to a register
    pub fn check_access(
        &self,
        register_id: &RegisterId,
        address: &Address,
        required_mode: AccessMode,
    ) -> TelResult<bool> {
        // First check if the register exists
        let register = self.resource_manager.get_register(register_id)?;
        
        // Owner always has full access
        if &register.owner == address {
            return Ok(true);
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        let acls = self.acls.read().map_err(|_| 
            TelError::InternalError("Failed to acquire ACL lock".to_string()))?;
            
        // Find register ACL
        for (id, entries) in acls.iter() {
            if id == register_id {
                // Check entries
                for entry in entries {
                    if &entry.address == address {
                        // Check expiry
                        if let Some(expiry) = entry.expiry {
                            if expiry <= now {
                                // Entry expired
                                continue;
                            }
                        }
                        
                        // Check access mode
                        match (entry.access_mode, required_mode) {
                            (AccessMode::ReadWrite, _) => return Ok(true), // Read-write implies read-only
                            (AccessMode::ReadOnly, AccessMode::ReadOnly) => return Ok(true),
                            (AccessMode::ReadOnly, AccessMode::ReadWrite) => return Ok(false),
                        }
                    }
                }
                
                // No matching entry found
                return Ok(false);
            }
        }
        
        // Register not found in ACL
        Ok(false)
    }
    
    /// Acquire a register guard
    pub fn acquire_guard(
        &self,
        register_id: &RegisterId,
        address: &Address,
        mode: AccessMode,
    ) -> TelResult<ResourceGuard> {
        // Check if address has access
        if !self.check_access(register_id, address, mode)? {
            return Err(TelError::AuthorizationError(format!(
                "Address {:?} does not have {:?} access to register {:?}",
                address, mode, register_id
            )));
        }
        
        // Get the register
        let register = self.resource_manager.get_register(register_id)?;
        
        // Check if register is active
        if !register.is_active() {
            return Err(TelError::ResourceError(format!(
                "Register {:?} is not in active state", register_id
            )));
        }
        
        // Create guard
        Ok(ResourceGuard {
            register,
            register_id: *register_id,
            mode,
            resource_manager: Arc::clone(&self.resource_manager),
        })
    }
    
    /// Clean up expired ACL entries
    pub fn cleanup_expired_entries(&self) -> TelResult<usize> {
        let mut count = 0;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        let mut acls = self.acls.write().map_err(|_| 
            TelError::InternalError("Failed to acquire ACL lock".to_string()))?;
            
        // Remove expired entries
        for (_, entries) in acls.iter_mut() {
            let original_len = entries.len();
            entries.retain(|entry| {
                if let Some(expiry) = entry.expiry {
                    expiry > now
                } else {
                    true
                }
            });
            
            count += original_len - entries.len();
        }
        
        // Remove empty ACLs
        acls.retain(|(_, entries)| !entries.is_empty());
        
        Ok(count)
    }
}

/// A guard for safe access to a register
pub struct ResourceGuard {
    /// The register being guarded
    register: Register,
    /// ID of the register
    register_id: RegisterId,
    /// Access mode
    mode: AccessMode,
    /// Resource manager
    resource_manager: Arc<ResourceManager>,
}

impl ResourceGuard {
    /// Get the register ID
    pub fn register_id(&self) -> RegisterId {
        self.register_id
    }
    
    /// Get the access mode
    pub fn access_mode(&self) -> AccessMode {
        self.mode
    }
    
    /// Check if this guard has write access
    pub fn can_write(&self) -> bool {
        self.mode == AccessMode::ReadWrite
    }
    
    /// Update the register contents
    pub fn update_contents(&mut self, contents: RegisterContents) -> TelResult<()> {
        if self.mode != AccessMode::ReadWrite {
            return Err(TelError::AuthorizationError(
                "Cannot update register with read-only access".to_string()
            ));
        }
        
        self.register.contents = contents.clone();
        self.resource_manager.update_register(&self.register_id, contents)
    }
}

impl Deref for ResourceGuard {
    type Target = Register;
    
    fn deref(&self) -> &Self::Target {
        &self.register
    }
}

impl DerefMut for ResourceGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.mode != AccessMode::ReadWrite {
            panic!("Cannot get mutable reference with read-only access");
        }
        
        &mut self.register
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        // If changes were made with write access, update the register
        if self.mode == AccessMode::ReadWrite {
            let _ = self.resource_manager.update_register(&self.register_id, self.register.contents.clone());
        }
    }
}

impl fmt::Debug for ResourceGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceGuard")
            .field("register_id", &self.register_id)
            .field("mode", &self.mode)
            .field("register", &self.register)
            .finish()
    }
}

/// A shared resource manager with access control
pub struct SharedResourceManager {
    /// The resource manager
    manager: Arc<ResourceManager>,
    /// The access control
    access_control: Arc<ResourceAccessControl>,
}

impl SharedResourceManager {
    /// Create a new shared resource manager
    pub fn new() -> Self {
        let manager = Arc::new(ResourceManager::new());
        let access_control = Arc::new(ResourceAccessControl::new(Arc::clone(&manager)));
        
        Self {
            manager,
            access_control,
        }
    }
    
    /// Get a reference to the resource manager
    pub fn manager(&self) -> &Arc<ResourceManager> {
        &self.manager
    }
    
    /// Get a reference to the access control
    pub fn access_control(&self) -> &Arc<ResourceAccessControl> {
        &self.access_control
    }
    
    /// Acquire a register guard
    pub fn acquire_guard(
        &self,
        register_id: &RegisterId,
        address: &Address,
        mode: AccessMode,
    ) -> TelResult<ResourceGuard> {
        self.access_control.acquire_guard(register_id, address, mode)
    }
}

impl Default for SharedResourceManager {
    fn default() -> Self {
        Self::new()
    }
} 