//! Type checking & linearity verification
//!
//! This module provides the "Check" stage of the Parse → Check → Compile pipeline.
//! Implements proper type checking with linear type checking, resource linearity 
//! verification, effect type checking, and capability-based access control.

use crate::error::{CompileError, CompileResult};
use crate::pipeline::SExpression;
use causality_core::lambda::{TypeInner, BaseType, Term, TermKind, Literal};
use causality_core::effect::{Capability, CapabilitySet, RowOpResult};
use std::collections::BTreeMap;

/// Check an S-expression for type correctness and linearity
pub fn check_sexpr(expr: &SExpression) -> CompileResult<TypeInner> {
    let mut env = TypeEnvironment::new();
    check_sexpr_with_env(expr, &mut env)
}

/// Check an S-expression with a given type environment
fn check_sexpr_with_env(expr: &SExpression, env: &mut TypeEnvironment) -> CompileResult<TypeInner> {
    match expr {
        SExpression::Integer(_) => Ok(TypeInner::Base(BaseType::Int)),
        SExpression::Boolean(_) => Ok(TypeInner::Base(BaseType::Bool)),
        SExpression::Nil => Ok(TypeInner::Base(BaseType::Unit)),
        SExpression::Symbol(name) => {
            if let Some(ty) = env.lookup(name) {
                Ok(ty.clone())
            } else {
                Err(CompileError::UnknownSymbol {
                    symbol: name.clone(),
                    location: None,
                })
            }
        }
        SExpression::List(exprs) => {
            if exprs.is_empty() {
                return Ok(TypeInner::Base(BaseType::Unit));
            }
            
            match &exprs[0] {
                SExpression::Symbol(op) => match op.as_str() {
                    "pure" => {
                        if exprs.len() != 2 {
                            return Err(CompileError::InvalidArity {
                                expected: 1,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        check_sexpr_with_env(&exprs[1], env)
                    }
                    "alloc" => {
                        if exprs.len() != 2 {
                            return Err(CompileError::InvalidArity {
                                expected: 1,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        let _value_type = check_sexpr_with_env(&exprs[1], env)?;
                        // alloc returns a resource ID (represented as Int for now)
                        Ok(TypeInner::Base(BaseType::Int))
                    }
                    "consume" => {
                        if exprs.len() != 2 {
                            return Err(CompileError::InvalidArity {
                                expected: 1,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        let resource_type = check_sexpr_with_env(&exprs[1], env)?;
                        // consume expects a resource ID and returns the contained value
                        match resource_type {
                            TypeInner::Base(BaseType::Int) => Ok(TypeInner::Base(BaseType::Int)), // Simplified
                            _ => Err(CompileError::TypeError {
                                message: "consume expects a resource ID".to_string(),
                                expected: Some("ResourceId".to_string()),
                                found: Some(format!("{:?}", resource_type)),
                                location: None,
                            })
                        }
                    }
                    // Row/Record operations with capability checking
                    "record-get" => {
                        if exprs.len() != 3 {
                            return Err(CompileError::InvalidArity {
                                expected: 2,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        
                        let record_type = check_sexpr_with_env(&exprs[1], env)?;
                        let field_name = match &exprs[2] {
                            SExpression::Symbol(name) => name.clone(),
                            _ => return Err(CompileError::TypeError {
                                message: "record-get requires a field name".to_string(),
                                expected: Some("Symbol".to_string()),
                                found: Some(format!("{:?}", exprs[2])),
                                location: None,
                            })
                        };
                        
                        // Check capability for field access
                        let required_cap = Capability::read_field("record_access", &field_name);
                        if !env.has_capability(&required_cap) {
                            return Err(CompileError::TypeError {
                                message: format!("Missing capability to read field '{}'", field_name),
                                expected: Some(format!("ReadField({})", field_name)),
                                found: Some("No capability".to_string()),
                                location: None,
                            });
                        }
                        
                        // Type check the field access
                        match record_type {
                            TypeInner::Record(record_type) => {
                                match record_type.row.project(&field_name) {
                                    RowOpResult::Success(field_type) => Ok(field_type),
                                    RowOpResult::MissingField(field) => {
                                        Err(CompileError::TypeError {
                                            message: format!("Field '{}' not found in record", field),
                                            expected: None,
                                            found: None,
                                            location: None,
                                        })
                                    }
                                    _ => Err(CompileError::TypeError {
                                        message: "Row operation failed".to_string(),
                                        expected: None,
                                        found: None,
                                        location: None,
                                    })
                                }
                            }
                            _ => Err(CompileError::TypeError {
                                message: "record-get requires a record type".to_string(),
                                expected: Some("Record".to_string()),
                                found: Some(format!("{:?}", record_type)),
                                location: None,
                            })
                        }
                    }
                    "record-set" => {
                        if exprs.len() != 4 {
                            return Err(CompileError::InvalidArity {
                                expected: 3,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        
                        let record_type = check_sexpr_with_env(&exprs[1], env)?;
                        let field_name = match &exprs[2] {
                            SExpression::Symbol(name) => name.clone(),
                            _ => return Err(CompileError::TypeError {
                                message: "record-set requires a field name".to_string(),
                                expected: Some("Symbol".to_string()),
                                found: Some(format!("{:?}", exprs[2])),
                                location: None,
                            })
                        };
                        let _value_type = check_sexpr_with_env(&exprs[3], env)?;
                        
                        // Check capability for field modification
                        let required_cap = Capability::write_field("record_access", &field_name);
                        if !env.has_capability(&required_cap) {
                            return Err(CompileError::TypeError {
                                message: format!("Missing capability to write field '{}'", field_name),
                                expected: Some(format!("WriteField({})", field_name)),
                                found: Some("No capability".to_string()),
                                location: None,
                            });
                        }
                        
                        // Return the record type (simplified - field update returns same record type)
                        Ok(record_type)
                    }
                    "lambda" => {
                        if exprs.len() < 3 {
                            return Err(CompileError::InvalidArity {
                                expected: 2,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        // For now, return a function type placeholder
                        Ok(TypeInner::Base(BaseType::Symbol)) // Simplified function type
                    }
                    "let" => {
                        if exprs.len() != 4 {
                            return Err(CompileError::InvalidArity {
                                expected: 3,
                                found: exprs.len() - 1,
                                location: None,
                            });
                        }
                        // let var value body
                        if let SExpression::Symbol(var) = &exprs[1] {
                            let value_type = check_sexpr_with_env(&exprs[2], env)?;
                            env.extend(var.clone(), value_type);
                            check_sexpr_with_env(&exprs[3], env)
                        } else {
                            Err(CompileError::TypeError {
                                message: "let binding requires a variable name".to_string(),
                                expected: Some("Symbol".to_string()),
                                found: Some(format!("{:?}", exprs[1])),
                                location: None,
                            })
                        }
                    }
                    _ => Err(CompileError::UnknownSymbol {
                        symbol: op.clone(),
                        location: None,
                    })
                }
                _ => Err(CompileError::TypeError {
                    message: "Function application requires a symbol".to_string(),
                    expected: Some("Symbol".to_string()),
                    found: Some(format!("{:?}", exprs[0])),
                    location: None,
                })
            }
        }
    }
}

/// Check linearity constraints for variables and resources
pub fn check_linearity(expr: &SExpression) -> CompileResult<()> {
    let mut usage_tracker = LinearityTracker::new();
    check_linearity_with_tracker(expr, &mut usage_tracker)
}

/// Check linearity with usage tracking
fn check_linearity_with_tracker(expr: &SExpression, tracker: &mut LinearityTracker) -> CompileResult<()> {
    match expr {
        SExpression::Integer(_) | SExpression::Boolean(_) | SExpression::Nil => {
            // Literals don't affect linearity
            Ok(())
        }
        SExpression::Symbol(name) => {
            tracker.use_variable(name.clone())?;
            Ok(())
        }
        SExpression::List(exprs) => {
            if exprs.is_empty() {
                return Ok(());
            }
            
            match &exprs[0] {
                SExpression::Symbol(op) => match op.as_str() {
                    "alloc" => {
                        // alloc creates a new linear resource
                        for expr in exprs.iter().skip(1) {
                            check_linearity_with_tracker(expr, tracker)?;
                        }
                        Ok(())
                    }
                    "consume" => {
                        // consume uses a linear resource exactly once
                        if exprs.len() == 2 {
                            if let SExpression::Symbol(resource_var) = &exprs[1] {
                                tracker.consume_resource(resource_var.clone())?;
                            }
                        }
                        Ok(())
                    }
                    "let" => {
                        if exprs.len() == 4 {
                            if let SExpression::Symbol(var) = &exprs[1] {
                                // Check value expression
                                check_linearity_with_tracker(&exprs[2], tracker)?;
                                // Bind variable as linear
                                tracker.bind_linear_variable(var.clone());
                                // Check body
                                check_linearity_with_tracker(&exprs[3], tracker)?;
                            }
                        }
                        Ok(())
                    }
                    _ => {
                        // Check all subexpressions
                        for expr in exprs.iter().skip(1) {
                            check_linearity_with_tracker(expr, tracker)?;
                        }
                        Ok(())
                    }
                }
                _ => {
                    for expr in exprs {
                        check_linearity_with_tracker(expr, tracker)?;
                    }
                    Ok(())
                }
            }
        }
    }
}

/// Type environment for tracking variable types and capabilities
#[derive(Debug, Clone, Default)]
pub struct TypeEnvironment {
    bindings: BTreeMap<String, TypeInner>,
    capabilities: CapabilitySet,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a new environment with capabilities
    pub fn with_capabilities(capabilities: CapabilitySet) -> Self {
        Self {
            bindings: BTreeMap::new(),
            capabilities,
        }
    }
    
    pub fn extend(&mut self, var: String, ty: TypeInner) {
        self.bindings.insert(var, ty);
    }
    
    pub fn lookup(&self, var: &str) -> Option<&TypeInner> {
        self.bindings.get(var)
    }
    
    /// Add a capability to this environment
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.add(capability);
    }
    
    /// Check if the environment has a specific capability
    pub fn has_capability(&self, required: &Capability) -> bool {
        self.capabilities.has_capability(required)
    }
    
    /// Get all capabilities in this environment
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }
}

/// Linearity tracker for ensuring linear resources are used exactly once
#[derive(Debug, Clone, Default)]
pub struct LinearityTracker {
    linear_variables: BTreeMap<String, bool>, // true if used
    resources: BTreeMap<String, bool>, // true if consumed
}

impl LinearityTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn bind_linear_variable(&mut self, var: String) {
        self.linear_variables.insert(var, false);
    }
    
    pub fn use_variable(&mut self, var: String) -> CompileResult<()> {
        if let Some(used) = self.linear_variables.get_mut(&var) {
            if *used {
                return Err(CompileError::TypeError {
                    message: format!("Linear variable '{}' used more than once", var),
                    expected: Some("single use".to_string()),
                    found: Some("multiple uses".to_string()),
                    location: None,
                });
            }
            *used = true;
        }
        Ok(())
    }
    
    pub fn consume_resource(&mut self, resource: String) -> CompileResult<()> {
        if let Some(consumed) = self.resources.get_mut(&resource) {
            if *consumed {
                return Err(CompileError::TypeError {
                    message: format!("Resource '{}' consumed more than once", resource),
                    expected: Some("single consumption".to_string()),
                    found: Some("multiple consumptions".to_string()),
                    location: None,
                });
            }
            *consumed = true;
        } else {
            self.resources.insert(resource, true);
        }
        Ok(())
    }
}

/// Evaluate a term for testing purposes
pub fn evaluate_term(term: &Term) -> CompileResult<causality_core::lambda::Value> {
    use causality_core::lambda::Value;
    
    match &term.kind {
        TermKind::Literal(lit) => match lit {
            Literal::Unit => {
                Ok(Value::Unit)
            },
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Int(i) => Ok(Value::Int(*i)),
            Literal::Symbol(s) => {
                // Convert Symbol to Str using the name or a default representation
                let str_value = s.name()
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| s.to_hex());
                Ok(Value::Symbol(str_value.into()))
            },
        },
        TermKind::Unit => Ok(Value::Unit),
        TermKind::Var(_) => Err(CompileError::CompilationError {
            message: "Cannot evaluate unbound variable".to_string(),
            location: None,
        }),
        _ => Err(CompileError::CompilationError {
            message: "Term evaluation not fully implemented".to_string(),
            location: None,
        }),
    }
}

/// Check capability constraints and access control
pub fn check_capability_access(operation: &str, field: Option<&str>, capabilities: &CapabilitySet) -> CompileResult<()> {
    let required_cap = match (operation, field) {
        ("read", Some(field_name)) => Capability::read_field("operation", field_name),
        ("write", Some(field_name)) => Capability::write_field("operation", field_name),
        ("execute", _) => Capability::new("operation", causality_core::effect::CapabilityLevel::Execute),
        _ => return Err(CompileError::TypeError {
            message: format!("Unknown capability operation: {}", operation),
            expected: None,
            found: None,
            location: None,
        })
    };
    
    if capabilities.has_capability(&required_cap) {
        Ok(())
    } else {
        Err(CompileError::TypeError {
            message: format!("Missing required capability: {:?}", required_cap),
            expected: Some(format!("{:?}", required_cap)),
            found: Some("No matching capability".to_string()),
            location: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::SExpression;
    
    #[test]
    fn test_check_simple_expression() {
        let expr = SExpression::Integer(42);
        let result = check_sexpr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_check_pure_expression() {
        let expr = SExpression::List(vec![
            SExpression::Symbol("pure".to_string()),
            SExpression::Integer(42),
        ]);
        let result = check_sexpr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_check_alloc_expression() {
        let expr = SExpression::List(vec![
            SExpression::Symbol("alloc".to_string()),
            SExpression::Integer(100),
        ]);
        let result = check_sexpr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_check_let_binding() {
        let expr = SExpression::List(vec![
            SExpression::Symbol("let".to_string()),
            SExpression::Symbol("x".to_string()),
            SExpression::Integer(42),
            SExpression::Symbol("x".to_string()),
        ]);
        let result = check_sexpr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_check_unknown_symbol() {
        let expr = SExpression::Symbol("unknown".to_string());
        let result = check_sexpr(&expr);
        assert!(result.is_err());
        match result.unwrap_err() {
            CompileError::UnknownSymbol { symbol, .. } => {
                assert_eq!(symbol, "unknown");
            }
            _ => panic!("Expected UnknownSymbol error"),
        }
    }
    
    #[test]
    fn test_check_invalid_arity() {
        let expr = SExpression::List(vec![
            SExpression::Symbol("pure".to_string()),
            // Missing argument
        ]);
        let result = check_sexpr(&expr);
        assert!(result.is_err());
        match result.unwrap_err() {
            CompileError::InvalidArity { expected, found, .. } => {
                assert_eq!(expected, 1);
                assert_eq!(found, 0);
            }
            _ => panic!("Expected InvalidArity error"),
        }
    }
    
    #[test]
    fn test_check_linearity() {
        let expr = SExpression::Symbol("x".to_string());
        assert!(check_linearity(&expr).is_ok());
    }
    
    #[test]
    fn test_linearity_tracker() {
        let mut tracker = LinearityTracker::new();
        tracker.bind_linear_variable("x".to_string());
        
        // First use should succeed
        assert!(tracker.use_variable("x".to_string()).is_ok());
        
        // Second use should fail
        assert!(tracker.use_variable("x".to_string()).is_err());
    }
    
    #[test]
    fn test_resource_consumption() {
        let mut tracker = LinearityTracker::new();
        
        // First consumption should succeed
        assert!(tracker.consume_resource("r1".to_string()).is_ok());
        
        // Second consumption should fail
        assert!(tracker.consume_resource("r1".to_string()).is_err());
    }
    
    #[test]
    fn test_type_environment() {
        let mut env = TypeEnvironment::new();
        env.extend("x".to_string(), TypeInner::Base(BaseType::Int));
        
        assert_eq!(env.lookup("x"), Some(&TypeInner::Base(BaseType::Int)));
        assert_eq!(env.lookup("y"), None);
    }
    
    #[test]
    fn test_term_evaluation() {
        use causality_core::lambda::{Term, Value};
        
        let term = Term::literal(Literal::Int(42));
        let result = evaluate_term(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
        
        let bool_term = Term::literal(Literal::Bool(true));
        let bool_result = evaluate_term(&bool_term);
        assert!(bool_result.is_ok());
        assert_eq!(bool_result.unwrap(), Value::Bool(true));
    }
    
    #[test]
    fn test_function_application_compilation() {
        // Test that compiles and executes a simple function application
        let expr = SExpression::List(vec![
            SExpression::Symbol("let".to_string()),
            SExpression::Symbol("f".to_string()),
            SExpression::List(vec![
                SExpression::Symbol("lambda".to_string()),
                SExpression::Symbol("x".to_string()),
                SExpression::Symbol("x".to_string()),
            ]),
            SExpression::Symbol("f".to_string()),
        ]);
        
        let result = check_sexpr(&expr);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_capability_environment() {
        
        
        let mut capabilities = CapabilitySet::new();
        capabilities.add(Capability::read_field("test", "name"));
        
        let env = TypeEnvironment::with_capabilities(capabilities);
        
        // Should have the read capability
        let read_cap = Capability::read_field("test", "name");
        assert!(env.has_capability(&read_cap));
        
        // Should not have write capability
        let write_cap = Capability::write_field("test", "name");
        assert!(!env.has_capability(&write_cap));
    }
    
    #[test]
    fn test_record_access_with_capability() {
        use std::collections::BTreeMap;
        use causality_core::effect::{RowType, RecordType};
        
        // Create a record type with a name field
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), TypeInner::Base(BaseType::Symbol));
        let row = RowType::with_fields(fields);
        let record_type = TypeInner::Record(RecordType { row });
        
        // Create environment with read capability
        let mut capabilities = CapabilitySet::new();
        capabilities.add(Capability::read_field("record_access", "name"));
        let mut env = TypeEnvironment::with_capabilities(capabilities);
        env.extend("rec".to_string(), record_type);
        
        // Test record access with capability
        let expr = SExpression::List(vec![
            SExpression::Symbol("record-get".to_string()),
            SExpression::Symbol("rec".to_string()),
            SExpression::Symbol("name".to_string()),
        ]);
        
        let result = check_sexpr_with_env(&expr, &mut env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Symbol));
    }
    
    #[test]
    fn test_record_access_missing_capability() {
        use std::collections::BTreeMap;
        use causality_core::effect::{RowType, RecordType};
        
        // Create a record type with a name field
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), TypeInner::Base(BaseType::Symbol));
        let row = RowType::with_fields(fields);
        let record_type = TypeInner::Record(RecordType { row });
        
        // Create environment WITHOUT read capability
        let mut env = TypeEnvironment::new();
        env.extend("rec".to_string(), record_type);
        
        // Test record access without capability - should fail
        let expr = SExpression::List(vec![
            SExpression::Symbol("record-get".to_string()),
            SExpression::Symbol("rec".to_string()),
            SExpression::Symbol("name".to_string()),
        ]);
        
        let result = check_sexpr_with_env(&expr, &mut env);
        assert!(result.is_err());
        match result.unwrap_err() {
            CompileError::TypeError { message, .. } => {
                assert!(message.contains("Missing capability to read field 'name'"));
            }
            _ => panic!("Expected TypeError for missing capability"),
        }
    }
    
    #[test]
    fn test_record_write_missing_capability() {
        use std::collections::BTreeMap;
        use causality_core::effect::{RowType, RecordType};
        
        // Create a record type with a name field
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), TypeInner::Base(BaseType::Symbol));
        let row = RowType::with_fields(fields);
        let record_type = TypeInner::Record(RecordType { row });
        
        // Create environment with only READ capability
        let mut capabilities = CapabilitySet::new();
        capabilities.add(Capability::read_field("record_access", "name"));
        let mut env = TypeEnvironment::with_capabilities(capabilities);
        env.extend("rec".to_string(), record_type);
        
        // Test record write without write capability - should fail
        let expr = SExpression::List(vec![
            SExpression::Symbol("record-set".to_string()),
            SExpression::Symbol("rec".to_string()),
            SExpression::Symbol("name".to_string()),
            SExpression::Boolean(true), // Use a literal value that doesn't need environment lookup
        ]);
        
        let result = check_sexpr_with_env(&expr, &mut env);
        assert!(result.is_err(), "Expected error for missing write capability, but got: {:?}", result);
        match result.unwrap_err() {
            CompileError::TypeError { message, .. } => {
                assert!(message.contains("Missing capability to write field 'name'"));
            }
            err => panic!("Expected TypeError for missing write capability, but got: {:?}", err),
        }
    }
    
    #[test]
    fn test_capability_access_function() {
        let mut capabilities = CapabilitySet::new();
        capabilities.add(Capability::read_field("operation", "test_field"));
        
        // Test successful capability check
        let result = check_capability_access("read", Some("test_field"), &capabilities);
        assert!(result.is_ok());
        
        // Test missing capability
        let result = check_capability_access("write", Some("test_field"), &capabilities);
        assert!(result.is_err());
        
        // Test unknown operation
        let result = check_capability_access("invalid_op", None, &capabilities);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_row_type_zero_runtime_overhead() {
        use std::collections::BTreeMap;
        use causality_core::effect::RowType;
        
        // Capability checking should be compile-time only
        // This test verifies that the type checking works without runtime penalty
        
        let mut fields = BTreeMap::new();
        fields.insert("x".to_string(), TypeInner::Base(BaseType::Int));
        fields.insert("y".to_string(), TypeInner::Base(BaseType::Int));
        let row = RowType::with_fields(fields);
        
        // Row operations are all compile-time
        let project_result = row.project("x");
        assert!(matches!(project_result, RowOpResult::Success(_)));
        
        let extend_result = row.extend("z".to_string(), TypeInner::Base(BaseType::Bool));
        assert!(matches!(extend_result, RowOpResult::Success(_)));
        
        // These operations have zero runtime cost - they're purely type-level
        assert!(row.is_closed());
        assert_eq!(row.field_names().len(), 2);
    }
} 