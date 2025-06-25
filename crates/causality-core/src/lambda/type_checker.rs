//! Type checker for Layer 1 lambda calculus with session types
//!
//! This module implements type checking for the linear lambda calculus
//! with session types, ensuring both type safety and linear resource usage.

use super::{
    term::{Term, TermKind, Literal},
    base::{TypeInner, BaseType, SessionType, SessionEnvironment, SessionEnvironmentError},
};
use std::collections::HashMap;
use thiserror::Error;

/// Type checking errors
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeCheckError {
    /// Type mismatch between expected and actual types
    #[error("Type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch {
        expected: Box<TypeInner>,
        actual: Box<TypeInner>,
    },
    
    /// Cannot apply function to argument
    #[error("Cannot apply function to argument: {0:?}")]
    CannotApply(Box<TypeInner>),
    
    /// Invalid tensor elimination
    #[error("Invalid tensor elimination: {0:?}")]
    InvalidTensorElimination(Box<TypeInner>),
    
    /// Invalid case analysis
    #[error("Invalid case analysis: {0:?}")]
    InvalidCase(Box<TypeInner>),
    
    /// Variable not found in context
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    
    /// Invalid session operation
    #[error("Invalid session operation: {0:?}")]
    InvalidSessionOperation(Box<TypeInner>),
    
    /// Linearity violation
    #[error("Linearity violation: {0}")]
    LinearityViolation(String),
    
    /// Channel not found
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    
    /// Session type error
    #[error("Session type error: {0}")]
    SessionTypeError(String),
    
    #[error("Session type error: {0}")]
    SessionError(#[from] SessionEnvironmentError),
    
    #[error("Session protocol mismatch: cannot perform {operation} on {session_type:?}")]
    SessionProtocolMismatch {
        operation: String,
        session_type: SessionType,
    },
    
    #[error("Choice label '{label}' not found in {session_type:?}")]
    ChoiceLabelNotFound {
        label: String,
        session_type: SessionType,
    },
    
    #[error("Linear variable '{0}' used more than once")]
    LinearVariableReused(String),
    
    #[error("Linear variable '{0}' not used")]
    LinearVariableUnused(String),
    
    #[error("Invalid branch: expected external choice, got {0:?}")]
    InvalidBranch(SessionType),
}

/// Type checking context for variables
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Variable type bindings
    variables: HashMap<String, TypeInner>,
    
    /// Linear variable usage tracking
    linear_usage: HashMap<String, bool>,
    
    /// Session environment for tracking channels
    session_env: SessionEnvironment,
}

