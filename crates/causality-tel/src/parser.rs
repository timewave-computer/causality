// Parser implementation for Temporal Effect Language
// This file defines the lexer and parser for TEL syntax

use crate::ast::{self, Program, Import, Expression, Statement, Literal};
use anyhow::{Result, anyhow};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1, take_until},
    character::complete::{alpha1, alphanumeric1, char, digit1, space0, space1, multispace0, multispace1, line_ending, not_line_ending, one_of},
    combinator::{map, map_res, opt, recognize, value, eof},
    multi::{many0, many1, separated_list0, fold_many0},
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use std::collections::HashMap;
use thiserror::Error;

/// Parser error type
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Syntax error: {0}")]
    Syntax(String),
    
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    
    #[error("Invalid character: {0}")]
    InvalidCharacter(char),
}

/// Result type for parser operations
pub type ParserResult<T> = Result<T, ParserError>;

/// Parse a TEL source file into an AST Program
pub fn parse_program(source: &str) -> ParserResult<Program> {
    match program_parser(source) {
        Ok((_, program)) => Ok(program),
        Err(e) => Err(ParserError::Syntax(format!("Failed to parse program: {:?}", e))),
    }
}

/// Parser for a complete program
fn program_parser(input: &str) -> IResult<&str, Program> {
    let (input, _) = space0(input)?;
    let (input, statements) = many0(preceded(parse_comment_or_whitespace, parse_statement))(input)?;
    
    // Return the parsed program
    Ok((input, Program {
        name: None,
        imports: Vec::new(),
        effect_defs: HashMap::new(),
        handler_defs: Vec::new(),
        flows: HashMap::new(),
        state_defs: HashMap::new(),
        state_machine: None,
        statements,
    }))
}

/// Parser for statements (let bindings, expressions, etc)
fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_let_binding,
        map(parse_expression, |expr| Statement::Expression(expr)),
    ))(input)
}

/// Parser for let bindings: "let name = value"
fn parse_let_binding(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("let")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = parse_expression(input)?;
    
    Ok((input, Statement::Let { 
        name, 
        value_expr: expr 
    }))
}

/// Parser for expressions
fn parse_expression(input: &str) -> IResult<&str, Expression> {
    // Start with the binary operations, which have lower precedence
    parse_binary_operation(input)
}

