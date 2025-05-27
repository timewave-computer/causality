//! AST and effect management
//!
//! This module provides utilities for registering, tracking, and mapping between
//! ASTs and their corresponding effects in a multi-domain program.

//-----------------------------------------------------------------------------
// Import
//-----------------------------------------------------------------------------

use anyhow::{anyhow, Error, Result};

/// Result type for AST operations
type AstResult<T> = Result<T, Error>;
use std::collections::{HashMap, HashSet};

use causality_types::{
    core::id::{DomainId, ExprId},
    expr::ast::Expr,
    expr::value::ValueExpr,
};

use crate::project::ProgramProject;

//-----------------------------------------------------------------------------
// Error Type
//-----------------------------------------------------------------------------

/// Error types for AST operations - defined as error codes and messages
/// to be used with anyhow
pub mod ast_errors {
    pub const NOT_FOUND: u32 = 4201;
    pub const ALREADY_EXISTS: u32 = 4202;
    pub const PARSE_ERROR: u32 = 4203;
    pub const VALIDATION_ERROR: u32 = 4204;
    pub const DOMAIN_NOT_FOUND: u32 = 4205;
    pub const GENERAL: u32 = 4206;

    /// AST not found error message
    pub fn not_found(id: &str) -> String {
        format!("AST not found: {}", id)
    }

    /// AST already exists error message
    pub fn already_exists(id: &str) -> String {
        format!("AST already exists: {}", id)
    }

    /// AST parse error message
    pub fn parse_error(message: &str) -> String {
        format!("AST parse error: {}", message)
    }

    /// AST validation error message
    pub fn validation_error(message: &str) -> String {
        format!("AST validation error: {}", message)
    }

    /// Domain not found error message
    pub fn domain_not_found(domain: &str) -> String {
        format!("Domain not found: {}", domain)
    }

    /// General AST error message
    pub fn general(message: &str) -> String {
        format!("AST error: {}", message)
    }
}

//-----------------------------------------------------------------------------
// AST Management Interface
//-----------------------------------------------------------------------------

/// Interface for AST management operations

pub trait AstManagement {
    /// Create a new AST entry
    fn create_ast(
        &mut self,
        ast_id: &str,
        domain_id: Option<&DomainId>,
        expr: Expr,
    ) -> AstResult<ExprId>;

    /// Get an AST by ID
    fn get_ast(&self, ast_id: &str) -> AstResult<Expr>;

    /// Update an existing AST
    fn update_ast(&mut self, ast_id: &str, expr: Expr) -> AstResult<ExprId>;

    /// Remove an AST
    fn remove_ast(&mut self, ast_id: &str) -> AstResult<Expr>;

    /// Check if an AST exists
    fn has_ast(&self, ast_id: &str) -> bool;

    /// List all AST IDs
    fn list_ast_ids(&self) -> Vec<String>;

    /// List ASTs by domain
    fn list_ast_ids_by_domain(&self, domain_id: &DomainId) -> Vec<String>;
}

//-----------------------------------------------------------------------------
// AST Manager Implementation
//-----------------------------------------------------------------------------

/// Manager for AST operations
pub struct AstManager {
    /// Map of AST ID to Expr
    ast_store: HashMap<String, Expr>,

    /// Map of AST ID to domain ID
    ast_domains: HashMap<String, DomainId>,

    /// Map of domain ID to set of AST IDs
    domain_asts: HashMap<DomainId, HashSet<String>>,

    /// Map of AST ID to Expr ID
    ast_expr_ids: HashMap<String, ExprId>,

    /// Counter for assigning IDs
    next_id: u64,
}

