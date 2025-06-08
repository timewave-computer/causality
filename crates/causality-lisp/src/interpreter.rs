//! Interpreter for Causality Lisp
//!
//! This module provides an interpreter that evaluates Causality Lisp expressions
//! and produces runtime values.

use crate::{
    ast::{Expr, ExprKind, LispValue},
    value::{Value, ValueKind, Environment},
    error::{EvalError},
};
use causality_core::{
    lambda::Symbol,
};
use std::collections::HashMap;

/// Evaluation context containing the current environment
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// Environment for variable bindings
    environment: Environment,
}

impl EvalContext {
    /// Create a new evaluation context
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }
    
    /// Create context from an environment
    pub fn from_environment(env: Environment) -> Self {
        Self {
            environment: env,
        }
    }
    
    /// Bind a value to a name
    pub fn bind(&mut self, name: Symbol, value: Value) {
        self.environment.bind(name, value);
    }
    
    /// Look up a value by name
    pub fn lookup(&self, name: &Symbol) -> Option<&Value> {
        self.environment.lookup(name)
    }
    
    /// Get a mutable reference to the environment
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }
    
    /// Get a reference to the environment
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}

/// Result type for evaluation operations
pub type EvalResult<T> = Result<T, EvalError>;

/// Main interpreter for Causality Lisp
pub struct Interpreter {
    /// Global environment
    global_env: Environment,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        let mut global_env = Environment::new();
        
        // Add built-in functions
        global_env.bind(Symbol::new("+"), Value::builtin("+", 2));
        global_env.bind(Symbol::new("-"), Value::builtin("-", 2));
        global_env.bind(Symbol::new("*"), Value::builtin("*", 2));
        global_env.bind(Symbol::new("/"), Value::builtin("/", 2));
        global_env.bind(Symbol::new("="), Value::builtin("=", 2));
        global_env.bind(Symbol::new("<"), Value::builtin("<", 2));
        global_env.bind(Symbol::new(">"), Value::builtin(">", 2));
        