impl TypeContext {
    /// Create a new empty type context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            linear_usage: HashMap::new(),
            session_env: SessionEnvironment::new(),
        }
    }
    
    /// Bind a variable with a type
    pub fn bind_variable(&mut self, name: String, ty: TypeInner) -> Result<(), TypeCheckError> {
        self.variables.insert(name.clone(), ty.clone());
        
        // Track linear variables
        if self.is_linear_type(&ty) {
            self.linear_usage.insert(name, false);
        }
        
        Ok(())
    }
    
    /// Look up a variable's type
    pub fn lookup_variable(&self, name: &str) -> Result<&TypeInner, TypeCheckError> {
        self.variables.get(name)
            .ok_or_else(|| TypeCheckError::VariableNotFound(name.to_string()))
    }
    
    /// Use a linear variable (mark as consumed)
    pub fn use_variable(&mut self, name: &str) -> Result<TypeInner, TypeCheckError> {
        let ty = self.lookup_variable(name)?.clone();
        
        if self.is_linear_type(&ty) {
            if let Some(used) = self.linear_usage.get_mut(name) {
                if *used {
                    return Err(TypeCheckError::LinearVariableReused(name.to_string()));
                }
                *used = true;
            }
        }
        
        Ok(ty)
    }
    
    /// Check if a type is linear (requires exactly-once usage)
    fn is_linear_type(&self, ty: &TypeInner) -> bool {
        matches!(ty, 
            TypeInner::Session(_) |
            TypeInner::LinearFunction(_, _) |
            TypeInner::Transform { .. }
        )
    }
    
    /// Bind a channel in the session environment
    pub fn bind_channel(&mut self, name: String, session_type: SessionType) -> Result<(), TypeCheckError> {
        self.session_env.bind_channel(name, session_type)?;
        Ok(())
    }
    
    /// Look up a channel's session type
    pub fn lookup_channel(&self, name: &str) -> Result<&SessionType, TypeCheckError> {
        self.session_env.lookup_channel(name)
            .ok_or_else(|| TypeCheckError::ChannelNotFound(name.to_string()))
    }
    
    /// Update a channel's session type (for protocol progression)
    pub fn update_channel(&mut self, name: &str, new_session_type: SessionType) -> Result<(), TypeCheckError> {
        self.session_env.update_channel(name, new_session_type)?;
        Ok(())
    }
    
    /// Consume a channel (when it's closed)
    pub fn consume_channel(&mut self, name: &str) -> Result<SessionType, TypeCheckError> {
        Ok(self.session_env.consume_channel(name)?)
    }
    
    /// Enter a new scope for session environment
    pub fn enter_scope(&mut self) {
        self.session_env.enter_scope();
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) -> Result<(), TypeCheckError> {
        self.session_env.exit_scope()?;
        Ok(())
    }
    
    /// Check for unused linear variables
    pub fn check_linear_usage(&self) -> Result<(), TypeCheckError> {
        for (name, used) in &self.linear_usage {
            if !used {
                return Err(TypeCheckError::LinearVariableUnused(name.clone()));
            }
        }
        Ok(())
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main type checking function
pub fn type_check(ctx: &mut TypeContext, term: &Term) -> Result<TypeInner, TypeCheckError> {
    match &term.kind {
        TermKind::Var(name) => {
            // First check if it's a channel
            if let Ok(session_type) = ctx.lookup_channel(name) {
                return Ok(TypeInner::Session(Box::new(session_type.clone())));
            }
            
            // Otherwise, use as regular variable
            ctx.use_variable(name)
        }
        
        TermKind::Literal(lit) => {
            Ok(literal_type(lit))
        }
        
        TermKind::Unit => {
            Ok(TypeInner::Base(BaseType::Unit))
        }
        
        TermKind::LetUnit { unit_term, body } => {
            // Check that unit_term has unit type
            let unit_ty = type_check(ctx, unit_term)?;
            if unit_ty != TypeInner::Base(BaseType::Unit) {
                return Err(TypeCheckError::TypeMismatch {
                    expected: Box::new(TypeInner::Base(BaseType::Unit)),
                    actual: Box::new(unit_ty),
                });
            }
            
            // Type check the body
            type_check(ctx, body)
        }
        
        TermKind::Tensor { left, right } => {
            let left_ty = type_check(ctx, left)?;
            let right_ty = type_check(ctx, right)?;
            Ok(TypeInner::Product(Box::new(left_ty), Box::new(right_ty)))
        }
        
        TermKind::LetTensor { tensor_term, left_var, right_var, body } => {
            let tensor_ty = type_check(ctx, tensor_term)?;
            
            match tensor_ty {
                TypeInner::Product(left_ty, right_ty) => {
                    ctx.bind_variable(left_var.clone(), *left_ty)?;
                    ctx.bind_variable(right_var.clone(), *right_ty)?;
                    type_check(ctx, body)
                }
                _ => Err(TypeCheckError::InvalidTensorElimination(Box::new(tensor_ty))),
            }
        }
        
        TermKind::Inl { value, sum_type } => {
            let value_ty = type_check(ctx, value)?;
            
            match sum_type {
                TypeInner::Sum(left_ty, _) => {
                    if value_ty == **left_ty {
                        Ok(sum_type.clone())
                    } else {
                        Err(TypeCheckError::TypeMismatch {
                            expected: Box::new(*left_ty.clone()),
                            actual: Box::new(value_ty),
                        })
                    }
                }
                _ => Err(TypeCheckError::TypeMismatch {
                    expected: Box::new(TypeInner::Sum(Box::new(TypeInner::Base(BaseType::Unit)), Box::new(TypeInner::Base(BaseType::Unit)))),
                    actual: Box::new(sum_type.clone()),
                }),
            }
        }
        
        TermKind::Inr { value, sum_type } => {
            let value_ty = type_check(ctx, value)?;
            
            match sum_type {
                TypeInner::Sum(_, right_ty) => {
                    if value_ty == **right_ty {
                        Ok(sum_type.clone())
                    } else {
                        Err(TypeCheckError::TypeMismatch {
                            expected: Box::new(*right_ty.clone()),
                            actual: Box::new(value_ty),
                        })
                    }
                }
                _ => Err(TypeCheckError::TypeMismatch {
                    expected: Box::new(TypeInner::Sum(Box::new(TypeInner::Base(BaseType::Unit)), Box::new(TypeInner::Base(BaseType::Unit)))),
                    actual: Box::new(sum_type.clone()),
                }),
            }
        }
        
        TermKind::Case { scrutinee, left_var, left_body, right_var, right_body } => {
            let scrutinee_ty = type_check(ctx, scrutinee)?;
            
            match scrutinee_ty {
                TypeInner::Sum(left_ty, right_ty) => {
                    // Type check left branch
                    ctx.enter_scope();
                    ctx.bind_variable(left_var.clone(), *left_ty)?;
                    let left_result_ty = type_check(ctx, left_body)?;
                    ctx.exit_scope()?;
                    
                    // Type check right branch
                    ctx.enter_scope();
                    ctx.bind_variable(right_var.clone(), *right_ty)?;
                    let right_result_ty = type_check(ctx, right_body)?;
                    ctx.exit_scope()?;
                    
                    // Both branches must have the same type
                    if left_result_ty == right_result_ty {
                        Ok(left_result_ty)
                    } else {
                        Err(TypeCheckError::TypeMismatch {
                            expected: Box::new(left_result_ty),
                            actual: Box::new(right_result_ty),
                        })
                    }
                }
                _ => Err(TypeCheckError::InvalidCase(Box::new(scrutinee_ty))),
            }
        }
        
        TermKind::Lambda { param, param_type, body } => {
            ctx.enter_scope();
            
            let param_ty = param_type.clone().unwrap_or(TypeInner::Base(BaseType::Unit));
            ctx.bind_variable(param.clone(), param_ty.clone())?;
            
            let body_ty = type_check(ctx, body)?;
            ctx.exit_scope()?;
            
            Ok(TypeInner::LinearFunction(Box::new(param_ty), Box::new(body_ty)))
        }
        
        TermKind::Apply { func, arg } => {
            let func_ty = type_check(ctx, func)?;
            let arg_ty = type_check(ctx, arg)?;
            
            match func_ty {
                TypeInner::LinearFunction(param_ty, result_ty) => {
                    if arg_ty == *param_ty {
                        Ok(*result_ty)
                    } else {
                        Err(TypeCheckError::TypeMismatch {
                            expected: Box::new(*param_ty),
                            actual: Box::new(arg_ty),
                        })
                    }
                }
                _ => Err(TypeCheckError::CannotApply(Box::new(func_ty))),
            }
        }
        
        TermKind::Alloc { value } => {
            let value_ty = type_check(ctx, value)?;
            // For now, alloc just returns the same type
            // In a full implementation, this would wrap in a resource type
            Ok(value_ty)
        }
        
        TermKind::Consume { resource } => {
            let resource_ty = type_check(ctx, resource)?;
            // For now, consume just returns the inner type
            // In a full implementation, this would unwrap a resource type
            Ok(resource_ty)
        }
        
        TermKind::Let { var, value, body } => {
            let value_ty = type_check(ctx, value)?;
            ctx.bind_variable(var.clone(), value_ty)?;
            type_check(ctx, body)
        }
        
        // Session type constructors
        
        TermKind::NewChannel { session_type } => {
            // Create a new channel with the given session type
            Ok(TypeInner::Session(Box::new(session_type.clone())))
        }
        
        TermKind::Send { channel, value } => {
            let channel_ty = type_check(ctx, channel)?;
            let value_ty = type_check(ctx, value)?;
            
            match channel_ty {
                TypeInner::Session(session_ty) => {
                    match *session_ty {
                        SessionType::Send(expected_ty, continuation) => {
                            if value_ty == *expected_ty {
                                // Update channel to continuation type
                                if let TermKind::Var(channel_name) = &channel.kind {
                                    ctx.update_channel(channel_name, *continuation)?;
                                }
                                Ok(TypeInner::Base(BaseType::Unit))
                            } else {
                                Err(TypeCheckError::TypeMismatch {
                                    expected: Box::new(*expected_ty),
                                    actual: Box::new(value_ty),
                                })
                            }
                        }
                        _ => Err(TypeCheckError::SessionProtocolMismatch {
                            operation: "send".to_string(),
                            session_type: *session_ty,
                        }),
                    }
                }
                _ => Err(TypeCheckError::InvalidSessionOperation(Box::new(channel_ty))),
            }
        }
        
        TermKind::Receive { channel } => {
            let channel_ty = type_check(ctx, channel)?;
            
            match channel_ty {
                TypeInner::Session(session_ty) => {
                    match *session_ty {
                        SessionType::Receive(value_ty, continuation) => {
                            // Update channel to continuation type
                            if let TermKind::Var(channel_name) = &channel.kind {
                                ctx.update_channel(channel_name, *continuation)?;
                            }
                            Ok(*value_ty)
                        }
                        _ => Err(TypeCheckError::SessionProtocolMismatch {
                            operation: "receive".to_string(),
                            session_type: *session_ty,
                        }),
                    }
                }
                _ => Err(TypeCheckError::InvalidSessionOperation(Box::new(channel_ty))),
            }
        }
        
        TermKind::Select { channel, label } => {
            let channel_ty = type_check(ctx, channel)?;
            
            match channel_ty {
                TypeInner::Session(session_ty) => {
                    match *session_ty {
                        SessionType::InternalChoice(ref choices) => {
                            // Find the selected choice
                            for (choice_label, continuation) in choices {
                                if choice_label == label {
                                    // Update channel to continuation type
                                    if let TermKind::Var(channel_name) = &channel.kind {
                                        ctx.update_channel(channel_name, continuation.clone())?;
                                    }
                                    return Ok(TypeInner::Base(BaseType::Unit));
                                }
                            }
                            Err(TypeCheckError::ChoiceLabelNotFound {
                                label: label.clone(),
                                session_type: *session_ty,
                            })
                        }
                        _ => Err(TypeCheckError::SessionProtocolMismatch {
                            operation: "select".to_string(),
                            session_type: *session_ty,
                        }),
                    }
                }
                _ => Err(TypeCheckError::InvalidSessionOperation(Box::new(channel_ty))),
            }
        }
        
        TermKind::Branch { channel, branches } => {
            let channel_ty = type_check(ctx, channel)?;
            
            match channel_ty {
                TypeInner::Session(session_ty) => {
                    match *session_ty {
                        SessionType::ExternalChoice(ref choices) => {
                            let mut result_types = Vec::new();
                            
                            // Type check each branch
                            for (label, branch_term) in branches {
                                // Find the corresponding choice
                                let mut found = false;
                                for (choice_label, continuation) in choices {
                                    if choice_label == label {
                                        ctx.enter_scope();
                                        // Update channel to continuation type for this branch
                                        if let TermKind::Var(channel_name) = &channel.kind {
                                            ctx.update_channel(channel_name, continuation.clone())?;
                                        }
                                        let branch_ty = type_check(ctx, branch_term)?;
                                        result_types.push(branch_ty);
                                        ctx.exit_scope()?;
                                        found = true;
                                        break;
                                    }
                                }
                                
                                if !found {
                                    return Err(TypeCheckError::ChoiceLabelNotFound {
                                        label: label.clone(),
                                        session_type: *session_ty,
                                    });
                                }
                            }
                            
                            // All branches must have the same result type
                            if let Some(first_ty) = result_types.first() {
                                for ty in &result_types[1..] {
                                    if ty != first_ty {
                                        return Err(TypeCheckError::TypeMismatch {
                                            expected: Box::new(first_ty.clone()),
                                            actual: Box::new(ty.clone()),
                                        });
                                    }
                                }
                                Ok(first_ty.clone())
                            } else {
                                Ok(TypeInner::Base(BaseType::Unit))
                            }
                        }
                        _ => Err(TypeCheckError::InvalidBranch(*session_ty)),
                    }
                }
                _ => Err(TypeCheckError::InvalidSessionOperation(Box::new(channel_ty))),
            }
        }
        
        TermKind::Close { channel } => {
            let channel_ty = type_check(ctx, channel)?;
            
            match channel_ty {
                TypeInner::Session(session_ty) => {
                    match *session_ty {
                        SessionType::End => {
                            // Consume the channel
                            if let TermKind::Var(channel_name) = &channel.kind {
                                ctx.consume_channel(channel_name)?;
                            }
                            Ok(TypeInner::Base(BaseType::Unit))
                        }
                        _ => Err(TypeCheckError::SessionProtocolMismatch {
                            operation: "close".to_string(),
                            session_type: *session_ty,
                        }),
                    }
                }
                _ => Err(TypeCheckError::InvalidSessionOperation(Box::new(channel_ty))),
            }
        }
        
        TermKind::Fork { session_type, client_var, server_var, body } => {
            ctx.enter_scope();
            
            // Bind client and server channels with dual session types
            let client_session = session_type.clone();
            let server_session = session_type.dual();
            
            ctx.bind_channel(client_var.clone(), client_session.clone())?;
            ctx.bind_channel(server_var.clone(), server_session.clone())?;
            
            // Also bind as regular variables so they can be referenced
            ctx.bind_variable(client_var.clone(), TypeInner::Session(Box::new(client_session)))?;
            ctx.bind_variable(server_var.clone(), TypeInner::Session(Box::new(server_session)))?;
            
            let body_ty = type_check(ctx, body)?;
            ctx.exit_scope()?;
            
            Ok(body_ty)
        }
        
        TermKind::Wait { channel, body } => {
            let channel_ty = type_check(ctx, channel)?;
            
            // Verify channel is closed/at End
            match channel_ty {
                TypeInner::Session(session_ty) => {
                    match *session_ty {
                        SessionType::End => {
                            type_check(ctx, body)
                        }
                        _ => Err(TypeCheckError::SessionProtocolMismatch {
                            operation: "wait".to_string(),
                            session_type: *session_ty,
                        }),
                    }
                }
                _ => Err(TypeCheckError::InvalidSessionOperation(Box::new(channel_ty))),
            }
        }
        
        // Transform type constructors
        
        TermKind::Transform { input_type, output_type, location, body } => {
            ctx.enter_scope();
            
            // Bind the input parameter (assuming it's named "x" for transforms)
            ctx.bind_variable("x".to_string(), input_type.clone())?;
            
            // Type check the body
            let body_ty = type_check(ctx, body)?;
            
            // Verify the body type matches the declared output type
            if body_ty != *output_type {
                ctx.exit_scope()?;
                return Err(TypeCheckError::TypeMismatch {
                    expected: Box::new(output_type.clone()),
                    actual: Box::new(body_ty),
                });
            }
            
            ctx.exit_scope()?;
            
            // Return the Transform type
            Ok(TypeInner::Transform {
                input: Box::new(input_type.clone()),
                output: Box::new(output_type.clone()),
                location: location.clone(),
            })
        }
        
        TermKind::ApplyTransform { transform, arg } => {
            let transform_ty = type_check(ctx, transform)?;
            let arg_ty = type_check(ctx, arg)?;
            
            match transform_ty {
                TypeInner::Transform { input, output, .. } => {
                    if arg_ty == *input {
                        Ok(*output)
                    } else {
                        Err(TypeCheckError::TypeMismatch {
                            expected: Box::new(*input),
                            actual: Box::new(arg_ty),
                        })
                    }
                }
                _ => Err(TypeCheckError::CannotApply(Box::new(transform_ty))),
            }
        }
        
        TermKind::At { location: _, body } => {
            // For now, "at" just type checks the body
            // In a full implementation, this would check location constraints
            type_check(ctx, body)
        }
    }
}

/// Get the type of a literal
fn literal_type(lit: &Literal) -> TypeInner {
    match lit {
        Literal::Unit => TypeInner::Base(BaseType::Unit),
        Literal::Bool(_) => TypeInner::Base(BaseType::Bool),
        Literal::Int(_) => TypeInner::Base(BaseType::Int),
        Literal::Symbol(_) => TypeInner::Base(BaseType::Symbol),
    }
}

/// Session type inference for terms with incomplete type annotations
pub fn infer_session_types(ctx: &mut TypeContext, term: &Term) -> Result<(TypeInner, Vec<SessionTypeConstraint>), TypeCheckError> {
    let mut constraints = Vec::new();
    let inferred_type = infer_with_constraints(ctx, term, &mut constraints)?;
    Ok((inferred_type, constraints))
}

/// Session type constraint for inference
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionTypeConstraint {
    /// Channel must have a specific session type
    ChannelType(String, SessionType),
    
    /// Two session types must be dual
    Dual(SessionType, SessionType),
    
    /// Session type must support a specific operation
    SupportsOperation(SessionType, SessionOperation),
    
    /// Session type must be compatible with another
    Compatible(SessionType, SessionType),
}

/// Session operations for constraint generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionOperation {
    Send(TypeInner),
    Receive(TypeInner),
    Select(String),
    Branch(Vec<String>),
    Close,
}

