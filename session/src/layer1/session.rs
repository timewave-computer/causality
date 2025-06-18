// Layer 1 AST - Terms for typed message passing and sessions

use crate::layer1::types::{Type, SessionType};
use crate::layer1::linear::Variable;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

/// Layer 1 terms - typed expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Term {
    /// Variable reference
    Var(Variable),
    
    /// Unit value
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value  
    Int(i64),
    
    /// Create a pair
    Pair(Box<Term>, Box<Term>),
    
    /// Project first element of pair
    Fst(Box<Term>),
    
    /// Project second element of pair
    Snd(Box<Term>),
    
    /// Left injection into sum
    Inl(Box<Term>, Type), // Need target type for type checking
    
    /// Right injection into sum
    Inr(Box<Term>, Type), // Need target type for type checking
    
    /// Case analysis on sum
    Case {
        scrutinee: Box<Term>,
        left_var: Variable,
        left_body: Box<Term>,
        right_var: Variable,
        right_body: Box<Term>,
    },
    
    /// Create a record (which is a message) - deterministic field ordering
    Record(BTreeMap<String, Box<Term>>),
    
    /// Project a field from a record
    Project {
        record: Box<Term>,
        label: String,
    },
    
    /// Extend a record with a new field
    Extend {
        record: Box<Term>,
        label: String,
        value: Box<Term>,
    },
    
    /// Restrict fields from a record
    Restrict {
        record: Box<Term>,
        labels: Vec<String>,
    },
    
    /// Create a new session channel
    NewSession(SessionType),
    
    /// Send on a session
    Send {
        channel: Box<Term>,
        value: Box<Term>,
    },
    
    /// Receive from a session
    Receive(Box<Term>),
    
    /// Select a branch (internal choice)
    Select {
        channel: Box<Term>,
        label: String,
    },
    
    /// Offer branches (external choice)
    Offer {
        channel: Box<Term>,
        branches: Vec<(String, Variable, Box<Term>)>, // (label, var, body)
    },
    
    /// Let binding (for sequencing)
    Let {
        var: Variable,
        value: Box<Term>,
        body: Box<Term>,
    },
}

/// Value produced by evaluation (for testing/interpretation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    Pair(Box<Value>, Box<Value>),
    Inl(Box<Value>),
    Inr(Box<Value>),
    Record(BTreeMap<String, Box<Value>>), // Deterministic field ordering
    Session(crate::layer0::ChannelId),
}

impl Term {
    /// Helper to create a sequential composition
    pub fn seq(first: Term, second: Term) -> Term {
        // Use a dummy variable for unit-typed expressions
        Term::Let {
            var: Variable("_".to_string()),
            value: Box::new(first),
            body: Box::new(second),
        }
    }
    
    /// Helper to create a variable term
    pub fn var(name: &str) -> Term {
        Term::Var(Variable(name.to_string()))
    }
    
    /// Helper for pair construction
    pub fn pair(left: Term, right: Term) -> Term {
        Term::Pair(Box::new(left), Box::new(right))
    }
    
    /// Helper for record creation (message)
    pub fn record(fields: Vec<(&str, Term)>) -> Term {
        Term::Record(
            fields.into_iter()
                .map(|(label, term)| (label.to_string(), Box::new(term)))
                .collect()
        )
    }
    
    /// Helper for field projection
    pub fn project(record: Term, label: &str) -> Term {
        Term::Project {
            record: Box::new(record),
            label: label.to_string(),
        }
    }
    
    /// Helper for let binding
    pub fn let_bind(var: &str, value: Term, body: Term) -> Term {
        Term::Let {
            var: Variable(var.to_string()),
            value: Box::new(value),
            body: Box::new(body),
        }
    }
}

/// Simple interpreter for testing (not used in actual compilation)
#[cfg(test)]
mod interpreter {
    use super::*;
    use std::collections::HashMap;
    
    type Env = HashMap<Variable, Value>;
    
