// TEL Abstract Syntax Tree Definitions 

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::types::TelType;

/// TEL program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Program name
    pub name: Option<String>,
    /// Program imports
    pub imports: Vec<Import>,
    /// Effect definitions
    pub effect_defs: HashMap<String, EffectDef>,
    /// Handler definitions
    pub handler_defs: Vec<HandlerDef>,
    /// Sequence flows
    pub flows: HashMap<String, Flow>,
    /// State definitions
    pub state_defs: HashMap<String, StateDef>,
    /// State machine
    pub state_machine: Option<StateMachine>,
    /// Program statements
    pub statements: Vec<Statement>,
}

/// Import declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
}

/// Effect definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TelType>,
    pub description: Option<String>,
}

impl EffectDef {
    /// Convert effect definition to metadata for TEG integration
    pub fn to_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        // Add basic metadata
        metadata.insert("name".to_string(), self.name.clone());
        
        // Add parameter information
        for (i, param) in self.params.iter().enumerate() {
            metadata.insert(
                format!("param_{}_name", i), 
                param.name.clone()
            );
            metadata.insert(
                format!("param_{}_type", i), 
                param.param_type.to_string()
            );
        }
        
        // Add return type if present
        if let Some(ret_type) = &self.return_type {
            metadata.insert("return_type".to_string(), ret_type.to_string());
        }
        
        // Add description if present
        if let Some(desc) = &self.description {
            metadata.insert("description".to_string(), desc.clone());
        }
        
        // Add parameter count
        metadata.insert("param_count".to_string(), self.params.len().to_string());
        
        metadata
    }
}

/// Handler definition for an effect
#[derive(Debug, Clone, PartialEq)]
pub struct HandlerDef {
    pub effect_name: String,
    pub params: Vec<Parameter>,
    pub resume_param: String, // Name of parameter that captures resumed value
    pub body: Vec<Statement>,
}

/// Parameter definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub param_type: TelType,
}

/// A flow definition representing a sequence of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TelType>,
    pub body: Vec<Statement>,
}

/// State definition for state machines
#[derive(Debug, Clone, PartialEq)]
pub struct StateDef {
    pub name: String,
    pub is_initial: bool,
    pub is_final: bool,
    pub fields: Vec<StateField>,
    pub transitions: Vec<Transition>,
}

/// Field in a state definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateField {
    pub name: String,
    pub field_type: TelType,
}

/// State transition definition
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub target_state: String,
    pub condition: Option<Expression>,
    pub action: Option<Vec<Statement>>,
}

/// A statement in TEL
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value_expr: Expression,
    },
    Perform {
        effect_name: String,
        args: Vec<Expression>,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Match {
        expr: Expression,
        arms: Vec<MatchArm>,
    },
    Return {
        expr: Option<Expression>,
    },
    Expression(Expression),
    StateTransition {
        target_state: String,
        args: Vec<Expression>,
    },
}

/// A match arm in a match statement
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Statement>,
}

/// A pattern used in match arms
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Literal(Literal),
    Variable(String),
    Wildcard,
}

/// An expression in TEL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Call {
        function: String,
        args: Vec<Expression>,
    },
    PerformExpr {
        effect_name: String,
        args: Vec<Expression>,
    },
    Pipeline {
        source: Box<Expression>,
        target: Box<Expression>,
    },
    Access {
        object: Box<Expression>,
        field: String,
    },
    StateExpr {
        state_name: String,
        fields: Vec<(String, Expression)>,
    },
    GetCurrentState,
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan, 
    GreaterThanOrEqual,
    And,
    Or,
    StringConcat,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Negate,
    Not,
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Map(HashMap<String, Literal>),
    List(Vec<Literal>),
    // ContentId handled separately via TelType
}

/// State machine definition
#[derive(Debug, Clone, PartialEq)]
pub struct StateMachine {
    pub initial_state: String,
    pub states: HashMap<String, StateDef>,
    pub transitions: Vec<Transition>,
}

impl Program {
    /// Get the main flow of the program, which is the entry point for execution
    pub fn get_main_flow(&self) -> Option<&Flow> {
        self.flows.get("main").or_else(|| self.flows.values().next())
    }
}

impl Expression {
    /// Returns true if the expression is a string literal
    pub fn is_string_literal(&self) -> bool {
        match self {
            Expression::Literal(Literal::String(_)) => true,
            _ => false,
        }
    }

    /// Returns true if the expression is a map literal
    pub fn is_map(&self) -> bool {
        match self {
            Expression::Literal(Literal::Map(_)) => true,
            _ => false,
        }
    }
}

// TODO: Add other necessary structs and functions for the AST 