/// Type inference with constraint generation
fn infer_with_constraints(
    ctx: &mut TypeContext, 
    term: &Term, 
    constraints: &mut Vec<SessionTypeConstraint>
) -> Result<TypeInner, TypeCheckError> {
    match &term.kind {
        // For most terms, use regular type checking
        TermKind::Var(_) | TermKind::Literal(_) | TermKind::Unit => {
            type_check(ctx, term)
        }
        
        TermKind::NewChannel { session_type } => {
            // Generate constraint that this channel has the specified session type
            Ok(TypeInner::Session(Box::new(session_type.clone())))
        }
        
        TermKind::Send { channel, value } => {
            let value_ty = infer_with_constraints(ctx, value, constraints)?;
            
            match &channel.kind {
                TermKind::Var(channel_name) => {
                    // Generate fresh session type variable for inference
                    let fresh_continuation = generate_fresh_session_var();
                    let expected_session = SessionType::Send(
                        Box::new(value_ty),
                        Box::new(fresh_continuation.clone())
                    );
                    
                    // Add constraint that channel must support send operation
                    constraints.push(SessionTypeConstraint::ChannelType(
                        channel_name.clone(),
                        expected_session
                    ));
                    
                    // Update channel to continuation type
                    ctx.update_channel(channel_name, fresh_continuation)?;
                    
                    Ok(TypeInner::Base(BaseType::Unit))
                }
                _ => {
                    // For complex channel expressions, fall back to regular type checking
                    type_check(ctx, term)
                }
            }
        }
        
        TermKind::Receive { channel } => {
            match &channel.kind {
                TermKind::Var(channel_name) => {
                    // Generate fresh type variable for received value
                    let fresh_value_type = generate_fresh_type_var();
                    let fresh_continuation = generate_fresh_session_var();
                    let expected_session = SessionType::Receive(
                        Box::new(fresh_value_type.clone()),
                        Box::new(fresh_continuation.clone())
                    );
                    
                    // Add constraint that channel must support receive operation
                    constraints.push(SessionTypeConstraint::ChannelType(
                        channel_name.clone(),
                        expected_session
                    ));
                    
                    // Update channel to continuation type
                    ctx.update_channel(channel_name, fresh_continuation)?;
                    
                    Ok(fresh_value_type)
                }
                _ => {
                    // For complex channel expressions, fall back to regular type checking
                    type_check(ctx, term)
                }
            }
        }
        
        TermKind::Select { channel, label } => {
            match &channel.kind {
                TermKind::Var(channel_name) => {
                    // Generate fresh continuation for the selected branch
                    let fresh_continuation = generate_fresh_session_var();
                    let expected_session = SessionType::InternalChoice(vec![
                        (label.clone(), fresh_continuation.clone())
                    ]);
                    
                    // Add constraint that channel must support select operation
                    constraints.push(SessionTypeConstraint::ChannelType(
                        channel_name.clone(),
                        expected_session
                    ));
                    
                    // Update channel to continuation type
                    ctx.update_channel(channel_name, fresh_continuation)?;
                    
                    Ok(TypeInner::Base(BaseType::Unit))
                }
                _ => {
                    type_check(ctx, term)
                }
            }
        }
        
        TermKind::Branch { channel, branches } => {
            match &channel.kind {
                TermKind::Var(channel_name) => {
                    let mut branch_constraints = Vec::new();
                    let mut result_types = Vec::new();
                    
                    // Infer types for each branch
                    for (label, branch_term) in branches {
                        ctx.enter_scope();
                        
                        let fresh_continuation = generate_fresh_session_var();
                        ctx.update_channel(channel_name, fresh_continuation.clone())?;
                        
                        let branch_ty = infer_with_constraints(ctx, branch_term, constraints)?;
                        result_types.push(branch_ty.clone());
                        branch_constraints.push((label.clone(), fresh_continuation));
                        
                        ctx.exit_scope()?;
                    }
                    
                    // Generate external choice constraint
                    let expected_session = SessionType::ExternalChoice(branch_constraints);
                    constraints.push(SessionTypeConstraint::ChannelType(
                        channel_name.clone(),
                        expected_session
                    ));
                    
                    // All branches should have the same result type
                    if let Some(first_ty) = result_types.first() {
                        for _ty in &result_types[1..] {
                            // For now, we skip type compatibility constraints
                            // In a full implementation, this would check type unification
                        }
                        Ok(first_ty.clone())
                    } else {
                        Ok(TypeInner::Base(BaseType::Unit))
                    }
                }
                _ => {
                    type_check(ctx, term)
                }
            }
        }
        
        TermKind::Fork { session_type, client_var, server_var, body } => {
            ctx.enter_scope();
            
            // Bind channels with dual session types
            let client_session = session_type.clone();
            let server_session = session_type.dual();
            
            // Add duality constraint
            constraints.push(SessionTypeConstraint::Dual(
                client_session.clone(),
                server_session.clone()
            ));
            
            ctx.bind_channel(client_var.clone(), client_session.clone())?;
            ctx.bind_channel(server_var.clone(), server_session.clone())?;
            ctx.bind_variable(client_var.clone(), TypeInner::Session(Box::new(client_session)))?;
            ctx.bind_variable(server_var.clone(), TypeInner::Session(Box::new(server_session)))?;
            
            let body_ty = infer_with_constraints(ctx, body, constraints)?;
            ctx.exit_scope()?;
            
            Ok(body_ty)
        }
        
        // For other terms, use regular type checking
        _ => {
            type_check(ctx, term)
        }
    }
}

