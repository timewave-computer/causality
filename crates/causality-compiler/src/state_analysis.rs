// ------------ STATE QUERY ANALYSIS ------------ 
// Purpose: Static analysis to identify required state queries from OCaml programs

use std::collections::{BTreeMap, BTreeSet};
use causality_lisp::ast::{Expr, ExprKind, LispValue};
use causality_core::lambda::Symbol;
use serde::{Deserialize, Serialize};

/// Represents a state query requirement identified during static analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateQueryRequirement {
    /// The contract address or identifier
    pub contract: String,
    /// The storage slot or field being queried
    pub storage_slot: String,
    /// The blockchain domain (ethereum, cosmos, etc.)
    pub domain: String,
    /// The type of query (balance, allowance, etc.)
    pub query_type: QueryType,
    /// Whether this query is used in conditional logic
    pub is_conditional: bool,
}

/// Types of state queries we can detect
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QueryType {
    /// Token balance query
    TokenBalance,
    /// Token allowance query
    TokenAllowance,
    /// Contract storage slot
    StorageSlot(String),
    /// General contract state query
    ContractState,
    /// Event log query
    EventLog,
    /// Custom query pattern
    Custom(String),
}

/// Analysis result containing all detected state query requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAnalysisResult {
    /// All state queries required by the program
    pub required_queries: Vec<StateQueryRequirement>,
    /// Queries grouped by contract for schema generation
    pub queries_by_contract: BTreeMap<String, Vec<StateQueryRequirement>>,
    /// Queries grouped by domain for cross-chain coordination
    pub queries_by_domain: BTreeMap<String, Vec<StateQueryRequirement>>,
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

/// Metadata about the analysis process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Number of expressions analyzed
    pub expressions_analyzed: usize,
    /// Number of query patterns detected
    pub patterns_detected: usize,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
}

/// State query analyzer that performs static analysis on OCaml programs
pub struct StateQueryAnalyzer {
    /// Detected query requirements
    requirements: BTreeSet<StateQueryRequirement>,
    /// Current analysis context
    context: AnalysisContext,
    /// Pattern matchers for different query types
    patterns: QueryPatternMatcher,
}

/// Analysis context tracking current scope and conditions
#[derive(Debug, Clone)]
struct AnalysisContext {
    /// Current function scope
    current_function: Option<String>,
    /// Whether we're inside conditional logic
    in_conditional: bool,
    /// Current blockchain domain context
    current_domain: Option<String>,
}

/// Pattern matcher for identifying different types of state queries
struct QueryPatternMatcher {
    /// Known query function names
    query_functions: BTreeSet<String>,
    /// Contract address patterns
    contract_patterns: BTreeMap<String, String>,
}

