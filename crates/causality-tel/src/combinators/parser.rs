//! Parser for the TEL Combinator Language
//!
//! This module provides parsing functionality for the TEL combinator language,
//! supporting both point-free notation and applicative forms.

use std::collections::HashMap;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1, space0, space1, anychar as nom_anychar},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use super::{Combinator, Literal};

/// Parse a complete combinator expression and ensure no trailing input
pub fn parse_combinator(input: &str) -> Result<Combinator, String> {
    let trimmed = input.trim();
    match parse_expr(trimmed) {
        Ok((remaining, expr)) => {
            if remaining.trim().is_empty() {
                Ok(expr)
            } else {
                Err(format!("Unexpected trailing input: '{}'", remaining))
            }
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Parse an expression, which is a composition of terms with pipeline operator
fn parse_expr(input: &str) -> IResult<&str, Combinator> {
    let (input, first_term) = parse_term(input)?;
    let (input, rest) = many0(
        preceded(
            delimited(space0, tag("|>"), space0),
            parse_term
        )
    )(input)?;
    
    let result = rest.into_iter().fold(first_term, |acc, term| {
        Combinator::app(term, acc)
    });
    
    Ok((input, result))
}

/// Parse a term, which could be an application or an atom
fn parse_term(input: &str) -> IResult<&str, Combinator> {
    let (input, first_atom) = parse_atom(input)?;
    let (input, rest) = many0(
        preceded(
            space1,
            parse_atom
        )
    )(input)?;
    
    let result = rest.into_iter().fold(first_atom, |acc, arg| {
        Combinator::app(acc, arg)
    });
    
    Ok((input, result))
}

/// Parse an atomic expression, which could be a combinator name, literal, etc.
fn parse_atom(input: &str) -> IResult<&str, Combinator> {
    alt((
        parse_core_combinator,
        parse_effect,
        parse_state_transition,
        parse_content_id,
        parse_store,
        parse_load,
        parse_parenthesized_expr,
        parse_literal,
        parse_reference,
    ))(input)
}

/// Parse a core combinator (I, K, S, B, C)
fn parse_core_combinator(input: &str) -> IResult<&str, Combinator> {
    alt((
        value(Combinator::I, tag("I")),
        value(Combinator::K, tag("K")),
        value(Combinator::S, tag("S")),
        value(Combinator::B, tag("B")),
        value(Combinator::C, tag("C")),
    ))(input)
}

/// Parse an effect application
fn parse_effect(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = tag("effect")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, args) = delimited(
        pair(space0, char('(')),
        separated_list0(
            delimited(space0, char(','), space0),
            parse_expr
        ),
        pair(space0, char(')'))
    )(input)?;
    
    Ok((input, Combinator::effect(name, args)))
}

/// Parse a state transition
pub fn parse_state_transition(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = tag("transition")(input)?;
    let (input, _) = space0(input)?;
    let (input, state) = parse_identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, fields) = delimited(
        char('{'),
        field_assignments,
        char('}')
    )(input)?;
    
    Ok((input, Combinator::StateTransition { target_state: state.to_string(), fields, resource_id: None }))
}

/// Parse a content ID expression
fn parse_content_id(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = tag("content_id")(input)?;
    let (input, expr) = delimited(
        pair(space0, char('(')),
        parse_expr,
        pair(space0, char(')'))
    )(input)?;
    
    Ok((input, Combinator::ContentId(Box::new(expr))))
}

/// Parse a store expression
fn parse_store(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = tag("store")(input)?;
    let (input, expr) = delimited(
        pair(space0, char('(')),
        parse_expr,
        pair(space0, char(')'))
    )(input)?;
    
    Ok((input, Combinator::Store(Box::new(expr))))
}

/// Parse a load expression
fn parse_load(input: &str) -> IResult<&str, Combinator> {
    let (input, _) = tag("load")(input)?;
    let (input, expr) = delimited(
        pair(space0, char('(')),
        parse_expr,
        pair(space0, char(')'))
    )(input)?;
    
    Ok((input, Combinator::Load(Box::new(expr))))
}

/// Parse a parenthesized expression
fn parse_parenthesized_expr(input: &str) -> IResult<&str, Combinator> {
    delimited(
        pair(char('('), space0),
        parse_expr,
        pair(space0, char(')'))
    )(input)
}

/// Parse a literal value
fn parse_literal(input: &str) -> IResult<&str, Combinator> {
    alt((
        map(parse_int_literal, |n| Combinator::Literal(Literal::Int(n))),
        map(parse_float_literal, |f| Combinator::Literal(Literal::Float(f))),
        map(parse_string_literal, |s| Combinator::Literal(Literal::String(s))),
        map(parse_bool_literal, |b| Combinator::Literal(Literal::Bool(b))),
        map(tag("null"), |_| Combinator::Literal(Literal::Null)),
    ))(input)
}

/// Parse an integer literal
fn parse_int_literal(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize(pair(
            opt(char('-')),
            digit1
        )),
        |s: &str| s.parse::<i64>()
    )(input)
}

