//! Valence Account Factory Analysis
//!
//! This module analyzes OCaml programs to detect Valence account factory usage patterns
//! and generates appropriate account configurations for the compilation pipeline.

use std::collections::BTreeMap;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use causality_lisp::ast::{Expr, ExprKind, LispValue};

/// Valence account factory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFactoryConfig {
    pub account_type: String,
    pub owner: String,
    pub libraries: Vec<String>,
    pub permissions: Vec<String>,
}

/// Analysis result for Valence account factory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceAnalysisResult {
    pub account_configs: Vec<AccountFactoryConfig>,
    pub library_approvals: Vec<LibraryApproval>,
    pub transaction_patterns: Vec<TransactionPattern>,
    pub effect_dependencies: Vec<String>,
}

/// Library approval detected in the program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryApproval {
    pub account: String,
    pub library: String,
    pub permissions: Vec<String>,
}

/// Transaction pattern detected in the program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPattern {
    pub operation_type: String,
    pub account: String,
    pub parameters: BTreeMap<String, String>,
}

/// Valence account factory analyzer
pub struct ValenceAnalyzer {
    /// Detected account factory operations
    account_operations: Vec<AccountFactoryConfig>,
    /// Detected library approvals
    library_approvals: Vec<LibraryApproval>,
    /// Detected transaction patterns
    transaction_patterns: Vec<TransactionPattern>,
    /// Effect dependencies
    effect_dependencies: Vec<String>,
}

impl ValenceAnalyzer {
    pub fn new() -> Self {
        Self {
            account_operations: Vec::new(),
            library_approvals: Vec::new(),
            transaction_patterns: Vec::new(),
            effect_dependencies: Vec::new(),
        }
    }
    
    /// Analyze an OCaml program (represented as Causality Lisp AST) for Valence usage
    pub fn analyze_program(&mut self, ast: &Expr) -> Result<ValenceAnalysisResult> {
        self.analyze_expr(ast)?;
        
        Ok(ValenceAnalysisResult {
            account_configs: self.account_operations.clone(),
            library_approvals: self.library_approvals.clone(),
            transaction_patterns: self.transaction_patterns.clone(),
            effect_dependencies: self.effect_dependencies.clone(),
        })
    }
    
    /// Analyze a single expression for Valence patterns
    fn analyze_expr(&mut self, expr: &Expr) -> Result<()> {
        match &expr.kind {
            ExprKind::Apply(func, args) => {
                self.analyze_apply(func, args)?;
            }
            ExprKind::LetUnit(unit_expr, body) => {
                self.analyze_expr(unit_expr)?;
                self.analyze_expr(body)?;
            }
            ExprKind::LetTensor(pair_expr, _left_var, _right_var, body) => {
                self.analyze_expr(pair_expr)?;
                self.analyze_expr(body)?;
            }
            ExprKind::Lambda(_params, body) => {
                self.analyze_expr(body)?;
            }
            ExprKind::Case(expr, _left_var, left_branch, _right_var, right_branch) => {
                self.analyze_expr(expr)?;
                self.analyze_expr(left_branch)?;
                self.analyze_expr(right_branch)?;
            }
            ExprKind::Tensor(left, right) => {
                self.analyze_expr(left)?;
                self.analyze_expr(right)?;
            }
            ExprKind::Inl(expr) | ExprKind::Inr(expr) => {
                self.analyze_expr(expr)?;
            }
            ExprKind::Alloc(expr) | ExprKind::Consume(expr) => {
                self.analyze_expr(expr)?;
            }
            ExprKind::RecordAccess { record, field: _ } => {
                self.analyze_expr(record)?;
            }
            ExprKind::RecordUpdate { record, field: _, value } => {
                self.analyze_expr(record)?;
                self.analyze_expr(value)?;
            }
            _ => {
                // For other expression types, no specific Valence analysis needed
            }
        }
        
        Ok(())
    }
    
    /// Analyze function application for Valence patterns
    fn analyze_apply(&mut self, func: &Expr, args: &[Expr]) -> Result<()> {
        // Check if this is a Valence account factory operation
        if let ExprKind::Var(symbol) = &func.kind {
            match symbol.as_str() {
                "create_account" => {
                    self.analyze_create_account(args)?;
                }
                "approve_library" => {
                    self.analyze_approve_library(args)?;
                }
                "submit_transaction" => {
                    self.analyze_submit_transaction(args)?;
                }
                _ => {
                    // Not a Valence operation, continue analyzing arguments
                    for arg in args {
                        self.analyze_expr(arg)?;
                    }
                }
            }
        } else {
            // Analyze function and arguments recursively
            self.analyze_expr(func)?;
            for arg in args {
                self.analyze_expr(arg)?;
            }
        }
        
        Ok(())
    }
    