impl StateQueryAnalyzer {
    /// Create a new state query analyzer
    pub fn new() -> Self {
        let mut query_functions = BTreeSet::new();
        query_functions.insert("query_state".to_string());
        query_functions.insert("get_balance".to_string());
        query_functions.insert("get_allowance".to_string());
        query_functions.insert("read_storage".to_string());
        
        let mut contract_patterns = BTreeMap::new();
        contract_patterns.insert("usdc".to_string(), "0xa0b86a33e6ba3e0e4ca4ba5d4e6b3e4c4d5e6f7".to_string());
        contract_patterns.insert("weth".to_string(), "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string());
        
        Self {
            requirements: BTreeSet::new(),
            context: AnalysisContext {
                current_function: None,
                in_conditional: false,
                current_domain: None,
            },
            patterns: QueryPatternMatcher {
                query_functions,
                contract_patterns,
            },
        }
    }
    
    /// Analyze an OCaml program and identify state query requirements
    pub fn analyze_program(&mut self, expr: &Expr) -> StateAnalysisResult {
        let start_time = std::time::Instant::now();
        let mut expressions_analyzed = 0;
        
        self.analyze_expression(expr, &mut expressions_analyzed);
        
        let analysis_duration = start_time.elapsed().as_millis() as u64;
        
        // Group requirements by contract and domain
        let mut queries_by_contract = BTreeMap::new();
        let mut queries_by_domain = BTreeMap::new();
        
        for req in &self.requirements {
            queries_by_contract
                .entry(req.contract.clone())
                .or_insert_with(Vec::new)
                .push(req.clone());
                
            queries_by_domain
                .entry(req.domain.clone())
                .or_insert_with(Vec::new)
                .push(req.clone());
        }
        
        StateAnalysisResult {
            required_queries: self.requirements.iter().cloned().collect(),
            queries_by_contract,
            queries_by_domain,
            metadata: AnalysisMetadata {
                expressions_analyzed,
                patterns_detected: self.requirements.len(),
                analysis_duration_ms: analysis_duration,
            },
        }
    }
    
    /// Analyze a single expression for state query patterns
    fn analyze_expression(&mut self, expr: &Expr, counter: &mut usize) {
        *counter += 1;
        
        match &expr.kind {
            ExprKind::Apply(func, args) => {
                self.analyze_function_application(func, args);
                self.analyze_expression(func, counter);
                for arg in args {
                    self.analyze_expression(arg, counter);
                }
            }
            ExprKind::Lambda(_, body) => {
                self.analyze_expression(body, counter);
            }
            ExprKind::Case(case_expr, _, left_branch, _, right_branch) => {
                // Mark that we're entering conditional logic
                let prev_conditional = self.context.in_conditional;
                self.context.in_conditional = true;
                
                self.analyze_expression(case_expr, counter);
                self.analyze_expression(left_branch, counter);
                self.analyze_expression(right_branch, counter);
                
                self.context.in_conditional = prev_conditional;
            }
            ExprKind::Const(_) | ExprKind::Var(_) | ExprKind::UnitVal => {
                // Base cases - no further analysis needed
            }
            ExprKind::Tensor(left, right) => {
                self.analyze_expression(left, counter);
                self.analyze_expression(right, counter);
            }
            ExprKind::LetTensor(binding, _, _, body) => {
                self.analyze_expression(binding, counter);
                self.analyze_expression(body, counter);
            }
            ExprKind::LetUnit(binding, body) => {
                self.analyze_expression(binding, counter);
                self.analyze_expression(body, counter);
            }
            ExprKind::Inl(expr) | ExprKind::Inr(expr) => {
                self.analyze_expression(expr, counter);
            }
            ExprKind::Alloc(expr) | ExprKind::Consume(expr) => {
                self.analyze_expression(expr, counter);
            }
            ExprKind::RecordAccess { record, .. } => {
                self.analyze_expression(record, counter);
            }
            ExprKind::RecordUpdate { record, value, .. } => {
                self.analyze_expression(record, counter);
                self.analyze_expression(value, counter);
            }            // Handle all other expression kinds including session operations
            _ => { /* No state queries in other expression types */ }
        }
    }
    
    /// Analyze function applications for state query patterns
    fn analyze_function_application(&mut self, func: &Expr, args: &[Expr]) {
        if let ExprKind::Var(func_name) = &func.kind {
            if self.patterns.query_functions.contains(func_name.as_str()) {
                self.detect_state_query(func_name, args);
            }
        }
    }
    
    /// Detect and record state query requirements
    fn detect_state_query(&mut self, func_name: &Symbol, args: &[Expr]) {
        match func_name.as_str() {
            "query_state" => self.detect_generic_query(args),
            "get_balance" => self.detect_balance_query(args),
            "get_allowance" => self.detect_allowance_query(args),
            "read_storage" => self.detect_storage_query(args),
            _ => {}
        }
    }
    
    /// Detect generic state query pattern
    fn detect_generic_query(&mut self, args: &[Expr]) {
        if args.len() >= 2 {
            if let (Some(contract), Some(slot)) = (
                self.extract_string_literal(&args[0]),
                self.extract_string_literal(&args[1])
            ) {
                let domain = self.context.current_domain.clone()
                    .unwrap_or_else(|| "ethereum".to_string());
                
                let requirement = StateQueryRequirement {
                    contract,
                    storage_slot: slot,
                    domain,
                    query_type: QueryType::StorageSlot("generic".to_string()),
                    is_conditional: self.context.in_conditional,
                };
                
                self.requirements.insert(requirement);
            }
        }
    }
    
    /// Detect token balance query pattern
    fn detect_balance_query(&mut self, args: &[Expr]) {
        if !args.is_empty() {
            if let Some(contract) = self.extract_string_literal(&args[0]) {
                let domain = self.context.current_domain.clone()
                    .unwrap_or_else(|| "ethereum".to_string());
                
                let requirement = StateQueryRequirement {
                    contract,
                    storage_slot: "balances".to_string(),
                    domain,
                    query_type: QueryType::TokenBalance,
                    is_conditional: self.context.in_conditional,
                };
                
                self.requirements.insert(requirement);
            }
        }
    }
    
    /// Detect token allowance query pattern
    fn detect_allowance_query(&mut self, args: &[Expr]) {
        if !args.is_empty() {
            if let Some(contract) = self.extract_string_literal(&args[0]) {
                let domain = self.context.current_domain.clone()
                    .unwrap_or_else(|| "ethereum".to_string());
                
                let requirement = StateQueryRequirement {
                    contract,
                    storage_slot: "allowances".to_string(),
                    domain,
                    query_type: QueryType::TokenAllowance,
                    is_conditional: self.context.in_conditional,
                };
                
                self.requirements.insert(requirement);
            }
        }
    }
    
    /// Detect storage slot query pattern
    fn detect_storage_query(&mut self, args: &[Expr]) {
        if args.len() >= 2 {
            if let (Some(contract), Some(slot)) = (
                self.extract_string_literal(&args[0]),
                self.extract_string_literal(&args[1])
            ) {
                let domain = self.context.current_domain.clone()
                    .unwrap_or_else(|| "ethereum".to_string());
                
                let requirement = StateQueryRequirement {
                    contract,
                    storage_slot: slot.clone(),
                    domain,
                    query_type: QueryType::StorageSlot(slot),
                    is_conditional: self.context.in_conditional,
                };
                
                self.requirements.insert(requirement);
            }
        }
    }
    
    /// Extract string literal from expression
    fn extract_string_literal(&self, expr: &Expr) -> Option<String> {
        match &expr.kind {
            ExprKind::Const(LispValue::String(s)) => Some(s.value.clone()),
            ExprKind::Const(LispValue::Symbol(sym)) => {
                // Check if this is a known contract symbol
                self.patterns.contract_patterns.get(sym.as_str()).cloned()
                    .or_else(|| Some(sym.as_str().to_string()))
            }
            _ => None,
        }
    }
}

impl Default for StateQueryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryType {
    /// Get the type name as a string
    pub fn type_name(&self) -> String {
        match self {
            QueryType::TokenBalance => "token_balance".to_string(),
            QueryType::TokenAllowance => "token_allowance".to_string(),
            QueryType::StorageSlot(slot) => format!("storage_slot_{}", slot),
            QueryType::ContractState => "contract_state".to_string(),
            QueryType::EventLog => "event_log".to_string(),
            QueryType::Custom(name) => format!("custom_{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_lisp::ast::helpers::*;
    
    #[test]
    fn test_balance_query_detection() {
        let mut analyzer = StateQueryAnalyzer::new();
        
        // Create a simple balance query expression manually
        let program = Expr::apply(
            Expr::variable("get_balance"),
            vec![string("usdc")]
        );
        let result = analyzer.analyze_program(&program);
        
        assert_eq!(result.required_queries.len(), 1);
        let query = &result.required_queries[0];
        assert_eq!(query.query_type, QueryType::TokenBalance);
        assert_eq!(query.storage_slot, "balances");
        assert!(!query.is_conditional);
    }
    
    #[test]
    fn test_conditional_query_detection() {
        let mut analyzer = StateQueryAnalyzer::new();
        
        // Create a conditional query expression
        let program = Expr::case(
            Expr::apply(Expr::variable("get_balance"), vec![string("usdc")]),
            "zero",
            unit(),
            "positive", 
            Expr::apply(Expr::variable("get_allowance"), vec![string("usdc")])
        );
        let result = analyzer.analyze_program(&program);
        
        assert!(result.required_queries.len() >= 1);
        // Should detect queries in conditional context
        let conditional_queries: Vec<_> = result.required_queries.iter()
            .filter(|q| q.is_conditional)
            .collect();
        assert!(!conditional_queries.is_empty());
    }
    
    #[test]
    fn test_multiple_contract_queries() {
        let mut analyzer = StateQueryAnalyzer::new();
        
        // Create multiple contract queries
        let program = Expr::tensor(
            Expr::apply(Expr::variable("get_balance"), vec![string("usdc")]),
            Expr::apply(Expr::variable("get_balance"), vec![string("weth")])
        );
        let result = analyzer.analyze_program(&program);
        
        assert_eq!(result.required_queries.len(), 2);
        assert_eq!(result.queries_by_contract.len(), 2);
    }
    
    #[test]
    fn test_storage_slot_query() {
        let mut analyzer = StateQueryAnalyzer::new();
        
        // Create a storage slot query
        let program = Expr::apply(
            Expr::variable("read_storage"),
            vec![string("0x123"), string("slot_5")]
        );
        let result = analyzer.analyze_program(&program);
        
        assert_eq!(result.required_queries.len(), 1);
        let query = &result.required_queries[0];
        assert_eq!(query.contract, "0x123");
        assert_eq!(query.storage_slot, "slot_5");
        assert!(matches!(query.query_type, QueryType::StorageSlot(_)));
    }
} 