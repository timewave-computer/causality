// State machine parser for TEL

use crate::ast::{StateMachine, StateDef, StateField, Transition, Expression};
use crate::types::TelType;
use crate::parser::expression::parse_expression;
use crate::parser::types::parse_type;
use crate::parser::common::{ws, identifier, string_literal};

use nom::{
    IResult,
    branch::alt,
    combinator::{map, opt, value},
    sequence::{preceded, tuple, terminated, pair},
    multi::{many0, separated_list0, separated_list1},
    character::complete::{char, space1},
    bytes::complete::{tag},
};

use std::collections::HashMap;

/// Parse a state machine definition
/// ```tel
/// state
///   initial Pending
///   Approved
///   Swapping
///   final Completed
///   final Failed reason:String
/// ```
pub fn parse_state_machine(input: &str) -> IResult<&str, StateMachine> {
    let (input, _) = tag("state")(input)?;
    let (input, _) = ws(input)?;
    
    let (input, states_with_modifiers) = many0(parse_state_def)(input)?;
    
    // Process the states, identify the initial state
    let mut states = HashMap::new();
    let mut initial_state = None;
    let mut transitions = Vec::new();
    
    for (name, state_def, is_initial) in states_with_modifiers {
        if is_initial {
            initial_state = Some(name.clone());
        }
        
        // Store transitions to be added later
        transitions.extend(state_def.transitions.clone());
        
        // Add the state to the map
        states.insert(name, state_def);
    }
    
    // Default to first state as initial if none specified
    let initial_state = initial_state.unwrap_or_else(|| {
        states.keys().next().cloned().unwrap_or_default()
    });
    
    Ok((input, StateMachine {
        initial_state,
        states,
        transitions,
    }))
}

/// Parse a state definition
/// Returns (state_name, state_def, is_initial)
fn parse_state_def(input: &str) -> IResult<&str, (String, StateDef, bool)> {
    let (input, is_initial) = opt(ws(tag("initial")))(input)?;
    let is_initial = is_initial.is_some();
    
    let (input, is_final) = opt(ws(tag("final")))(input)?;
    let is_final = is_final.is_some();
    
    let (input, name) = ws(identifier)(input)?;
    
    // Parse optional fields
    let (input, fields) = many0(parse_state_field)(input)?;
    
    // Parse transitions (if any)
    let (input, transitions) = many0(parse_transition)(input)?;
    
    let state_def = StateDef {
        name: name.to_string(),
        is_initial,
        is_final,
        fields,
        transitions,
    };
    
    Ok((input, (name.to_string(), state_def, is_initial)))
}

/// Parse a state field definition like `reason:String`
fn parse_state_field(input: &str) -> IResult<&str, StateField> {
    let (input, field_name) = ws(identifier)(input)?;
    let (input, _) = ws(char(':'))(input)?;
    let (input, field_type) = ws(parse_type)(input)?;
    
    Ok((input, StateField {
        name: field_name.to_string(),
        field_type,
    }))
}

/// Parse a state transition definition
fn parse_transition(input: &str) -> IResult<&str, Transition> {
    let (input, _) = ws(tag("-"))(input)?;
    let (input, _) = ws(tag(">"))(input)?;
    let (input, target_state) = ws(identifier)(input)?;
    
    // Optional condition
    let (input, condition) = opt(preceded(
        ws(tag("if")),
        parse_expression
    ))(input)?;
    
    // TODO: Parse optional actions
    
    Ok((input, Transition {
        target_state: target_state.to_string(),
        condition,
        action: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_state_field() {
        let input = "reason:String";
        let (rest, field) = parse_state_field(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(field.name, "reason");
        assert_eq!(field.field_type, TelType::String);
    }
    
    #[test]
    fn test_parse_state_def() {
        let input = "final Failed reason:String";
        let (rest, (name, state_def, is_initial)) = parse_state_def(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(name, "Failed");
        assert_eq!(state_def.is_final, true);
        assert_eq!(state_def.is_initial, false);
        assert_eq!(is_initial, false);
        assert_eq!(state_def.fields.len(), 1);
        assert_eq!(state_def.fields[0].name, "reason");
        assert_eq!(state_def.fields[0].field_type, TelType::String);
    }
    
    #[test]
    fn test_parse_simple_state_machine() {
        let input = "state\n  initial Pending\n  Approved\n  final Completed";
        let (rest, state_machine) = parse_state_machine(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(state_machine.initial_state, "Pending");
        assert_eq!(state_machine.states.len(), 3);
        assert!(state_machine.states.contains_key("Pending"));
        assert!(state_machine.states.contains_key("Approved"));
        assert!(state_machine.states.contains_key("Completed"));
        assert!(state_machine.states.get("Pending").unwrap().is_initial);
        assert!(state_machine.states.get("Completed").unwrap().is_final);
    }
} 