    /// Analyze create_account operation
    fn analyze_create_account(&mut self, args: &[Expr]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("create_account requires at least 1 argument"));
        }
        
        // Extract owner from arguments (simplified analysis)
        let owner = self.extract_string_literal(&args[0])
            .unwrap_or_else(|| "unknown_owner".to_string());
        
        let config = AccountFactoryConfig {
            account_type: "factory".to_string(),
            owner,
            libraries: Vec::new(),
            permissions: vec!["create".to_string(), "approve".to_string()],
        };
        
        self.account_operations.push(config);
        self.effect_dependencies.push("account_creation".to_string());
        
        Ok(())
    }
    
    /// Analyze approve_library operation
    fn analyze_approve_library(&mut self, args: &[Expr]) -> Result<()> {
        if args.len() < 2 {
            return Err(anyhow!("approve_library requires at least 2 arguments"));
        }
        
        let account = self.extract_string_literal(&args[0])
            .unwrap_or_else(|| "unknown_account".to_string());
        let library = self.extract_string_literal(&args[1])
            .unwrap_or_else(|| "unknown_library".to_string());
        
        let approval = LibraryApproval {
            account,
            library,
            permissions: vec!["execute".to_string()],
        };
        
        self.library_approvals.push(approval);
        self.effect_dependencies.push("library_approval".to_string());
        
        Ok(())
    }
    
    /// Analyze submit_transaction operation
    fn analyze_submit_transaction(&mut self, args: &[Expr]) -> Result<()> {
        if args.len() < 2 {
            return Err(anyhow!("submit_transaction requires at least 2 arguments"));
        }
        
        let account = self.extract_string_literal(&args[0])
            .unwrap_or_else(|| "unknown_account".to_string());
        
        let mut parameters = BTreeMap::new();
        parameters.insert("account".to_string(), account.clone());
        
        // Analyze operation type from second argument
        let operation_type = if let ExprKind::Var(symbol) = &args[1].kind {
            symbol.as_str().to_string()
        } else {
            "unknown_operation".to_string()
        };
        
        let pattern = TransactionPattern {
            operation_type,
            account,
            parameters,
        };
        
        self.transaction_patterns.push(pattern);
        self.effect_dependencies.push("transaction_submission".to_string());
        
        Ok(())
    }
    
    /// Extract string literal from expression (simplified)
    fn extract_string_literal(&self, expr: &Expr) -> Option<String> {
        match &expr.kind {
            ExprKind::Const(value) => {
                match value {
                    LispValue::String(s) => Some(s.as_str().to_string()),
                    LispValue::Symbol(s) => Some(s.as_str().to_string()),
                    _ => None,
                }
            }
            ExprKind::Var(symbol) => Some(symbol.as_str().to_string()),
            _ => None,
        }
    }
    
    /// Generate Valence account configuration artifacts
    pub fn generate_account_configs(&self) -> Vec<AccountFactoryConfig> {
        self.account_operations.clone()
    }
    
    /// Get detected effect dependencies
    pub fn get_effect_dependencies(&self) -> &[String] {
        &self.effect_dependencies
    }
}

impl Default for ValenceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::system::Str;
    
    fn create_symbol_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Var(Symbol::from(name)),
            ty: None,
            span: None,
        }
    }
    
    fn create_string_literal_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Const(LispValue::String(Str::from(value))),
            ty: None,
            span: None,
        }
    }
    
    fn create_apply_expr(func_name: &str, args: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Apply(
                Box::new(create_symbol_expr(func_name)),
                args,
            ),
            ty: None,
            span: None,
        }
    }
    
    #[test]
    fn test_create_account_analysis() {
        let mut analyzer = ValenceAnalyzer::new();
        
        let expr = create_apply_expr("create_account", vec![
            create_string_literal_expr("owner_keypair")
        ]);
        
        let result = analyzer.analyze_program(&expr).unwrap();
        
        assert_eq!(result.account_configs.len(), 1);
        assert_eq!(result.account_configs[0].account_type, "factory");
        assert_eq!(result.account_configs[0].owner, "owner_keypair");
        assert!(result.effect_dependencies.contains(&"account_creation".to_string()));
    }
    
    #[test]
    fn test_approve_library_analysis() {
        let mut analyzer = ValenceAnalyzer::new();
        
        let expr = create_apply_expr("approve_library", vec![
            create_string_literal_expr("my_account"),
            create_string_literal_expr("SwapLibrary")
        ]);
        
        let result = analyzer.analyze_program(&expr).unwrap();
        
        assert_eq!(result.library_approvals.len(), 1);
        assert_eq!(result.library_approvals[0].account, "my_account");
        assert_eq!(result.library_approvals[0].library, "SwapLibrary");
        assert!(result.effect_dependencies.contains(&"library_approval".to_string()));
    }
    
    #[test]
    fn test_submit_transaction_analysis() {
        let mut analyzer = ValenceAnalyzer::new();
        
        let expr = create_apply_expr("submit_transaction", vec![
            create_string_literal_expr("my_account"),
            create_symbol_expr("transfer")
        ]);
        
        let result = analyzer.analyze_program(&expr).unwrap();
        
        assert_eq!(result.transaction_patterns.len(), 1);
        assert_eq!(result.transaction_patterns[0].account, "my_account");
        assert_eq!(result.transaction_patterns[0].operation_type, "transfer");
        assert!(result.effect_dependencies.contains(&"transaction_submission".to_string()));
    }
    
    #[test]
    fn test_complex_program_analysis() {
        let mut analyzer = ValenceAnalyzer::new();
        
        // Create a complex expression with multiple Valence operations
        let create_account = create_apply_expr("create_account", vec![
            create_string_literal_expr("my_keypair")
        ]);
        
        let approve_library = create_apply_expr("approve_library", vec![
            create_string_literal_expr("my_account"),
            create_string_literal_expr("SwapLibrary")
        ]);
        
        let submit_transaction = create_apply_expr("submit_transaction", vec![
            create_string_literal_expr("my_account"),
            create_symbol_expr("swap")
        ]);
        
        // Create a tensor expression to combine operations
        let complex_expr = Expr {
            kind: ExprKind::Tensor(
                Box::new(create_account),
                Box::new(Expr {
                    kind: ExprKind::Tensor(
                        Box::new(approve_library),
                        Box::new(submit_transaction),
                    ),
                    ty: None,
                    span: None,
                })
            ),
            ty: None,
            span: None,
        };
        
        let result = analyzer.analyze_program(&complex_expr).unwrap();
        
        assert_eq!(result.account_configs.len(), 1);
        assert_eq!(result.library_approvals.len(), 1);
        assert_eq!(result.transaction_patterns.len(), 1);
        assert_eq!(result.effect_dependencies.len(), 3);
    }
} 