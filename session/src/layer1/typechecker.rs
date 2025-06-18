// Type checking for Layer 1 terms with row-polymorphic records

use crate::layer1::{
    Term, Variable, Type, RowType,
};
use crate::layer1::{LinearContext, LinearityError};
use std::collections::HashMap;
use thiserror::Error;

/// Type checking errors
#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Variable {0:?} not found")]
    VariableNotFound(Variable),
    
    #[error("Type mismatch: expected {expected:?}, got {got:?}")]
    TypeMismatch { expected: Type, got: Type },
    
    #[error("Field {0} not found in record")]
    FieldNotFound(String),
    
    #[error("Cannot project from non-record type")]
    ProjectFromNonRecord,
    
    #[error("Cannot extend non-record type")]
    ExtendNonRecord,
    
    #[error("Session type mismatch")]
    SessionTypeMismatch,
    
    #[error("Linearity error: {0}")]
    LinearityError(#[from] LinearityError),
}

/// Type checking context
pub struct TypeContext {
    /// Type bindings for variables
    types: HashMap<Variable, Type>,
    
    /// Linear context for tracking usage
    linear: LinearContext,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    /// Create a new empty context
    pub fn new() -> Self {
        TypeContext {
            types: HashMap::new(),
            linear: LinearContext::new(),
        }
    }
    
    /// Bind a variable with a type
    pub fn bind(&mut self, var: Variable, ty: Type) -> Result<(), TypeError> {
        self.types.insert(var.clone(), ty.clone());
        self.linear.bind(var, ty)?;
        Ok(())
    }
    
    /// Get the type of a variable
    pub fn get_type(&self, var: &Variable) -> Result<&Type, TypeError> {
        self.types.get(var)
            .ok_or_else(|| TypeError::VariableNotFound(var.clone()))
    }
    
    /// Use a linear variable
    pub fn use_linear(&mut self, var: &Variable) -> Result<Type, TypeError> {
        Ok(self.linear.use_var(var)?)
    }
}

/// Type check a term
pub fn typecheck(ctx: &mut TypeContext, term: &Term) -> Result<Type, TypeError> {
    match term {
        Term::Var(v) => {
            // For linear types, mark as used
            let ty = ctx.get_type(v)?.clone();
            if is_linear_type(&ty) {
                ctx.use_linear(v)?;
            }
            Ok(ty)
        }
        
        Term::Unit => Ok(Type::Unit),
        Term::Bool(_) => Ok(Type::Bool),
        Term::Int(_) => Ok(Type::Int),
        
        Term::Pair(t1, t2) => {
            let ty1 = typecheck(ctx, t1)?;
            let ty2 = typecheck(ctx, t2)?;
            Ok(Type::Product(Box::new(ty1), Box::new(ty2)))
        }
        
        Term::Fst(t) => {
            match typecheck(ctx, t)? {
                Type::Product(ty1, _) => Ok(*ty1),
                _ => Err(TypeError::TypeMismatch {
                    expected: Type::Product(Box::new(Type::Unit), Box::new(Type::Unit)),
                    got: Type::Unit,
                }),
            }
        }
        
        Term::Snd(t) => {
            match typecheck(ctx, t)? {
                Type::Product(_, ty2) => Ok(*ty2),
                _ => Err(TypeError::TypeMismatch {
                    expected: Type::Product(Box::new(Type::Unit), Box::new(Type::Unit)),
                    got: Type::Unit,
                }),
            }
        }
        
        Term::Record(fields) => {
            // Type check all fields
            let mut field_types = Vec::new();
            for (label, term) in fields {
                let ty = typecheck(ctx, term)?;
                field_types.push((label.clone(), ty));
            }
            
            // Create row type from fields
            let row_type = RowType::from_fields(field_types);
            Ok(Type::Record(row_type))
        }
        
        Term::Project { record, label } => {
            match typecheck(ctx, record)? {
                Type::Record(row) => {
                    // Check if field exists and get its type
                    row.get_field_type(label)
                        .cloned()
                        .ok_or_else(|| TypeError::FieldNotFound(label.clone()))
                }
                _ => Err(TypeError::ProjectFromNonRecord),
            }
        }
        
        Term::Extend { record, label, value } => {
            match typecheck(ctx, record)? {
                Type::Record(row) => {
                    let val_ty = typecheck(ctx, value)?;
                    // Create extended row type
                    let new_row = RowType::Extend(
                        label.clone(),
                        Box::new(val_ty),
                        Box::new(row)
                    );
                    Ok(Type::Record(new_row))
                }
                _ => Err(TypeError::ExtendNonRecord),
            }
        }
        
        Term::Restrict { record, labels } => {
            match typecheck(ctx, record)? {
                Type::Record(row) => {
                    // Create restricted row type by filtering fields
                    let field_map = row.to_field_map();
                    let mut remaining_fields = Vec::new();
                    
                    for (field_label, field_type) in field_map {
                        if !labels.contains(&field_label) {
                            remaining_fields.push((field_label, field_type));
                        }
                    }
                    
                    let new_row = RowType::from_fields(remaining_fields);
                    Ok(Type::Record(new_row))
                }
                _ => Err(TypeError::ExtendNonRecord),
            }
        }
        
        Term::Let { var, value, body } => {
            // Type check the value
            let val_ty = typecheck(ctx, value)?;
            
            // Bind the variable
            ctx.bind(var.clone(), val_ty)?;
            
            // Type check the body
            typecheck(ctx, body)
        }
        
        // TODO: Add session type operations
        _ => Err(TypeError::TypeMismatch {
            expected: Type::Unit,
            got: Type::Unit,
        }),
    }
}