/// Generate a fresh session type variable for inference
fn generate_fresh_session_var() -> SessionType {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    let _id = COUNTER.fetch_add(1, Ordering::SeqCst);
    SessionType::Variable(format!("S{}", _id))
}

/// Generate a fresh type variable for inference
fn generate_fresh_type_var() -> TypeInner {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    let _id = COUNTER.fetch_add(1, Ordering::SeqCst);
    TypeInner::Base(BaseType::Symbol) // Placeholder - in a full implementation this would be a type variable
}

/// Solve session type constraints
pub fn solve_constraints(constraints: &[SessionTypeConstraint]) -> Result<HashMap<String, SessionType>, TypeCheckError> {
    let mut solution = HashMap::new();
    
    for constraint in constraints {
        match constraint {
            SessionTypeConstraint::ChannelType(channel_name, session_type) => {
                // Bind channel to session type
                solution.insert(channel_name.clone(), session_type.clone());
            }
            
            SessionTypeConstraint::Dual(s1, s2) => {
                // Check that s1 and s2 are dual
                if !s1.is_dual_to(s2) {
                    return Err(TypeCheckError::SessionProtocolMismatch {
                        operation: "duality".to_string(),
                        session_type: s1.clone(),
                    });
                }
            }
            
            SessionTypeConstraint::SupportsOperation(session_type, operation) => {
                // Check that session type supports the operation
                match (session_type, operation) {
                    (SessionType::Send(_, _), SessionOperation::Send(_)) => {}
                    (SessionType::Receive(_, _), SessionOperation::Receive(_)) => {}
                    (SessionType::InternalChoice(_), SessionOperation::Select(_)) => {}
                    (SessionType::ExternalChoice(_), SessionOperation::Branch(_)) => {}
                    (SessionType::End, SessionOperation::Close) => {}
                    _ => {
                        return Err(TypeCheckError::SessionProtocolMismatch {
                            operation: format!("{:?}", operation),
                            session_type: session_type.clone(),
                        });
                    }
                }
            }
            
            SessionTypeConstraint::Compatible(_, _) => {
                // For now, assume all types are compatible
                // In a full implementation, this would perform unification
            }
        }
    }
    
    Ok(solution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::term::Term;
    
    #[test]
    fn test_basic_type_checking() {
        let mut ctx = TypeContext::new();
        
        // Test literals
        assert_eq!(type_check(&mut ctx, &Term::unit()).unwrap(), TypeInner::Base(BaseType::Unit));
        assert_eq!(type_check(&mut ctx, &Term::literal(Literal::Bool(true))).unwrap(), TypeInner::Base(BaseType::Bool));
        assert_eq!(type_check(&mut ctx, &Term::literal(Literal::Int(42))).unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_tensor_types() {
        let mut ctx = TypeContext::new();
        
        let tensor_term = Term::tensor(
            Term::literal(Literal::Int(42)),
            Term::literal(Literal::Bool(true))
        );
        
        let result_ty = type_check(&mut ctx, &tensor_term).unwrap();
        
        match result_ty {
            TypeInner::Product(left, right) => {
                assert_eq!(*left, TypeInner::Base(BaseType::Int));
                assert_eq!(*right, TypeInner::Base(BaseType::Bool));
            }
            _ => panic!("Expected product type"),
        }
    }
    
    #[test]
    fn test_lambda_types() {
        let mut ctx = TypeContext::new();
        
        let lambda_term = Term::lambda_typed(
            "x",
            TypeInner::Base(BaseType::Int),
            Term::var("x")
        );
        
        let result_ty = type_check(&mut ctx, &lambda_term).unwrap();
        
        match result_ty {
            TypeInner::LinearFunction(param_ty, result_ty) => {
                assert_eq!(*param_ty, TypeInner::Base(BaseType::Int));
                assert_eq!(*result_ty, TypeInner::Base(BaseType::Int));
            }
            _ => panic!("Expected function type"),
        }
    }
    
    #[test]
    fn test_session_new_channel() {
        let mut ctx = TypeContext::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let new_channel_term = Term::new_channel(session_type.clone());
        let result_ty = type_check(&mut ctx, &new_channel_term).unwrap();
        
        match result_ty {
            TypeInner::Session(st) => {
                assert_eq!(*st, session_type);
            }
            _ => panic!("Expected session type"),
        }
    }
    
    #[test]
    fn test_session_send_receive() {
        let mut ctx = TypeContext::new();
        
        // Create a session type: Send Int; Receive Bool; End
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::Receive(
                Box::new(TypeInner::Base(BaseType::Bool)),
                Box::new(SessionType::End)
            ))
        );
        
        // Bind a channel
        ctx.bind_channel("ch".to_string(), session_type.clone()).unwrap();
        ctx.bind_variable("ch".to_string(), TypeInner::Session(Box::new(session_type))).unwrap();
        
        // Test send
        let send_term = Term::send(Term::var("ch"), Term::literal(Literal::Int(42)));
        let send_result = type_check(&mut ctx, &send_term).unwrap();
        assert_eq!(send_result, TypeInner::Base(BaseType::Unit));
        
        // Test receive (channel should now be at Receive Bool; End)
        let receive_term = Term::receive(Term::var("ch"));
        let receive_result = type_check(&mut ctx, &receive_term).unwrap();
        assert_eq!(receive_result, TypeInner::Base(BaseType::Bool));
        
        // Test close (channel should now be at End)
        let close_term = Term::close(Term::var("ch"));
        let close_result = type_check(&mut ctx, &close_term).unwrap();
        assert_eq!(close_result, TypeInner::Base(BaseType::Unit));
    }
    
    #[test]
    fn test_session_choice() {
        let mut ctx = TypeContext::new();
        
        // Create internal choice: Select { option_a: End, option_b: End }
        let session_type = SessionType::InternalChoice(vec![
            ("option_a".to_string(), SessionType::End),
            ("option_b".to_string(), SessionType::End),
        ]);
        
        ctx.bind_channel("ch".to_string(), session_type.clone()).unwrap();
        ctx.bind_variable("ch".to_string(), TypeInner::Session(Box::new(session_type))).unwrap();
        
        // Test select
        let select_term = Term::select(Term::var("ch"), "option_a");
        let select_result = type_check(&mut ctx, &select_term).unwrap();
        assert_eq!(select_result, TypeInner::Base(BaseType::Unit));
    }
    
    #[test]
    fn test_session_fork() {
        let mut ctx = TypeContext::new();
        
        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End)
        );
        
        let body = Term::send(Term::var("client"), Term::literal(Literal::Int(42)));
        let fork_term = Term::fork(session_type, "client", "server", body);
        
        let result = type_check(&mut ctx, &fork_term);
        assert!(result.is_ok());
    }
    
    // --- Session Type Inference Tests ---
    
    #[test]
    fn test_session_type_inference_send() {
        let mut ctx = TypeContext::new();
        
        // First, bind a channel with a placeholder session type
        let placeholder_session = SessionType::Variable("S0".to_string());
        ctx.bind_channel("ch".to_string(), placeholder_session.clone()).unwrap();
        ctx.bind_variable("ch".to_string(), TypeInner::Session(Box::new(placeholder_session))).unwrap();
        
        // Create a send term without explicit session type
        let send_term = Term::send(Term::var("ch"), Term::literal(Literal::Int(42)));
        
        let result = infer_session_types(&mut ctx, &send_term);
        assert!(result.is_ok());
        
        let (inferred_type, constraints) = result.unwrap();
        assert_eq!(inferred_type, TypeInner::Base(BaseType::Unit));
        
        // Should generate a constraint for the channel
        assert!(!constraints.is_empty());
        if let SessionTypeConstraint::ChannelType(channel_name, session_type) = &constraints[0] {
            assert_eq!(channel_name, "ch");
            match session_type {
                SessionType::Send(value_type, _) => {
                    assert_eq!(**value_type, TypeInner::Base(BaseType::Int));
                }
                _ => panic!("Expected Send session type"),
            }
        } else {
            panic!("Expected ChannelType constraint");
        }
    }
    
    #[test]
    fn test_constraint_solving() {
        let constraints = vec![
            SessionTypeConstraint::ChannelType(
                "ch1".to_string(),
                SessionType::Send(
                    Box::new(TypeInner::Base(BaseType::Int)),
                    Box::new(SessionType::End)
                )
            ),
        ];
        
        let solution = solve_constraints(&constraints);
        assert!(solution.is_ok());
        
        let solution = solution.unwrap();
        assert!(solution.contains_key("ch1"));
    }
    
    // --- Transform Type Checking Tests ---
    
    #[test]
    fn test_transform_type_checking() {
        let mut ctx = TypeContext::new();
        
        // Create a transform that takes an int and returns a bool
        let transform_term = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Bool),
            crate::lambda::base::Location::Local,
            Term::literal(Literal::Bool(true)) // Body just returns true
        );
        
        let result = type_check(&mut ctx, &transform_term);
        assert!(result.is_ok());
        
        let transform_type = result.unwrap();
        match transform_type {
            TypeInner::Transform { input, output, location } => {
                assert_eq!(*input, TypeInner::Base(BaseType::Int));
                assert_eq!(*output, TypeInner::Base(BaseType::Bool));
                assert_eq!(location, crate::lambda::base::Location::Local);
            }
            _ => panic!("Expected Transform type"),
        }
    }
    
    #[test]
    fn test_transform_application() {
        let mut ctx = TypeContext::new();
        
        // Create a transform that doubles an integer (conceptually)
        let transform_term = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Int),
            crate::lambda::base::Location::Local,
            Term::var("x") // Body uses the input parameter
        );
        
        // Apply the transform to an integer
        let application = Term::apply_transform(
            transform_term,
            Term::literal(Literal::Int(42))
        );
        
        let result = type_check(&mut ctx, &application);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_transform_type_mismatch() {
        let mut ctx = TypeContext::new();
        
        // Create a transform that claims to return Bool but actually returns Int
        let bad_transform = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Bool), // Claims to return Bool
            crate::lambda::base::Location::Local,
            Term::literal(Literal::Int(42)) // But returns Int
        );
        
        let result = type_check(&mut ctx, &bad_transform);
        assert!(result.is_err());
        
        if let Err(TypeCheckError::TypeMismatch { expected, actual }) = result {
            assert_eq!(expected, Box::new(TypeInner::Base(BaseType::Bool)));
            assert_eq!(actual, Box::new(TypeInner::Base(BaseType::Int)));
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
    
    #[test]
    fn test_transform_application_type_mismatch() {
        let mut ctx = TypeContext::new();
        
        // Create a transform that expects Int
        let transform_term = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Bool),
            crate::lambda::base::Location::Local,
            Term::literal(Literal::Bool(true))
        );
        
        // Try to apply it to a Bool (wrong type)
        let bad_application = Term::apply_transform(
            transform_term,
            Term::literal(Literal::Bool(false))
        );
        
        let result = type_check(&mut ctx, &bad_application);
        assert!(result.is_err());
        
        if let Err(TypeCheckError::TypeMismatch { expected, actual }) = result {
            assert_eq!(expected, Box::new(TypeInner::Base(BaseType::Int)));
            assert_eq!(actual, Box::new(TypeInner::Base(BaseType::Bool)));
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
    
    #[test]
    fn test_located_computation() {
        let mut ctx = TypeContext::new();
        
        // Create a computation that runs at a remote location
        let at_term = Term::at(
            crate::lambda::base::Location::Remote(EntityId::from_content(&"server".as_bytes().to_vec())),
            Term::literal(Literal::Int(42))
        );
        
        let result = type_check(&mut ctx, &at_term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInner::Base(BaseType::Int));
    }
    
    #[test]
    fn test_transform_with_remote_location() {
        let mut ctx = TypeContext::new();
        
        // Create a transform that runs on a remote server
        let remote_transform = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Int),
            crate::lambda::base::Location::Remote(EntityId::from_content(&"gpu_server".as_bytes().to_vec())),
            Term::var("x")
        );
        
        let result = type_check(&mut ctx, &remote_transform);
        assert!(result.is_ok());
        
        let transform_type = result.unwrap();
        match transform_type {
            TypeInner::Transform { location, .. } => {
                assert_eq!(location, crate::lambda::base::Location::Remote(EntityId::from_content(&"gpu_server".as_bytes().to_vec())));
            }
            _ => panic!("Expected Transform type"),
        }
    }
} 