impl Default for AstManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AstManager {
    /// Create a new AST manager
    pub fn new() -> Self {
        Self {
            ast_store: HashMap::new(),
            ast_domains: HashMap::new(),
            domain_asts: HashMap::new(),
            ast_expr_ids: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new AST manager from a program project
    pub fn from_project(project: &ProgramProject) -> Self {
        let manager = Self::new();

        // For the current SMT-backed storage, we'll work with the single domain
        // that the project represents
        let _domain_id = project.domain_id();
        
        // Since the current ProgramProject doesn't store ASTs directly,
        // we'll create an empty manager that can be populated later
        // This is a placeholder implementation for the current architecture
        
        // TODO: If ASTs need to be stored in the project, they should be added
        // to the SMT storage structure or maintained separately
        
        manager
    }

    /// Generate a new unique expression ID
    fn generate_expr_id(&mut self) -> ExprId {
        let id = self.next_id;
        self.next_id += 1;
        ExprId::new([id as u8; 32])
    }
}

impl AstManagement for AstManager {
    fn create_ast(
        &mut self,
        ast_id: &str,
        domain_id: Option<&DomainId>,
        expr: Expr,
    ) -> AstResult<ExprId> {
        // Check if AST already exists
        if self.has_ast(ast_id) {
            return Err(anyhow!(ast_errors::already_exists(ast_id)));
        }

        // Generate a new expression ID
        let expr_id = self.generate_expr_id();

        // Store the AST
        self.ast_store.insert(ast_id.to_string(), expr);
        self.ast_expr_ids.insert(ast_id.to_string(), expr_id);

        // Associate with domain if provided
        if let Some(domain) = domain_id {
            self.ast_domains.insert(ast_id.to_string(), *domain);

            // Make sure domain entry exists
            self.domain_asts
                .entry(*domain)
                .or_default()
                .insert(ast_id.to_string());
        }

        Ok(expr_id)
    }

    fn get_ast(&self, ast_id: &str) -> AstResult<Expr> {
        self.ast_store
            .get(ast_id)
            .cloned()
            .ok_or_else(|| anyhow!(ast_errors::not_found(ast_id)))
    }

    fn update_ast(&mut self, ast_id: &str, expr: Expr) -> AstResult<ExprId> {
        // Check if AST exists
        if !self.has_ast(ast_id) {
            return Err(anyhow!(ast_errors::not_found(ast_id)));
        }

        // Update the AST
        self.ast_store.insert(ast_id.to_string(), expr);

        // Return the expression ID
        self.ast_expr_ids.get(ast_id).cloned().ok_or_else(|| {
            anyhow!(ast_errors::general(&format!(
                "Expression ID not found for AST: {}",
                ast_id
            )))
        })
    }

    fn remove_ast(&mut self, ast_id: &str) -> AstResult<Expr> {
        // Get the AST
        let expr = self.get_ast(ast_id)?;

        // Remove the AST
        self.ast_store.remove(ast_id);

        // Remove from domain mapping if it exists
        if let Some(domain_id) = self.ast_domains.remove(ast_id) {
            if let Some(asts) = self.domain_asts.get_mut(&domain_id) {
                asts.remove(ast_id);

                // Remove domain entry if empty
                if asts.is_empty() {
                    self.domain_asts.remove(&domain_id);
                }
            }
        }

        // Remove expression ID mapping
        self.ast_expr_ids.remove(ast_id);

        Ok(expr)
    }

    fn has_ast(&self, ast_id: &str) -> bool {
        self.ast_store.contains_key(ast_id)
    }

    fn list_ast_ids(&self) -> Vec<String> {
        self.ast_store.keys().cloned().collect()
    }

    fn list_ast_ids_by_domain(&self, domain_id: &DomainId) -> Vec<String> {
        self.domain_asts
            .get(domain_id)
            .map(|asts| asts.iter().cloned().collect())
            .unwrap_or_default()
    }
}

//-----------------------------------------------------------------------------
// Expression Helper
//-----------------------------------------------------------------------------

/// Helper functions for working with Expr
pub struct ExprHelpers;

impl ExprHelpers {
    //-----------------------------------------------------------------------------
    // Expr Creation Helpers
    //-----------------------------------------------------------------------------
    
    /// Create a placeholder expression (for testing/initialization)
    pub fn placeholder() -> Expr {
        Expr::Const(ValueExpr::Nil)
    }

    /// An empty expression (placeholder)
    pub fn empty() -> Expr {
        Self::placeholder()
    }
}

//-----------------------------------------------------------------------------
// Domain AST Management Interface
//-----------------------------------------------------------------------------

#[allow(dead_code)]
pub trait DomainAstManagement {
    /// Register an AST with a domain
    fn register_domain_ast(
        &mut self,
        domain: &DomainId,
        ast_id: &str,
        ast: Expr,
    ) -> AstResult<()>;

    /// Map an AST to an effect within a domain
    fn map_domain_ast_to_effect(
        &mut self,
        domain: &DomainId,
        ast_id: &str,
        effect_id: &str,
    ) -> AstResult<()>;

    /// Get all ASTs for a domain
    fn get_domain_asts(&self, domain: &DomainId) -> AstResult<Vec<String>>;

    /// Get all effects mapped to an AST within a domain
    fn get_domain_effects_for_ast(
        &self,
        domain: &DomainId,
        ast_id: &str,
    ) -> AstResult<Vec<String>>;
}

impl DomainAstManagement for ProgramProject {
    fn register_domain_ast(
        &mut self,
        _domain: &DomainId,
        _ast_id: &str,
        _ast: Expr, 
    ) -> AstResult<()> {
        // Placeholder implementation - domains field does not exist
        Ok(())
    }

    fn map_domain_ast_to_effect(
        &mut self,
        _domain: &DomainId,
        _ast_id: &str,
        _effect_id: &str, 
    ) -> AstResult<()> {
        // Placeholder implementation - domains field does not exist
        Ok(())
    }

    fn get_domain_asts(&self, _domain: &DomainId) -> AstResult<Vec<String>> {
        // Placeholder implementation - domains field does not exist
        Ok(Vec::new())
    }

    fn get_domain_effects_for_ast(
        &self,
        _domain: &DomainId,
        _ast_id: &str,
    ) -> AstResult<Vec<String>> {
        // Placeholder implementation - domains field does not exist
        Ok(Vec::new())
    }
}

//-----------------------------------------------------------------------------
// Internal AST Management Interface
//-----------------------------------------------------------------------------

/// A different AstManagement trait for internal purposes
#[allow(dead_code)]
pub trait ExprAstManagement {
    /// Get AST expression
    fn get_ast(&self, id: &ExprId) -> Option<&Expr>;

    /// Add a new AST expression
    fn add_ast(&mut self, id: ExprId, expr: Expr) -> &mut Self;

    /// Get all expressions
    fn get_all_asts(&self) -> Vec<(&ExprId, &Expr)>;

    /// Check if an AST exists
    fn has_ast(&self, id: &ExprId) -> bool;
}

//-----------------------------------------------------------------------------
// Storage for AST expression
//-----------------------------------------------------------------------------

/// Storage for AST expressions
pub struct AstStorage {
    /// Map of expression ID to expression
    exprs: HashMap<ExprId, Expr>,
}

impl AstStorage {
    /// Create a new empty storage
    pub fn new() -> Self {
        Self {
            exprs: HashMap::new(),
        }
    }
}

impl Default for AstStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprAstManagement for AstStorage {
    fn get_ast(&self, id: &ExprId) -> Option<&Expr> {
        self.exprs.get(id)
    }

    fn add_ast(&mut self, id: ExprId, expr: Expr) -> &mut Self {
        self.exprs.insert(id, expr);
        self
    }

    fn get_all_asts(&self) -> Vec<(&ExprId, &Expr)> {
        self.exprs.iter().collect()
    }

    fn has_ast(&self, id: &ExprId) -> bool {
        self.exprs.contains_key(id)
    }
}

/// Stub implementation for ProgramProject
impl ExprAstManagement for ProgramProject {
    fn get_ast(&self, _id: &ExprId) -> Option<&Expr> {
        None
    }

    fn add_ast(&mut self, _id: ExprId, _expr: Expr) -> &mut Self {
        self
    }

    fn get_all_asts(&self) -> Vec<(&ExprId, &Expr)> {
        Vec::new()
    }

    fn has_ast(&self, _id: &ExprId) -> bool {
        false
    }
}
