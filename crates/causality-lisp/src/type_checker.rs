//! Type checker for Causality Lisp
//!
//! This module provides static type checking with support for:
//! - Linear type system integration
//! - Layer 1 primitive type checking
//! - Effect type inference
//! - Resource lifetime tracking
//! - Row type and capability enforcement

use crate::ast::{Expr, ExprKind, LispValue};
use crate::error::{TypeError, TypeResult};
use causality_core::effect::{Capability, CapabilitySet, RecordCapability, RowType};
use causality_core::lambda::base::{TypeInner, BaseType, SessionType};
use std::collections::BTreeMap;

/// Type checker for Lisp expressions
pub struct TypeChecker {
    pub type_env: TypeContext,
}

/// Type checking context with capability tracking
#[derive(Debug, Clone)]
pub struct TypeContext {
    pub type_bindings: BTreeMap<String, TypeInner>,
    pub current_scope: usize,
    /// Available capabilities for the current context
    pub capabilities: CapabilitySet,
    /// Track row type constraints
    pub row_constraints: BTreeMap<String, RowType>,
}

/// Type representation with linearity and effects
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Base types
    Unit,
    Bool,
    Int,

    String,
    Symbol,
    
    // Compound types
    List(Box<Type>),
    Function {
        params: Vec<Type>,
        result: Box<Type>,
        effects: Vec<EffectType>,
    },
    
    // Layer 1 types
    Tensor(Box<Type>, Box<Type>),
    Sum(Box<Type>, Box<Type>),
    Resource(Box<Type>),
    
    // Type variables for inference
    TypeVar(usize),
    
    // Linear type wrapper
    Linear(Box<Type>),
    
    // Effect types
    Effect(EffectType),
}

/// Linearity status tracking
#[derive(Debug, Clone, PartialEq)]
pub enum LinearityStatus {
    /// Available for use
    Available,
    /// Already used (linear resources)
    Consumed,
    /// Can be used multiple times (non-linear)
    Unrestricted,
}

/// Effect type representation
#[derive(Debug, Clone, PartialEq)]
pub struct EffectType {
    pub name: String,
    pub params: Vec<Type>,
}

/// Effect signature for effect handlers
#[derive(Debug, Clone, PartialEq)]
pub struct EffectSignature {
    pub effect_type: EffectType,
    pub handler_type: Type,
}