/// Parser for lambda expressions: "\param1 param2 -> body"
fn parse_lambda(input: &str) -> IResult<&str, Expression> {
    let (input, _) = char('\\')(input)?;
    let (input, _) = space0(input)?;
    let (input, params) = separated_list0(
        delimited(space0, char(','), space0),
        parse_identifier
    )(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("->")(input)?;
    let (input, _) = space0(input)?;
    let (input, body) = parse_expression(input)?;
    
    // Create a function call with the params and body since AST doesn't have direct Lambda
    let func_name = "lambda".to_string();
    let mut args = Vec::new();
    args.push(body);
    
    Ok((input, Expression::Call {
        function: func_name,
        args
    }))
}

/// Parser for if-then-else expressions
fn parse_if_expression(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tag("if")(input)?;
    let (input, _) = space1(input)?;
    let (input, condition) = parse_expression(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("then")(input)?;
    let (input, _) = space1(input)?;
    let (input, then_expr) = parse_expression(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("else")(input)?;
    let (input, _) = space1(input)?;
    let (input, else_expr) = parse_expression(input)?;
    
    // Create a call to an "if" function since AST doesn't have direct If
    let func_name = "if".to_string();
    let args = vec![condition, then_expr, else_expr];
    
    Ok((input, Expression::Call {
        function: func_name,
        args
    }))
}

/// Parser for record expressions: { field1: value1, field2: value2 }
fn parse_record_expression(input: &str) -> IResult<&str, Expression> {
    let (input, _) = char('{')(input)?;
    let (input, _) = space0(input)?;
    let (input, fields) = separated_list0(
        delimited(space0, char(','), space0),
        parse_record_field
    )(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('}')(input)?;
    
    // Convert to a map
    let mut field_map = HashMap::new();
    for (key, value) in fields {
        field_map.insert(key, value);
    }
    
    Ok((input, Expression::Literal(Literal::Map(field_map))))
}

/// Parser for record fields: field_name: expression
fn parse_record_field(input: &str) -> IResult<&str, (String, Literal)> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = parse_expression(input)?;
    
    // Convert the expression to a literal
    let literal = match expr {
        Expression::Literal(lit) => lit,
        Expression::Variable(name) => Literal::String(name),
        // Handle other types by converting to a string representation
        _ => Literal::String("<expr>".to_string())
    };
    
    Ok((input, (name, literal)))
}

/// Parser for binary operations
fn parse_binary_operation(input: &str) -> IResult<&str, Expression> {
    // Parse the first operand and operator
    let (input, first) = parse_application_expression(input)?;
    
    // Create a clone of first that will live for the entire function scope
    let first_clone = first.clone();
    
    // Parse the remaining operators and operands, if any
    fold_many0(
        pair(
            delimited(space0, parse_binary_operator, space0),
            parse_application_expression
        ),
        move || first_clone.clone(),
        |left, (op, right)| {
            Expression::BinaryOp { 
                op,
                left: Box::new(left),
                right: Box::new(right)
            }
        }
    )(input)
}

/// Parser for binary operators
fn parse_binary_operator(input: &str) -> IResult<&str, ast::BinaryOperator> {
    alt((
        value(ast::BinaryOperator::Add, tag("+")),
        value(ast::BinaryOperator::Subtract, tag("-")),
        value(ast::BinaryOperator::Multiply, tag("*")),
        value(ast::BinaryOperator::Divide, tag("/")),
        value(ast::BinaryOperator::Modulo, tag("%")),
        value(ast::BinaryOperator::Equal, tag("==")),
        value(ast::BinaryOperator::NotEqual, tag("!=")),
        value(ast::BinaryOperator::LessThan, tag("<")),
        value(ast::BinaryOperator::LessThanOrEqual, tag("<=")),
        value(ast::BinaryOperator::GreaterThan, tag(">")),
        value(ast::BinaryOperator::GreaterThanOrEqual, tag(">=")),
        value(ast::BinaryOperator::And, tag("&&")),
        value(ast::BinaryOperator::Or, tag("||")),
        value(ast::BinaryOperator::StringConcat, tag("<>")),
    ))(input)
}

/// Parser for function application expressions
fn parse_application_expression(input: &str) -> IResult<&str, Expression> {
    // Parse the function
    let (input, func) = parse_primary_expression(input)?;
    
    // Create a clone of func that will live for the entire function scope
    let func_clone = func.clone();
    
    // Parse the arguments, if any
    fold_many0(
        preceded(space1, parse_primary_expression),
        move || func_clone.clone(),
        |f, arg| {
            Expression::Call { 
                function: match &f {
                    Expression::Variable(name) => name.clone(),
                    _ => "<anonymous>".to_string()
                },
                args: vec![arg]
            }
        }
    )(input)
}

/// Parser for primary expressions (literals, identifiers, etc)
fn parse_primary_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        map(parse_literal, Expression::Literal),
        map(parse_identifier, Expression::Variable),
        delimited(
            pair(char('('), space0),
            parse_expression,
            pair(space0, char(')'))
        ),
        parse_if_expression,
        parse_lambda,
        parse_record_expression,
    ))(input)
}

/// Parser for literals
fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((
        map(parse_string_literal, Literal::String),
        map(parse_float_literal, Literal::Float),
        map(parse_int_literal, Literal::Int),
        map(parse_bool_literal, Literal::Bool),
        parse_null_literal,
        parse_map_literal,
        parse_list_literal,
    ))(input)
}

/// Parser for null literals
fn parse_null_literal(input: &str) -> IResult<&str, Literal> {
    let (input, _) = tag("null")(input)?;
    Ok((input, Literal::Null))
}

/// Parser for map literals (e.g., {"key": value})
fn parse_map_literal(input: &str) -> IResult<&str, Literal> {
    let (input, _) = char('{')(input)?;
    let (input, _) = space0(input)?;
    
    let (input, entries) = separated_list0(
        delimited(space0, char(','), space0),
        pair(
            delimited(
                space0,
                delimited(char('"'), take_while1(|c| c != '"'), char('"')),
                space0
            ),
            preceded(
                delimited(space0, char(':'), space0),
                parse_literal
            )
        )
    )(input)?;
    
    let (input, _) = space0(input)?;
    let (input, _) = char('}')(input)?;
    
    let map = entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    
    Ok((input, Literal::Map(map)))
}