/// Parse a floating-point literal
fn parse_float_literal(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            char('.'),
            digit1
        ))),
        |s: &str| s.parse::<f64>()
    )(input)
}

/// Parse a string literal
fn parse_string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            recognize(many0(alt((
                recognize(take_while1(|c: char| c != '"' && c != '\\')),
                recognize(pair(char('\\'), nom_anychar)),
            )))),
            |s: &str| s.to_string()
        ),
        char('"')
    )(input)
}

/// Parse a boolean literal
fn parse_bool_literal(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("true")),
        value(false, tag("false")),
    ))(input)
}

/// Parse a reference to a named combinator
fn parse_reference(input: &str) -> IResult<&str, Combinator> {
    map(
        parse_identifier,
        |id: &str| Combinator::Ref(id.to_string())
    )(input)
}

/// Parse an identifier
fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"))))
    ))(input)
}

/// Parse a set of field assignments
fn field_assignments(input: &str) -> IResult<&str, HashMap<String, Combinator>> {
    map(
        separated_list0(
            delimited(space0, char(','), space0),
            separated_pair(
                parse_identifier,
                delimited(space0, char(':'), space0),
                parse_expr
            )
        ),
        |pairs| {
            let mut map = HashMap::new();
            for (key, value) in pairs {
                map.insert(key.to_string(), value);
            }
            map
        }
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_core_combinators() {
        assert_eq!(parse_combinator("I"), Ok(Combinator::I));
        assert_eq!(parse_combinator("K"), Ok(Combinator::K));
        assert_eq!(parse_combinator("S"), Ok(Combinator::S));
        assert_eq!(parse_combinator("B"), Ok(Combinator::B));
        assert_eq!(parse_combinator("C"), Ok(Combinator::C));
    }

    #[test]
    fn test_parse_application() {
        // I x
        let expected = Combinator::app(Combinator::I, Combinator::Ref("x".to_string()));
        assert_eq!(parse_combinator("I x"), Ok(expected));
        
        // K x y
        let expected = Combinator::app(
            Combinator::app(Combinator::K, Combinator::Ref("x".to_string())),
            Combinator::Ref("y".to_string())
        );
        assert_eq!(parse_combinator("K x y"), Ok(expected));
    }

    #[test]
    fn test_parse_effect() {
        // effect log("hello")
        let expected = Combinator::effect(
            "log", 
            vec![Combinator::Literal(Literal::String("hello".to_string()))]
        );
        assert_eq!(parse_combinator("effect log(\"hello\")"), Ok(expected));
    }

    #[test]
    fn test_parse_state_transition() {
        // transition Account{ balance: 100 }
        let mut fields = HashMap::new();
        fields.insert("balance".to_string(), Combinator::Literal(Literal::Int(100)));
        
        let expected = Combinator::StateTransition { 
            target_state: "Account".to_string(), 
            fields,
            resource_id: None
        };
        assert_eq!(parse_combinator("transition Account {balance: 100}"), Ok(expected));
    }

    #[test]
    fn test_parse_pipeline() {
        // x |> f |> g
        let x = Combinator::Ref("x".to_string());
        let f = Combinator::Ref("f".to_string());
        let g = Combinator::Ref("g".to_string());
        
        let expected = Combinator::app(
            g,
            Combinator::app(f, x)
        );
        assert_eq!(parse_combinator("x |> f |> g"), Ok(expected));
    }
} 