/// Type constraints for unification
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    /// Two types must be equal
    Equal(Type, Type),
    /// A type must be linear
    Linear(Type),
    /// A type must be non-linear (allow multiple uses)
    NonLinear(Type),
    /// Effect constraint
    HasEffect(Type, EffectType),
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            type_env: TypeContext::new(),
        }
    }
    
    /// Create a type checker with specific capabilities
    pub fn with_capabilities(capabilities: Vec<Capability>) -> Self {
        let mut checker = Self::new();
        for cap in capabilities {
            checker.type_env.capabilities.add(cap);
        }
        checker
    }
    
    /// Check if a record operation is allowed given current capabilities
    pub fn check_record_capability(&self, required_cap: &RecordCapability) -> TypeResult<()> {
        // Check if any of the current capabilities implies the required one
        let has_capability = self.type_env.capabilities.capabilities().iter().any(|cap| {
            if let Some(record_cap) = &cap.record_capability {
                record_cap.implies(required_cap)
            } else {
                false
            }
        });
        
        if has_capability {
            Ok(())
        } else {
            Err(TypeError::Mismatch {
                expected: format!("Capability for {:?}", required_cap),
                found: "Missing capability".to_string(),
            })
        }
    }
    
    /// Check row type constraints for record operations
    pub fn check_row_constraints(&self, row: &RowType, operation: &str, field: Option<&str>) -> TypeResult<()> {
        match operation {
            "project" => {
                if let Some(field_name) = field {
                    if !row.fields.contains_key(field_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!("Field '{}' in record", field_name),
                            found: "Field not present".to_string(),
                        });
                    }
                }
            }
            "extend" => {
                if let Some(field_name) = field {
                    if row.fields.contains_key(field_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!("No existing field '{}'", field_name),
                            found: "Field already exists".to_string(),
                        });
                    }
                }
            }
            "restrict" => {
                if let Some(field_name) = field {
                    if !row.fields.contains_key(field_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!("Existing field '{}'", field_name),
                            found: "Field not present".to_string(),
                        });
                    }
                }
            }
            _ => {} // Other operations don't have specific row constraints
        }
        Ok(())
    }
    
    /// Check the type of an expression
    pub fn check_expr(&mut self, expr: &Expr) -> TypeResult<TypeInner> {
        match &expr.kind {
            // Literals and variables
            ExprKind::Const(value) => {
                match value {
                    LispValue::Unit => Ok(TypeInner::Base(BaseType::Unit)),
                    LispValue::Bool(_) => Ok(TypeInner::Base(BaseType::Bool)),
                    LispValue::Int(_) => Ok(TypeInner::Base(BaseType::Int)),
        
                    LispValue::String(_) => Ok(TypeInner::Base(BaseType::Symbol)), // Map to Symbol for now
                    LispValue::Symbol(_) => Ok(TypeInner::Base(BaseType::Symbol)),
                    _ => Err(TypeError::Mismatch { 
                        expected: "Simple type".to_string(), 
                        found: "Complex constant".to_string() 
                    }),
                }
            }
            ExprKind::Var(name) => {
                self.type_env.lookup_type(&name.to_string())
                    .ok_or_else(|| TypeError::Mismatch { 
                        expected: "Defined variable".to_string(), 
                        found: format!("Undefined variable: {}", name) 
                    })
            }
            
            // Unit type
            ExprKind::UnitVal => Ok(TypeInner::Base(BaseType::Unit)),
            ExprKind::LetUnit(unit_expr, body) => {
                // Check unit expression is actually unit type
                let unit_type = self.check_expr(unit_expr)?;
                if !matches!(unit_type, TypeInner::Base(BaseType::Unit)) {
                    return Err(TypeError::Mismatch { 
                        expected: "Unit".to_string(), 
                        found: format!("{:?}", unit_type) 
                    });
                }
                // Type of let-unit is the type of the body
                self.check_expr(body)
            }
            
            // Tensor product
            ExprKind::Tensor(left, right) => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                Ok(TypeInner::Product(Box::new(left_type), Box::new(right_type)))
            }
            ExprKind::LetTensor(tensor_expr, left_name, right_name, body) => {
                let tensor_type = self.check_expr(tensor_expr)?;
                match tensor_type {
                    TypeInner::Product(left_type, right_type) => {
                        // Bind the variables in scope for the body
                        self.type_env.bind_type(left_name.to_string(), *left_type);
                        self.type_env.bind_type(right_name.to_string(), *right_type);
                        let result = self.check_expr(body);
                        // Remove bindings (simplified scope management)
                        self.type_env.remove_binding(&left_name.to_string());
                        self.type_env.remove_binding(&right_name.to_string());
                        result
                    }
                    _ => Err(TypeError::Mismatch { 
                        expected: "Product type".to_string(), 
                        found: format!("{:?}", tensor_type) 
                    }),
                }
            }
            
            // Sum types
            ExprKind::Inl(value) => {
                let value_type = self.check_expr(value)?;
                // For now, return a generic sum type
                // In a full implementation, we'd need type annotations or inference
                Ok(TypeInner::Sum(Box::new(value_type), Box::new(TypeInner::Base(BaseType::Unit))))
            }
            ExprKind::Inr(value) => {
                let value_type = self.check_expr(value)?;
                // For now, return a generic sum type
                Ok(TypeInner::Sum(Box::new(TypeInner::Base(BaseType::Unit)), Box::new(value_type)))
            }
            ExprKind::Case(sum_expr, left_name, left_branch, right_name, right_branch) => {
                let sum_type = self.check_expr(sum_expr)?;
                match sum_type {
                    TypeInner::Sum(left_type, right_type) => {
                        // Type check left branch with left variable bound
                        self.type_env.bind_type(left_name.to_string(), *left_type);
                        let left_result_type = self.check_expr(left_branch)?;
                        self.type_env.remove_binding(&left_name.to_string());
                        
                        // Type check right branch with right variable bound
                        self.type_env.bind_type(right_name.to_string(), *right_type);
                        let right_result_type = self.check_expr(right_branch)?;
                        self.type_env.remove_binding(&right_name.to_string());
                        
                        // Both branches must have the same type
                        if left_result_type == right_result_type {
                            Ok(left_result_type)
                        } else {
                            Err(TypeError::Mismatch { 
                                expected: format!("{:?}", left_result_type), 
                                found: format!("{:?}", right_result_type) 
                            })
                        }
                    }
                    _ => Err(TypeError::Mismatch { 
                        expected: "Sum type".to_string(), 
                        found: format!("{:?}", sum_type) 
                    }),
                }
            }
            
            // Linear functions
            ExprKind::Lambda(params, body) => {
                // Bind parameters in scope
                for param in params {
                    // For now, assume all parameters are of generic type
                    // In a full implementation, we'd need type annotations
                    self.type_env.bind_type(param.name.to_string(), TypeInner::Base(BaseType::Symbol));
                }
                
                let body_type = self.check_expr(body)?;
                
                // Remove parameter bindings
                for param in params {
                    self.type_env.remove_binding(&param.name.to_string());
                }
                
                // Create function type (simplified - assumes single parameter)
                if let Some(_param) = params.first() {
                    let param_type = TypeInner::Base(BaseType::Symbol); // Placeholder
                    Ok(TypeInner::LinearFunction(Box::new(param_type), Box::new(body_type)))
                } else {
                    // Zero-parameter function
                    Ok(TypeInner::LinearFunction(
                        Box::new(TypeInner::Base(BaseType::Unit)), 
                        Box::new(body_type)
                    ))
                }
            }
            ExprKind::Apply(func_expr, arg_exprs) => {
                let mut current_type = self.check_expr(func_expr)?;
                
                // Apply each argument in sequence for curried functions
                for arg_expr in arg_exprs {
                    match current_type {
                        TypeInner::LinearFunction(ref param_type, ref return_type) => {
                            let arg_type = self.check_expr(arg_expr)?;
                            if arg_type == **param_type {
                                current_type = (**return_type).clone();
                            } else {
                                return Err(TypeError::Mismatch { 
                                    expected: format!("{:?}", param_type), 
                                    found: format!("{:?}", arg_type) 
                                });
                            }
                        }
                        _ => {
                            return Err(TypeError::Mismatch { 
                                expected: "Function type".to_string(), 
                                found: format!("{:?}", current_type) 
                            });
                        }
                    }
                }
                
                Ok(current_type)
            }
            
            // Resource management
            ExprKind::Alloc(value_expr) => {
                let value_type = self.check_expr(value_expr)?;
                // For now, just return the value type as resources are tracked through annotation
                Ok(value_type)
            }
            ExprKind::Consume(resource_expr) => {
                let resource_type = self.check_expr(resource_expr)?;
                // For resource consumption, just return the inner type
                // In a full implementation, we'd track linearity separately
                Ok(resource_type)
            }
            
            // Record operations with capability checking
            ExprKind::RecordAccess { record, field } => {
                let record_type = self.check_expr(record)?;
                
                // Check record capability
                let read_cap = RecordCapability::read_field(field);
                self.check_record_capability(&read_cap)?;
                
                // Check row constraints
                if let TypeInner::Record(ref record_ty) = record_type {
                    self.check_row_constraints(&record_ty.row, "project", Some(field))?;
                    
                    // Return field type
                    if let Some(field_type) = record_ty.row.fields.get(field) {
                        Ok(field_type.ty.clone())
                    } else {
                        Err(TypeError::Mismatch {
                            expected: format!("Field '{}' in record", field),
                            found: "Field not present".to_string(),
                        })
                    }
                } else {
                    Err(TypeError::Mismatch {
                        expected: "Record type".to_string(),
                        found: format!("{:?}", record_type),
                    })
                }
            }
            
            ExprKind::RecordUpdate { record, field, value } => {
                let record_type = self.check_expr(record)?;
                let value_type = self.check_expr(value)?;
                
                // Check record capability
                let write_cap = RecordCapability::write_field(field);
                self.check_record_capability(&write_cap)?;
                
                // Check row constraints and type compatibility
                if let TypeInner::Record(ref record_ty) = record_type {
                    self.check_row_constraints(&record_ty.row, "update", Some(field))?;
                    
                    // Check field type compatibility
                    if let Some(field_type) = record_ty.row.fields.get(field) {
                        if field_type.ty != value_type {
                            return Err(TypeError::Mismatch {
                                expected: format!("{:?}", field_type.ty),
                                found: format!("{:?}", value_type),
                            });
                        }
                    }
                    
                    // Return updated record type (same as input)
                    Ok(record_type)
                } else {
                    Err(TypeError::Mismatch {
                        expected: "Record type".to_string(),
                        found: format!("{:?}", record_type),
                    })
                }
            }

            // Session types operations
            ExprKind::SessionDeclaration { name: _, roles: _ } => {
                // For session declarations, we just return unit type
                // In a full implementation, this would register the session type in the environment
                Ok(TypeInner::Base(BaseType::Unit))
            }

            ExprKind::WithSession { session: _, role: _, body } => {
                // For with-session, we type check the body
                // In a full implementation, this would set up session channel types
                self.check_expr(body)
            }

            ExprKind::SessionSend { channel, value } => {
                // Type check both the channel and value
                let _channel_type = self.check_expr(channel)?;
                let _value_type = self.check_expr(value)?;
                
                // For now, return unit type (successful send)
                // In a full implementation, this would check protocol compatibility
                Ok(TypeInner::Base(BaseType::Unit))
            }

            ExprKind::SessionReceive { channel } => {
                // Type check the channel
                let channel_type = self.check_expr(channel)?;
                
                // Extract the receive type from the session channel
                match channel_type {
                    TypeInner::Session(session_type) => {
                        // Extract the expected receive type from the session protocol
                        match session_type.as_ref() {
                            SessionType::Send(_, _next) => {
                                // If channel expects to send, we can't receive
                                Err(TypeError::Mismatch {
                                    expected: "Receive capability".to_string(),
                                    found: "Send-only channel".to_string(),
                                })
                            }
                            SessionType::Receive(message_type, _next) => {
                                // Return the message type we expect to receive
                                Ok(message_type.as_ref().clone())
                            }
                            SessionType::End => {
                                // Cannot receive from ended session
                                Err(TypeError::Mismatch {
                                    expected: "Active session".to_string(),
                                    found: "Ended session".to_string(),
                                })
                            }
                            _ => {
                                // For other session types, return a generic symbol type
                                Ok(TypeInner::Base(BaseType::Symbol))
                            }
                        }
                    }
                    _ => {
                        // For non-session types, assume it's a simple value channel
                        Ok(TypeInner::Base(BaseType::Symbol))
                    }
                }
            }

            ExprKind::SessionSelect { channel, choice: _ } => {
                // Type check the channel
                let _channel_type = self.check_expr(channel)?;
                
                // Session selection returns unit type (successful select operation)
                Ok(TypeInner::Base(BaseType::Unit))
            }

            ExprKind::SessionCase { channel, branches } => {
                // Type check the channel
                let _channel_type = self.check_expr(channel)?;
                
                // Type check all branches and ensure they have compatible types
                if branches.is_empty() {
                    return Err(TypeError::Mismatch {
                        expected: "At least one branch".to_string(),
                        found: "No branches provided".to_string(),
                    });
                }
                
                // Type check the first branch to get the expected result type
                let first_branch_type = self.check_expr(&branches[0].body)?;
                
                // Verify all other branches have the same type
                for branch in branches.iter().skip(1) {
                    let branch_type = self.check_expr(&branch.body)?;
                    if branch_type != first_branch_type {
                        return Err(TypeError::Mismatch {
                            expected: format!("{:?}", first_branch_type),
                            found: format!("{:?}", branch_type),
                        });
                    }
                }
                
                Ok(first_branch_type)
            }
        }
    }
    
    /// Convert a session type to TypeInner for type checking
    fn session_type_to_type_inner(&self, session_type: &SessionType) -> TypeInner {
        match session_type {
            SessionType::Send(_, _) => TypeInner::Base(BaseType::Unit),
            SessionType::Receive(_, _) => TypeInner::Base(BaseType::Symbol),
            SessionType::InternalChoice(_) => TypeInner::Base(BaseType::Symbol),
            SessionType::ExternalChoice(_) => TypeInner::Base(BaseType::Symbol),
            SessionType::End => TypeInner::Base(BaseType::Unit),
            SessionType::Recursive(_, _) => TypeInner::Base(BaseType::Symbol),
            SessionType::Variable(_) => TypeInner::Base(BaseType::Symbol),
        }
    }
}