/// Check if a type is linear  
fn is_linear_type(ty: &Type) -> bool {
    matches!(ty, Type::Record(_) | Type::Session(_))
}

/// Check row polymorphic subtyping
pub fn is_row_subtype(sub: &RowType, sup: &RowType) -> bool {
    // A row is a subtype if it has at least all the fields of the supertype
    match sup {
        RowType::Empty => true,
        RowType::Extend(label, ty, rest) => {
            // Check if sub has this field with compatible type
            match sub.get_field_type(label) {
                Some(sub_ty) if sub_ty == &**ty => is_row_subtype(sub, rest),
                _ => false,
            }
        }
        RowType::RowVar(_) => true, // Row variables can be instantiated to anything
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let mut ctx = TypeContext::new();
        
        // Create a record { x: 42, y: true }
        let term = Term::record(vec![
            ("x", Term::Int(42)),
            ("y", Term::Bool(true)),
        ]);
        
        let ty = typecheck(&mut ctx, &term).unwrap();
        
        match ty {
            Type::Record(row) => {
                assert!(row.has_field("x"));
                assert!(row.has_field("y"));
                assert_eq!(row.get_field_type("x"), Some(&Type::Int));
                assert_eq!(row.get_field_type("y"), Some(&Type::Bool));
            }
            _ => panic!("Expected record type"),
        }
    }
    
    #[test]
    fn test_record_projection() {
        let mut ctx = TypeContext::new();
        
        // let r = { x: 42 } in r.x
        let term = Term::let_bind(
            "r",
            Term::record(vec![("x", Term::Int(42))]),
            Term::project(Term::var("r"), "x")
        );
        
        let ty = typecheck(&mut ctx, &term).unwrap();
        assert_eq!(ty, Type::Int);
    }
    
    #[test]
    fn test_record_extension() {
        let mut ctx = TypeContext::new();
        
        // let r = { x: 42 } in extend(r, y, true)
        let term = Term::let_bind(
            "r",
            Term::record(vec![("x", Term::Int(42))]),
            Term::Extend {
                record: Box::new(Term::var("r")),
                label: "y".to_string(),
                value: Box::new(Term::Bool(true)),
            }
        );
        
        let ty = typecheck(&mut ctx, &term).unwrap();
        
        match ty {
            Type::Record(row) => {
                assert!(row.has_field("x"));
                assert!(row.has_field("y"));
            }
            _ => panic!("Expected record type"),
        }
    }
    
    #[test]
    fn test_row_subtyping() {
        let sub_row = RowType::from_fields(vec![
            ("x".to_string(), Type::Int),
            ("y".to_string(), Type::Bool),
        ]);
        
        let sup_row = RowType::from_fields(vec![
            ("x".to_string(), Type::Int),
        ]);
        
        // { x: Int, y: Bool } <: { x: Int }
        assert!(is_row_subtype(&sub_row, &sup_row));
        
        // But not the other way
        assert!(!is_row_subtype(&sup_row, &sub_row));
    }
} 