        Self { global_env }
    }
    
    /// Evaluate an expression
    pub fn eval(&mut self, expr: &Expr) -> EvalResult<Value> {
        let mut context = EvalContext::from_environment(self.global_env.clone());
        self.eval_with_context(expr, &mut context)
    }
    
    /// Evaluate an expression with a given context
    pub fn eval_with_context(&mut self, expr: &Expr, context: &mut EvalContext) -> EvalResult<Value> {
        match &expr.kind {
            // Literals and variables
            ExprKind::Const(value) => self.eval_const(value),
            ExprKind::Var(name) => self.eval_var(name, context),
            
            // Unit type
            ExprKind::UnitVal => Ok(Value::unit()),
            ExprKind::LetUnit(unit_expr, body) => {
                // Evaluate unit expression (for side effects) then evaluate body
                let _unit_val = self.eval_with_context(unit_expr, context)?;
                self.eval_with_context(body, context)
            }
            
            // Tensor product
            ExprKind::Tensor(left, right) => {
                let left_val = self.eval_with_context(left, context)?;
                let right_val = self.eval_with_context(right, context)?;
                Ok(Value::tensor(left_val, right_val))
            }
            ExprKind::LetTensor(tensor_expr, left_name, right_name, body) => {
                let tensor_val = self.eval_with_context(tensor_expr, context)?;
                if let ValueKind::Tensor(left_val, right_val) = tensor_val.kind {
                    // Bind the tensor components
                    let old_left = context.environment.bindings.insert(left_name.clone(), *left_val);
                    let old_right = context.environment.bindings.insert(right_name.clone(), *right_val);
                    
                    let result = self.eval_with_context(body, context)?;
                    
                    // Restore old bindings
                    if let Some(val) = old_left {
                        context.environment.bindings.insert(left_name.clone(), val);
                    } else {
                        context.environment.bindings.remove(left_name);
                    }
                    if let Some(val) = old_right {
                        context.environment.bindings.insert(right_name.clone(), val);
                    } else {
                        context.environment.bindings.remove(right_name);
                    }
                    
                    Ok(result)
                } else {
                    Err(EvalError::TypeMismatch { 
                        expected: "Tensor".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            
            // Sum types
            ExprKind::Inl(value) => {
                let val = self.eval_with_context(value, context)?;
                Ok(Value::sum(0, val)) // Left variant with tag 0
            }
            ExprKind::Inr(value) => {
                let val = self.eval_with_context(value, context)?;
                Ok(Value::sum(1, val)) // Right variant with tag 1
            }
            ExprKind::Case(expr, left_name, left_branch, right_name, right_branch) => {
                let val = self.eval_with_context(expr, context)?;
                if let ValueKind::Sum { tag: 0, value } = val.kind {
                    // Left branch
                    let old_binding = context.environment.bindings.insert(left_name.clone(), *value);
                    let result = self.eval_with_context(left_branch, context)?;
                    
                    // Restore old binding
                    if let Some(val) = old_binding {
                        context.environment.bindings.insert(left_name.clone(), val);
                    } else {
                        context.environment.bindings.remove(left_name);
                    }
                    
                    Ok(result)
                } else if let ValueKind::Sum { tag: 1, value } = val.kind {
                    // Right branch
                    let old_binding = context.environment.bindings.insert(right_name.clone(), *value);
                    let result = self.eval_with_context(right_branch, context)?;
                    
                    // Restore old binding
                    if let Some(val) = old_binding {
                        context.environment.bindings.insert(right_name.clone(), val);
                    } else {
                        context.environment.bindings.remove(right_name);
                    }
                    
                    Ok(result)
                } else {
                    Err(EvalError::TypeMismatch { 
                        expected: "Sum type".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            
            // Linear functions
            ExprKind::Lambda(params, body) => {
                Ok(Value::lambda(params.clone(), *body.clone()))
            }
            ExprKind::Apply(func_expr, args) => {
                self.eval_apply(func_expr, args, context)
            }
            
            // Resource management
            ExprKind::Alloc(value_expr) => {
                let val = self.eval_with_context(value_expr, context)?;
                // For now, just return the value wrapped as a resource
                Ok(Value::resource("alloc_id", val.type_name()))
            }
            ExprKind::Consume(resource_expr) => {
                let resource_val = self.eval_with_context(resource_expr, context)?;
                // For now, just extract the resource value
                match resource_val.kind {
                    ValueKind::Resource { id, resource_type: _, consumed: _ } => {
                        // Return a simple value representation
                        Ok(Value::string(id))
                    },
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Resource".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            
            // Record operations (simplified implementation for interpreter)
            ExprKind::RecordAccess { record, field } => {
                let record_val = self.eval_with_context(record, context)?;
                match record_val.kind {
                    ValueKind::Record(ref map) => {
                        let field_symbol = Symbol::from(field.as_str());
                        map.get(&field_symbol)
                            .cloned()
                            .ok_or_else(|| EvalError::TypeMismatch {
                                expected: format!("Field '{}'", field),
                                found: "Field not found".to_string(),
                            })
                    },
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Record".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            ExprKind::RecordUpdate { record, field, value } => {
                let record_val = self.eval_with_context(record, context)?;
                let new_value = self.eval_with_context(value, context)?;
                
                match record_val.kind {
                    ValueKind::Record(mut map) => {
                        let field_symbol = Symbol::from(field.as_str());
                        map.insert(field_symbol, new_value);
                        Ok(Value::record(map))
                    },
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Record".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
        }
    }
    
    /// Evaluate a constant value
    fn eval_const(&self, value: &LispValue) -> EvalResult<Value> {
        match value {
            LispValue::Unit => Ok(Value::unit()),
            LispValue::Bool(b) => Ok(Value::bool(*b)),
            LispValue::Int(i) => Ok(Value::int(*i)),
            LispValue::Float(f) => Ok(Value::float(*f)),
            LispValue::String(s) => Ok(Value::string(s.clone())),
            LispValue::Symbol(s) => Ok(Value::symbol(s.clone())),
            LispValue::List(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|item| self.eval_const(item))
                    .collect();
                Ok(Value::list(values?))
            }
            LispValue::Map(map) => {
                let result: Result<HashMap<Symbol, Value>, _> = map
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), self.eval_const(v)?)))
                    .collect();
                Ok(Value::record(result?))
            }
            LispValue::Record(record) => {
                let result: Result<HashMap<Symbol, Value>, _> = record
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), self.eval_const(v)?)))
                    .collect();
                Ok(Value::record(result?))
            }
            _ => Err(EvalError::NotImplemented("Constant evaluation not implemented".to_string())),
        }
    }
    
    /// Evaluate a variable lookup
    fn eval_var(&self, name: &Symbol, context: &EvalContext) -> EvalResult<Value> {
        context.lookup(name)
            .cloned()
            .ok_or_else(|| EvalError::UndefinedVariable(name.to_string()))
    }
    
    /// Evaluate function application
    fn eval_apply(&mut self, func_expr: &Expr, args: &[Expr], context: &mut EvalContext) -> EvalResult<Value> {
        let func_val = self.eval_with_context(func_expr, context)?;
        let arg_vals: Result<Vec<_>, _> = args
            .iter()
            .map(|arg| self.eval_with_context(arg, context))
            .collect();
        let arg_vals = arg_vals?;
        
        match func_val.kind {
            ValueKind::Lambda { params, body } => {
                if params.len() != arg_vals.len() {
                    return Err(EvalError::ArityMismatch { 
                        expected: params.len(), 
                        found: arg_vals.len() 
                    });
                }
                
                let mut new_context = EvalContext::new(); // Create new context for lambda
                for (param, arg_val) in params.iter().zip(arg_vals.iter()) {
                    new_context.bind(param.name.clone(), arg_val.clone());
                }
                
                self.eval_with_context(&body, &mut new_context)
            }
            ValueKind::Function { params, body, closure } => {
                if params.len() != arg_vals.len() {
                    return Err(EvalError::ArityMismatch { 
                        expected: params.len(), 
                        found: arg_vals.len() 
                    });
                }
                
                let mut new_context = EvalContext::from_environment(closure);
                for (param, arg_val) in params.iter().zip(arg_vals.iter()) {
                    new_context.bind(param.clone(), arg_val.clone());
                }
                
                self.eval_with_context(&body, &mut new_context)
            }
            ValueKind::Builtin { name, .. } => {
                self.eval_builtin(&name, &arg_vals)
            }
            _ => Err(EvalError::TypeMismatch { 
                expected: "Function".to_string(), 
                found: "Other".to_string() 
            })
        }
    }
    
    /// Evaluate a built-in function
    fn eval_builtin(&self, name: &Symbol, args: &[Value]) -> EvalResult<Value> {
        match name.as_str() {
            "+" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                match (&args[0].kind, &args[1].kind) {
                    (ValueKind::Int(a), ValueKind::Int(b)) => Ok(Value::int(a + b)),
                    (ValueKind::Float(a), ValueKind::Float(b)) => Ok(Value::float(a + b)),
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Numeric types".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            "-" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                match (&args[0].kind, &args[1].kind) {
                    (ValueKind::Int(a), ValueKind::Int(b)) => Ok(Value::int(a - b)),
                    (ValueKind::Float(a), ValueKind::Float(b)) => Ok(Value::float(a - b)),
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Numeric types".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            "*" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                match (&args[0].kind, &args[1].kind) {
                    (ValueKind::Int(a), ValueKind::Int(b)) => Ok(Value::int(a * b)),
                    (ValueKind::Float(a), ValueKind::Float(b)) => Ok(Value::float(a * b)),
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Numeric types".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            "/" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                match (&args[0].kind, &args[1].kind) {
                    (ValueKind::Int(a), ValueKind::Int(b)) => {
                        if *b == 0 {
                            Err(EvalError::DivisionByZero)
                        } else {
                            Ok(Value::int(a / b))
                        }
                    }
                    (ValueKind::Float(a), ValueKind::Float(b)) => Ok(Value::float(a / b)),
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Numeric types".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            "=" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                Ok(Value::bool(args[0] == args[1]))
            }
            "<" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                match (&args[0].kind, &args[1].kind) {
                    (ValueKind::Int(a), ValueKind::Int(b)) => Ok(Value::bool(a < b)),
                    (ValueKind::Float(a), ValueKind::Float(b)) => Ok(Value::bool(a < b)),
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Numeric types".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            ">" => {
                if args.len() != 2 {
                    return Err(EvalError::ArityMismatch { expected: 2, found: args.len() });
                }
                match (&args[0].kind, &args[1].kind) {
                    (ValueKind::Int(a), ValueKind::Int(b)) => Ok(Value::bool(a > b)),
                    (ValueKind::Float(a), ValueKind::Float(b)) => Ok(Value::bool(a > b)),
                    _ => Err(EvalError::TypeMismatch { 
                        expected: "Numeric types".to_string(), 
                        found: "Other".to_string() 
                    })
                }
            }
            _ => Err(EvalError::UnknownBuiltin(name.to_string()))
        }
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{helpers::*, Param};

    #[test]
    fn test_basic_evaluation() {
        let mut interpreter = Interpreter::new();
        
        // Test constant evaluation
        let expr = int(42);
        let result = interpreter.eval(&expr).unwrap();
        assert_eq!(result.kind, ValueKind::Int(42));
        
        // Test boolean evaluation
        let expr = bool(true);
        let result = interpreter.eval(&expr).unwrap();
        assert_eq!(result.kind, ValueKind::Bool(true));
    }
    
    #[test]
    fn test_arithmetic() {
        let mut interpreter = Interpreter::new();
        
        // Test addition: (+ 1 2)
        let expr = Expr::apply(
            Expr::variable("+"),
            vec![int(1), int(2)]
        );
        let result = interpreter.eval(&expr).unwrap();
        assert_eq!(result.kind, ValueKind::Int(3));
    }
    
    #[test]
    fn test_variable_binding() {
        let mut interpreter = Interpreter::new();
        
        // Test lambda application that simulates let binding: ((λx. x) 42)
        let expr = Expr::apply(
            Expr::lambda(vec![Param::new("x")], Expr::variable("x")),
            vec![int(42)]
        );
        let result = interpreter.eval(&expr).unwrap();
        assert_eq!(result.kind, ValueKind::Int(42));
    }
} 