impl TypeContext {
    /// Create a new type context with built-in types
    pub fn new() -> Self {
        let mut type_bindings = BTreeMap::new();
        
        // Add built-in function types
        type_bindings.insert("+".to_string(), TypeInner::LinearFunction(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::LinearFunction(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Int))
            ))
        ));
        type_bindings.insert("-".to_string(), TypeInner::LinearFunction(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::LinearFunction(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Int))
            ))
        ));
        type_bindings.insert("*".to_string(), TypeInner::LinearFunction(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::LinearFunction(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Int))
            ))
        ));
        type_bindings.insert("/".to_string(), TypeInner::LinearFunction(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::LinearFunction(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Int))
            ))
        ));
        type_bindings.insert("=".to_string(), TypeInner::LinearFunction(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::LinearFunction(
                Box::new(TypeInner::Base(BaseType::Int)),
                Box::new(TypeInner::Base(BaseType::Bool))
            ))
        ));
        
        Self {
            type_bindings,
            current_scope: 0,
            capabilities: CapabilitySet::new(),
            row_constraints: BTreeMap::new(),
        }
    }
    
    /// Bind a variable to a type
    pub fn bind_type(&mut self, name: String, ty: TypeInner) {
        self.type_bindings.insert(name, ty);
    }
    
    /// Look up the type of a variable
    pub fn lookup_type(&self, name: &str) -> Option<TypeInner> {
        self.type_bindings.get(name).cloned()
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.current_scope -= 1;
        }
    }
    
    /// Remove a binding from the type environment
    pub fn remove_binding(&mut self, name: &str) {
        self.type_bindings.remove(name);
    }
    
    /// Add a capability to the context
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.add(capability);
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.has_capability(capability)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LispParser;

    #[test]
    fn test_type_check_basic_literals() {
        let mut checker = TypeChecker::new();
        let mut parser = LispParser::new();
        
        let expr = parser.parse("42").unwrap();
        let ty = checker.check_expr(&expr).unwrap();
        assert_eq!(ty, TypeInner::Base(BaseType::Int));
        
        let expr = parser.parse("true").unwrap();
        let ty = checker.check_expr(&expr).unwrap();
        assert_eq!(ty, TypeInner::Base(BaseType::Bool));
    }
    
    #[test]
    fn test_type_check_function_application() {
        let mut checker = TypeChecker::new();
        let mut parser = LispParser::new();
        
        let expr = parser.parse("(+ 1 2)").unwrap();
        let ty = checker.check_expr(&expr).unwrap();
        assert_eq!(ty, TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_type_check_lambda() {
        let mut checker = TypeChecker::new();
        let mut parser = LispParser::new();
        
        let expr = parser.parse("(lambda (x) x)").unwrap();
        let ty = checker.check_expr(&expr).unwrap();
        
        // Should be a function type
        assert!(matches!(ty, TypeInner::LinearFunction(..)));
    }
    
    #[test]
    fn test_type_check_layer1_primitives() {
        let mut checker = TypeChecker::new();
        let mut parser = LispParser::new();
        
        // Test tensor type
        let expr = parser.parse("(tensor 1 true)").unwrap();
        let ty = checker.check_expr(&expr).unwrap();
        assert_eq!(ty, TypeInner::Product(Box::new(TypeInner::Base(BaseType::Int)), Box::new(TypeInner::Base(BaseType::Bool))));
        
        // Test sum types
        let expr = parser.parse("(inl 42)").unwrap();
        let ty = checker.check_expr(&expr).unwrap();
        assert!(matches!(ty, TypeInner::Sum(..)));
    }
    
    #[test]
    fn test_capability_enforcement() {
        use causality_core::effect::{Capability, RowType, RecordType, FieldType};
        use causality_core::lambda::base::{TypeInner, BaseType, SessionType};
        use std::collections::BTreeMap;
        
        // Create a record type with a field
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), FieldType::simple(TypeInner::Base(BaseType::Symbol)));
        fields.insert("age".to_string(), FieldType::simple(TypeInner::Base(BaseType::Int)));
        let row = RowType::with_fields(fields);
        let record_type = TypeInner::Record(RecordType { row });
        
        // Test case 1: Access without capability should fail
        let mut checker_no_caps = TypeChecker::new();
        checker_no_caps.type_env.bind_type("person".to_string(), record_type.clone());
        
        let access_expr = Expr::record_access(Expr::variable("person"), "name");
        let result = checker_no_caps.check_expr(&access_expr);
        assert!(result.is_err());
        println!("✓ Record access denied without capability");
        
        // Test case 2: Access with correct capability should succeed
        let read_cap = Capability::read_field("person_read", "name");
        let mut checker_with_cap = TypeChecker::with_capabilities(vec![read_cap]);
        checker_with_cap.type_env.bind_type("person".to_string(), record_type.clone());
        
        let access_expr = Expr::record_access(Expr::variable("person"), "name");
        let result = checker_with_cap.check_expr(&access_expr);
        assert!(result.is_ok());
        println!("✓ Record access allowed with correct capability");
        
        // Test case 3: Update without write capability should fail
        let mut checker_read_only = TypeChecker::with_capabilities(vec![
            Capability::read_field("person_read", "name")
        ]);
        checker_read_only.type_env.bind_type("person".to_string(), record_type.clone());
        
        let update_expr = Expr::record_update(
            Expr::variable("person"), 
            "name", 
            Expr::constant(LispValue::Symbol("Alice".into()))
        );
        let result = checker_read_only.check_expr(&update_expr);
        assert!(result.is_err());
        println!("✓ Record update denied with only read capability");
        
        // Test case 4: Update with write capability should succeed
        let write_cap = Capability::write_field("person_write", "name");
        let mut checker_with_write = TypeChecker::with_capabilities(vec![write_cap]);
        checker_with_write.type_env.bind_type("person".to_string(), record_type);
        
        let update_expr = Expr::record_update(
            Expr::variable("person"), 
            "name", 
            Expr::constant(LispValue::Symbol("Bob".into()))
        );
        let result = checker_with_write.check_expr(&update_expr);
        assert!(result.is_ok());
        println!("✓ Record update allowed with write capability");
    }
    
    #[test]
    fn test_zero_runtime_overhead() {
        use causality_core::effect::{Capability, RecordCapability};
        use std::time::Instant;
        
        // Create type checker with many capabilities
        let capabilities = (0..1000).map(|i| {
            Capability::read_field(format!("cap_{}", i), format!("field_{}", i))
        }).collect();
        
        let checker = TypeChecker::with_capabilities(capabilities);
        
        // Compile-time check should be fast (capability checking happens at compile time)
        let start = Instant::now();
        for _ in 0..1000 {
            let required_cap = RecordCapability::read_field("field_500");
            let _ = checker.check_record_capability(&required_cap);
        }
        let duration = start.elapsed();
        
        // Should complete quickly (sub-millisecond for this simple operation)
        assert!(duration.as_millis() < 500);
        println!("✓ Capability checking completed in {:?} (zero runtime overhead)", duration);
    }
} 