/// Parser for list literals (e.g., [1, 2, 3])
fn parse_list_literal(input: &str) -> IResult<&str, Literal> {
    let (input, _) = char('[')(input)?;
    let (input, _) = space0(input)?;
    
    let (input, items) = separated_list0(
        delimited(space0, char(','), space0),
        parse_literal
    )(input)?;
    
    let (input, _) = space0(input)?;
    let (input, _) = char(']')(input)?;
    
    Ok((input, Literal::List(items)))
}

/// Parser for string literals
fn parse_string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            many0(alt((
                map(
                    take_while1(|c: char| c != '"' && c != '\\'),
                    |s: &str| s.to_string()
                ),
                map(pair(char('\\'), char('n')), |_| "\n".to_string()),
                map(pair(char('\\'), char('t')), |_| "\t".to_string()),
                map(pair(char('\\'), char('r')), |_| "\r".to_string()),
                map(pair(char('\\'), char('"')), |_| "\"".to_string()),
                map(pair(char('\\'), char('\\')), |_| "\\".to_string()),
            ))),
            |parts| parts.join("")
        ),
        char('"')
    )(input)
}

/// Parser for integer literals
fn parse_int_literal(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize(pair(
            opt(char('-')),
            digit1
        )),
        |s: &str| s.parse::<i64>()
    )(input)
}

/// Parser for floating-point literals
fn parse_float_literal(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(
            tuple((
                opt(char('-')),
                digit1,
                char('.'),
                digit1,
                opt(pair(
                    one_of("eE"),
                    pair(
                        opt(one_of("+-")),
                        digit1
                    )
                ))
            ))
        ),
        |s: &str| s.parse::<f64>()
    )(input)
}

/// Parser for boolean literals
fn parse_bool_literal(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("true")),
        value(false, tag("false"))
    ))(input)
}

/// Parser for identifiers
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"))))
        )),
        |s: &str| s.to_string()
    )(input)
}

/// Parser for comments and whitespace
fn parse_comment_or_whitespace(input: &str) -> IResult<&str, ()> {
    map(
        many0(alt((
            // Line comment (PureScript style)
            map(tuple((tag("--"), not_line_ending, line_ending)), |_| ()),
            map(tuple((tag("--"), not_line_ending, eof)), |_| ()),
            // Line comment (C style) - for backward compatibility
            map(tuple((tag("//"), not_line_ending, line_ending)), |_| ()),
            map(tuple((tag("//"), not_line_ending, eof)), |_| ()),
            // Block comment
            map(tuple((tag("/*"), take_until("*/"), tag("*/"))), |_| ()),
            // Whitespace
            map(multispace1, |_| ())
        ))),
        |_| ()
    )(input)
}

/// Helper function to parse a character in a given range
fn char_in_range(start: char, end: char) -> impl Fn(&str) -> IResult<&str, char> {
    let range_string = (start..=end).collect::<String>();
    
    move |input: &str| {
        one_of(range_string.as_str())(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comment() {
        let (_, _) = parse_comment_or_whitespace("-- This is a comment\n").unwrap();
        let (_, _) = parse_comment_or_whitespace("// This is a C-style comment\n").unwrap();
        let (_, _) = parse_comment_or_whitespace("/* This is a block comment */").unwrap();
    }

    #[test]
    fn test_parse_identifier() {
        let (_, id) = parse_identifier("myVar123").unwrap();
        assert_eq!(id, "myVar123");
    }

    #[test]
    fn test_parse_string_literal() {
        let (_, s) = parse_string_literal("\"hello world\"").unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_parse_int_literal() {
        let (_, n) = parse_int_literal("42").unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_parse_float_literal() {
        let (_, f) = parse_float_literal("3.14").unwrap();
        assert_eq!(f, 3.14);
    }

    #[test]
    fn test_parse_let_binding() {
        let (_, stmt) = parse_let_binding("let x = 42").unwrap();
        match stmt {
            Statement::Let { name, value_expr } => {
                assert_eq!(name, "x");
                match value_expr {
                    Expression::Literal(Literal::Int(val)) => assert_eq!(val, 42),
                    _ => panic!("Expected int literal")
                }
            },
            _ => panic!("Expected let statement")
        }
    }

    #[test]
    fn test_parse_simple_program() {
        let program = parse_program("let x = 42\nlet y = \"hello\"").unwrap();
        assert_eq!(program.statements.len(), 2);
    }
} 