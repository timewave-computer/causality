// Role-based actor functionality
// Original file: src/actor/role.rs

// Actor Roles and Capabilities Module
//
// This module defines the roles and capabilities for actors in the Causality system.

use std::collections::HashSet;
use serde::{Serialize, Deserialize};

/// Actor capability representing actions an actor can perform
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorCapability {
    /// Create programs
    CreateProgram,
    /// Deploy programs
    DeployProgram,
    /// Upgrade programs
    UpgradeProgram,
    /// Verify facts
    VerifyFact,
    /// Execute programs
    ExecuteProgram,
    /// Manage users
    ManageUsers,
    /// Manage governance
    ManageGovernance,
    /// Operate nodes
    OperateNode,
    /// Audit system
    Audit,
    /// Custom capability
    Custom(String),
}

/// Actor role representing a defined set of responsibilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorRole {
    /// End user
    User,
    /// Committee member
    CommitteeMember,
    /// Operator of a node
    Operator,
    /// System administrator
    Admin,
    /// Custom role
    Custom(String),
}

impl ActorRole {
    /// Get the capabilities associated with this role
    pub fn capabilities(&self) -> HashSet<ActorCapability> {
        match self {
            ActorRole::User => {
                let mut capabilities = HashSet::new();
                capabilities.insert(ActorCapability::CreateProgram);
                capabilities.insert(ActorCapability::DeployProgram);
                capabilities.insert(ActorCapability::UpgradeProgram);
                capabilities.insert(ActorCapability::ExecuteProgram);
                capabilities
            }
            ActorRole::CommitteeMember => {
                let mut capabilities = HashSet::new();
                capabilities.insert(ActorCapability::VerifyFact);
                capabilities.insert(ActorCapability::ManageGovernance);
                capabilities.insert(ActorCapability::Audit);
                capabilities
            }
            ActorRole::Operator => {
                let mut capabilities = HashSet::new();
                capabilities.insert(ActorCapability::OperateNode);
                capabilities.insert(ActorCapability::Audit);
                capabilities
            }
            ActorRole::Admin => {
                let mut capabilities = HashSet::new();
                capabilities.insert(ActorCapability::CreateProgram);
                capabilities.insert(ActorCapability::DeployProgram);
                capabilities.insert(ActorCapability::UpgradeProgram);
                capabilities.insert(ActorCapability::VerifyFact);
                capabilities.insert(ActorCapability::ExecuteProgram);
                capabilities.insert(ActorCapability::ManageUsers);
                capabilities.insert(ActorCapability::ManageGovernance);
                capabilities.insert(ActorCapability::OperateNode);
                capabilities.insert(ActorCapability::Audit);
                capabilities
            }
            ActorRole::Custom(_) => {
                // Custom roles have no default capabilities
                HashSet::new()
            }
        }
    }
    
    /// Check if the role has a specific capability
    pub fn has_capability(&self, capability: &ActorCapability) -> bool {
        self.capabilities().contains(capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_capabilities() {
        let role = ActorRole::User;
        assert!(role.has_capability(&ActorCapability::CreateProgram));
        assert!(role.has_capability(&ActorCapability::DeployProgram));
        assert!(role.has_capability(&ActorCapability::UpgradeProgram));
        assert!(role.has_capability(&ActorCapability::ExecuteProgram));
        assert!(!role.has_capability(&ActorCapability::VerifyFact));
        assert!(!role.has_capability(&ActorCapability::ManageUsers));
    }
    
    #[test]
    fn test_committee_member_capabilities() {
        let role = ActorRole::CommitteeMember;
        assert!(role.has_capability(&ActorCapability::VerifyFact));
        assert!(role.has_capability(&ActorCapability::ManageGovernance));
        assert!(role.has_capability(&ActorCapability::Audit));
        assert!(!role.has_capability(&ActorCapability::CreateProgram));
        assert!(!role.has_capability(&ActorCapability::ManageUsers));
    }
    
    #[test]
    fn test_operator_capabilities() {
        let role = ActorRole::Operator;
        assert!(role.has_capability(&ActorCapability::OperateNode));
        assert!(role.has_capability(&ActorCapability::Audit));
        assert!(!role.has_capability(&ActorCapability::ManageUsers));
        assert!(!role.has_capability(&ActorCapability::CreateProgram));
    }
    
    #[test]
    fn test_admin_capabilities() {
        let role = ActorRole::Admin;
        assert!(role.has_capability(&ActorCapability::CreateProgram));
        assert!(role.has_capability(&ActorCapability::DeployProgram));
        assert!(role.has_capability(&ActorCapability::UpgradeProgram));
        assert!(role.has_capability(&ActorCapability::VerifyFact));
        assert!(role.has_capability(&ActorCapability::ExecuteProgram));
        assert!(role.has_capability(&ActorCapability::ManageUsers));
        assert!(role.has_capability(&ActorCapability::ManageGovernance));
        assert!(role.has_capability(&ActorCapability::OperateNode));
        assert!(role.has_capability(&ActorCapability::Audit));
    }
    
    #[test]
    fn test_custom_role() {
        let role = ActorRole::Custom("TestRole".to_string());
        assert!(!role.has_capability(&ActorCapability::CreateProgram));
        assert!(!role.has_capability(&ActorCapability::ManageUsers));
        
        // Custom roles have no capabilities by default
        assert_eq!(role.capabilities().len(), 0);
    }
} 