    pub fn eval(env: &Env, term: &Term) -> Result<Value, String> {
        match term {
            Term::Var(v) => env.get(v)
                .cloned()
                .ok_or_else(|| format!("Unbound variable: {:?}", v)),
                
            Term::Unit => Ok(Value::Unit),
            Term::Bool(b) => Ok(Value::Bool(*b)),
            Term::Int(n) => Ok(Value::Int(*n)),
            
            Term::Pair(t1, t2) => {
                let v1 = eval(env, t1)?;
                let v2 = eval(env, t2)?;
                Ok(Value::Pair(Box::new(v1), Box::new(v2)))
            }
            
            Term::Fst(t) => match eval(env, t)? {
                Value::Pair(v1, _) => Ok(*v1),
                _ => Err("Fst applied to non-pair".to_string()),
            },
            
            Term::Snd(t) => match eval(env, t)? {
                Value::Pair(_, v2) => Ok(*v2),
                _ => Err("Snd applied to non-pair".to_string()),
            },
            
            Term::Inl(t, _) => {
                let v = eval(env, t)?;
                Ok(Value::Inl(Box::new(v)))
            }
            
            Term::Inr(t, _) => {
                let v = eval(env, t)?;
                Ok(Value::Inr(Box::new(v)))
            }
            
            Term::Record(fields) => {
                let mut record = BTreeMap::new();
                for (label, term) in fields {
                    let value = eval(env, term)?;
                    record.insert(label.clone(), Box::new(value));
                }
                Ok(Value::Record(record))
            }
            
            Term::Project { record, label } => {
                match eval(env, record)? {
                    Value::Record(fields) => {
                        fields.get(label)
                            .cloned()
                            .map(|v| *v)
                            .ok_or_else(|| format!("Field {} not found", label))
                    }
                    _ => Err("Project applied to non-record".to_string()),
                }
            }
            
            Term::Let { var, value, body } => {
                let v = eval(env, value)?;
                let mut new_env = env.clone();
                new_env.insert(var.clone(), v);
                eval(&new_env, body)
            }
            
            _ => Err("Eval not implemented for this term".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::interpreter::eval;
    use std::collections::HashMap;
    
    #[test]
    fn test_basic_terms() {
        let env = HashMap::new();
        
        // Test unit
        assert_eq!(eval(&env, &Term::Unit).unwrap(), Value::Unit);
        
        // Test bool
        assert_eq!(eval(&env, &Term::Bool(true)).unwrap(), Value::Bool(true));
        
        // Test int
        assert_eq!(eval(&env, &Term::Int(42)).unwrap(), Value::Int(42));
    }
    
    #[test]
    fn test_pairs() {
        let env = HashMap::new();
        
        // Create pair
        let pair = Term::pair(Term::Int(1), Term::Bool(true));
        let result = eval(&env, &pair).unwrap();
        assert!(matches!(result, Value::Pair(_, _)));
        
        // Project first
        let fst = Term::Fst(Box::new(pair.clone()));
        assert_eq!(eval(&env, &fst).unwrap(), Value::Int(1));
        
        // Project second
        let snd = Term::Snd(Box::new(pair));
        assert_eq!(eval(&env, &snd).unwrap(), Value::Bool(true));
    }
    
    #[test]
    fn test_records() {
        let env = HashMap::new();
        
        // Create record
        let record = Term::record(vec![
            ("x", Term::Int(42)),
            ("y", Term::Bool(true)),
        ]);
        
        let result = eval(&env, &record).unwrap();
        assert!(matches!(result, Value::Record(_)));
        
        // Project field
        let proj_x = Term::project(record.clone(), "x");
        assert_eq!(eval(&env, &proj_x).unwrap(), Value::Int(42));
        
        let proj_y = Term::project(record, "y");
        assert_eq!(eval(&env, &proj_y).unwrap(), Value::Bool(true));
    }
    
    #[test]
    fn test_let_binding() {
        let env = HashMap::new();
        
        // let x = 5 in x + 1 (simulated as just x for now)
        let term = Term::let_bind(
            "x",
            Term::Int(5),
            Term::var("x")
        );
        
        assert_eq!(eval(&env, &term).unwrap(), Value::Int(5));
    }
}
