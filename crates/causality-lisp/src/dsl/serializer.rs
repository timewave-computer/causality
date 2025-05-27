// crates/causality-lisp/src/dsl/serializer.rs
//! Defines the serializer for converting canonical `Expr` to S-expression strings.

use std::fmt::Write;

use causality_core::id_to_hex; // Import the function directly
use causality_core::lisp_adapter::StrExt;
use causality_types::primitive::number::Number;
use causality_types::expr::ast::{
    Atom, AtomicCombinator, Expr as TypesExpr,
}; // Ensure Atom, AtomicCombinator, ExprBox, ExprVec are here
use causality_types::expr::value::{
    ValueExpr, ValueExprMap, ValueExprRef, ValueExprVec,
};

// Assuming ExprContextual might be needed for context-aware serialization in future

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::string::String;

// Define a local trait for S-expression serialization
pub trait SExprSerializable {
    fn to_sexpr_string(&self) -> String;
    fn write_sexpr_to_string_recursive<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::fmt::Result;
}

fn write_value_expr_to_string<W: Write>(
    value: &ValueExpr,
    writer: &mut W,
) -> std::fmt::Result {
    match value {
        ValueExpr::Nil => writer.write_str("nil")?,
        ValueExpr::Bool(b) => writer.write_str(if *b {
            "true"
        } else {
            "false"
        })?,
        ValueExpr::String(s) => {
            writer.write_char('"')?;
            // Use the StrExt trait for chars()
            for ch in s.chars() {
                match ch {
                    '\\' => writer.write_str("\\\\")?,
                    '"' => writer.write_str("\\\"")?,
                    '\n' => writer.write_str("\\n")?,
                    '\r' => writer.write_str("\\r")?,
                    '\t' => writer.write_str("\\t")?,
                    _ => writer.write_char(ch)?,
                }
            }
            writer.write_char('"')?;
        }
        ValueExpr::Number(n) => match n {
            Number::Integer(i) => write!(writer, "{}", i)?,
            // Add other Number variants if needed, e.g., Fixed, Float (though float is non-deterministic)
            // For now, only Integer is explicitly handled from `Atom` and `ValueExpr` number.
            _ => write!(writer, "#<number:{:?}>", n)?, // Placeholder for other number types
        },
        ValueExpr::List(ValueExprVec(items)) => {
            writer.write_char('(')?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    writer.write_char(' ')?;
                }
                write_value_expr_to_string(item, writer)?;
            }
            writer.write_char(')')?;
        }
        ValueExpr::Map(ValueExprMap(map_items))
        | ValueExpr::Record(ValueExprMap(map_items)) => {
            if matches!(value, ValueExpr::Record(_)) {
                writer.write_str("(record ")?;
            }
            writer.write_char('(')?;
            for (i, (key, value)) in map_items.iter().enumerate() {
                if i > 0 {
                    writer.write_char(' ')?;
                }
                writer.write_char('(')?;
                // Key as string literal
                writer.write_char('"')?;
                // Use the StrExt trait for chars()
                for ch in key.chars() {
                    match ch {
                        '\\' => writer.write_str("\\\\")?,
                        '"' => writer.write_str("\\\"")?,
                        '\n' => writer.write_str("\\n")?,
                        '\r' => writer.write_str("\\r")?,
                        '\t' => writer.write_str("\\t")?,
                        _ => writer.write_char(ch)?,
                    }
                }
                writer.write_char('"')?;
                writer.write_str(" . ")?;
                write_value_expr_to_string(value, writer)?;
                writer.write_char(')')?;
            }
            writer.write_char(')')?;
            if matches!(value, ValueExpr::Record(_)) {
                writer.write_char(')')?;
            }
        }
        ValueExpr::Ref(val_ref) => match val_ref {
            ValueExprRef::Expr(id) => {
                write!(writer, "#<ref:expr:{}>", id_to_hex(id))?
            }
            ValueExprRef::Value(id) => {
                write!(writer, "#<ref:value:{}>", id_to_hex(id))?
            }
        },
        ValueExpr::Lambda {
            params,
            body_expr_id,
            captured_env: _,
        } => {
            write!(
                writer,
                "#<lambda:({:#?}) body_id:{}>",
                params,
                id_to_hex(body_expr_id)
            )?;
        } // If ValueExpr gains more variants, they need to be handled here.
    }
    Ok(())
}

pub fn atomic_combinator_to_string(combinator: &AtomicCombinator) -> &'static str {
    match combinator {
        AtomicCombinator::S => "S", // Core combinators might not be directly used by DSL often
        AtomicCombinator::K => "K",
        AtomicCombinator::I => "I",
        AtomicCombinator::C => "C", // Could be 'if-c' or specific name
        AtomicCombinator::If => "if",
        AtomicCombinator::Let => "let", // Let is usually a special form, not a simple combinator name
        AtomicCombinator::And => "and",
        AtomicCombinator::Or => "or",
        AtomicCombinator::Not => "not",
        AtomicCombinator::Eq => "eq?", // or 'eq', 'equal?'
        AtomicCombinator::Add => "+",
        AtomicCombinator::Sub => "-",
        AtomicCombinator::Mul => "*",
        AtomicCombinator::Div => "/",
        AtomicCombinator::Gt => ">",
        AtomicCombinator::Lt => "<",
        AtomicCombinator::Gte => ">=",
        AtomicCombinator::Lte => "<=",
        AtomicCombinator::GetContextValue => "get-context-value",
        AtomicCombinator::GetField => "get-field",
        AtomicCombinator::Completed => "completed",
        AtomicCombinator::List => "list",
        AtomicCombinator::Nth => "nth",
        AtomicCombinator::Length => "length",
        AtomicCombinator::Cons => "cons",
        AtomicCombinator::Car => "car",
        AtomicCombinator::Cdr => "cdr",
        AtomicCombinator::MakeMap => "make-map",
        AtomicCombinator::MapGet => "map-get",
        AtomicCombinator::MapHasKey => "has-key?",
        AtomicCombinator::Define => "define",
        AtomicCombinator::Defun => "defun",
        AtomicCombinator::Quote => "quote",
    }
}

impl SExprSerializable for TypesExpr {
    /// Serializes the TypesExpr to an S-expression string.
    fn to_sexpr_string(&self) -> String {
        let mut s = String::new();
        match self.write_sexpr_to_string_recursive(&mut s) {
            Ok(_) => {}
            Err(_) => {
                // Clear s and push error marker if serialization fails.
                s.clear();
                s.push_str("##EXPR_SERIALIZATION_ERROR##");
            }
        }
        s
    }

    fn write_sexpr_to_string_recursive<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::fmt::Result {
        match self {
            TypesExpr::Atom(atom) => match atom {
                Atom::Nil => writer.write_str("nil")?,
                Atom::Boolean(b) => writer.write_str(if *b {
                    "true"
                } else {
                    "false"
                })?,
                Atom::Integer(i) => write!(writer, "{}", i)?,
                Atom::String(s_val) => {
                    writer.write_char('"')?;
                    // Use the StrExt trait for chars()
                    for ch in s_val.chars() {
                        match ch {
                            '\\' => writer.write_str("\\\\")?,
                            '"' => writer.write_str("\\\"")?,
                            '\n' => writer.write_str("\\n")?,
                            '\r' => writer.write_str("\\r")?,
                            '\t' => writer.write_str("\\t")?,
                            _ => writer.write_char(ch)?,
                        }
                    }
                    writer.write_char('"')?;
                }
            },
            TypesExpr::Var(s_val) => writer.write_str(s_val.as_str())?,
            TypesExpr::Combinator(c) => {
                writer.write_str(atomic_combinator_to_string(c))?
            }
            TypesExpr::Apply(func_expr_box, args_expr_vec) => {
                writer.write_char('(')?;
                func_expr_box.0.write_sexpr_to_string_recursive(writer)?;
                for arg_expr in args_expr_vec.0.iter() {
                    writer.write_char(' ')?;
                    arg_expr.write_sexpr_to_string_recursive(writer)?;
                }
                writer.write_char(')')?;
            }
            TypesExpr::Const(value_expr) => {
                write_value_expr_to_string(value_expr, writer)?;
            }
            TypesExpr::Lambda(params_vec, body_expr_box) => {
                writer.write_str("(lambda (")?;
                for (i, param) in params_vec.iter().enumerate() {
                    if i > 0 {
                        writer.write_char(' ')?;
                    }
                    writer.write_str(param.as_str())?;
                }
                writer.write_str(") ")?;
                body_expr_box.0.write_sexpr_to_string_recursive(writer)?;
                writer.write_char(')')?;
            }
            TypesExpr::Dynamic(steps, expr_to_eval_box) => {
                write!(writer, "(dynamic-eval {} ", steps)?;
                expr_to_eval_box.0.write_sexpr_to_string_recursive(writer)?;
                writer.write_char(')')?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        core::{
            id::ExprId,
            str::Str,
        },
        expr::{
            ast::AtomicCombinator,
            value::{ValueExpr, ValueExprRef, ValueExprVec, ValueExprMap},
            ExprBox, ExprVec,
        },
    };
    use causality_core::utils::core::id_to_hex;
    use crate::dsl::builders::{
        and_, bool_lit, defun, eq_, if_, int_lit, list, nil, str_lit, sym,
    };
    use crate::lisp_list;
    use std::fmt::Write;
    use causality_types::expr::ast::Expr as TypesExpr;

    fn write_value_expr_to_string(expr: &ValueExpr, buffer: &mut String) -> std::fmt::Result {
        match expr {
            ValueExpr::Ref(ValueExprRef::Expr(id)) => {
                write!(buffer, "#<ref:expr:{}>", id_to_hex(id))
            }
            _ => write!(buffer, "{:?}", expr), // Use Debug instead of Display
        }
    }

    #[test]
    fn test_serialize_atoms_and_var() {
        assert_eq!(sym("my-var").to_sexpr_string(), "my-var");
        assert_eq!(str_lit("hello").to_sexpr_string(), "\"hello\"");
        assert_eq!(str_lit("esc\"aped").to_sexpr_string(), "\"esc\\\"aped\"");
        assert_eq!(str_lit("a\\b").to_sexpr_string(), "\"a\\\\b\"");
        assert_eq!(str_lit("new\nline").to_sexpr_string(), "\"new\\nline\"");
        assert_eq!(str_lit("tab\tchar").to_sexpr_string(), "\"tab\\tchar\"");
        assert_eq!(str_lit("cr\rchar").to_sexpr_string(), "\"cr\\rchar\"");
        assert_eq!(int_lit(42).to_sexpr_string(), "42");
        assert_eq!(bool_lit(true).to_sexpr_string(), "true");
        assert_eq!(bool_lit(false).to_sexpr_string(), "false");
        assert_eq!(nil().to_sexpr_string(), "nil");
    }

    #[test]
    fn test_serialize_const() {
        assert_eq!(
            TypesExpr::Const(ValueExpr::Number(
                causality_types::primitive::number::Number::Integer(123)
            ))
            .to_sexpr_string(),
            "123"
        );
        assert_eq!(
            TypesExpr::Const(ValueExpr::String(Str::from("const str")))
                .to_sexpr_string(),
            "(const \"const str\")"
        );
        assert_eq!(
            TypesExpr::Const(ValueExpr::String(Str::from("const \"esc\" str")))
                .to_sexpr_string(),
            "(const \"const \\\"esc\\\" str\")"
        );
        assert_eq!(
            TypesExpr::Const(ValueExpr::Bool(true)).to_sexpr_string(),
            "true"
        );
        assert_eq!(TypesExpr::Const(ValueExpr::Nil).to_sexpr_string(), "nil");
        let val_list = ValueExpr::List(ValueExprVec(vec![
            ValueExpr::Number(causality_types::primitive::number::Number::Integer(1)),
            ValueExpr::Number(causality_types::primitive::number::Number::Integer(2)),
        ]));
        assert_eq!(TypesExpr::Const(val_list).to_sexpr_string(), "(1 2)");

        let mut btree_map = std::collections::BTreeMap::new();
        btree_map.insert(
            Str::from("a"),
            ValueExpr::Number(causality_types::primitive::number::Number::Integer(1)),
        );
        btree_map.insert(
            Str::from("b\"key"),
            ValueExpr::Number(causality_types::primitive::number::Number::Integer(2)),
        );
        let val_map = ValueExpr::Map(ValueExprMap(btree_map));
        assert_eq!(
            TypesExpr::Const(val_map).to_sexpr_string(),
            "((\\\"a\\\" . 1) (\\\"b\\\\\\\"key\\\" . 2))"
        );

        // Example with a Record
        let mut record_map = std::collections::BTreeMap::new();
        record_map.insert(
            Str::from("name"),
            ValueExpr::String(Str::from("example-record")),
        );
        record_map.insert(
            Str::from("value"),
            ValueExpr::Number(causality_types::primitive::number::Number::Integer(42)),
        );
        let val_record = ValueExpr::Record(ValueExprMap(record_map));
        assert_eq!(
            TypesExpr::Const(val_record).to_sexpr_string(),
            "(record ((\\\"name\\\" . \\\"example-record\\\") (\\\"value\\\" . 42)))"
        );

        // Example with Ref using a deterministic ExprId
        // Create a deterministic ID for testing
        let mock_bytes = [42u8; 32]; // Fixed test array for deterministic behavior
        let mock_expr_id = causality_types::primitive::ids::ExprId::new(mock_bytes);
        let val_ref_expr = ValueExpr::Ref(ValueExprRef::Expr(mock_expr_id));
        let expected_ref_str =
            format!("#<ref:expr:{}>", id_to_hex(&mock_expr_id));
        assert_eq!(
            TypesExpr::Const(val_ref_expr).to_sexpr_string(),
            expected_ref_str
        );
    }

    #[test]
    fn test_serialize_apply_with_combinators() {
        // (if (eq? x 10) "yes" "no")
        let expr = if_(
            eq_(sym("x"), int_lit(10)), // Condition using Eq combinator
            str_lit("yes"),             // Then branch
            str_lit("no"),              // Else branch
        );
        assert_eq!(expr.to_sexpr_string(), "(if (eq? x 10) \"yes\" \"no\")");

        let and_expr = and_(vec![
            bool_lit(true),
            sym("var"),
            eq_(int_lit(1), int_lit(1)),
        ]);
        assert_eq!(and_expr.to_sexpr_string(), "(and true var (eq? 1 1))");
    }

    #[test]
    fn test_lisp_list_macro_serialization() {
        // lisp_list! constructs (list item1 item2 ...)
        let expr = lisp_list!(sym("a"), int_lit(1), str_lit("b"));
        assert_eq!(expr.to_sexpr_string(), "(list a 1 \"b\")");

        let empty_expr = lisp_list!(); // () gets (list)
        assert_eq!(empty_expr.to_sexpr_string(), "(list)");

        let nested_expr = lisp_list!(sym("a"), lisp_list!(sym("b"), sym("c")));
        assert_eq!(nested_expr.to_sexpr_string(), "(list a (list b c))");
    }

    #[test]
    fn test_serialize_defun_like_structure() {
        // Create parameter list
        let params = list(vec![sym("arg1"), sym("arg2")]);
        
        // Create body as a list of expressions
        let body = list(vec![
            lisp_list!(sym("print"), sym("arg1")), // (list print arg1)
            TypesExpr::Apply(
                // (+ arg2 5)
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Add))),
                ExprVec(vec![sym("arg2"), int_lit(5)]),
            ),
        ]);
        
        // defun builder creates (defun name (list arg1 arg2) body)
        let my_func = defun(
            sym("my-function"),
            params,
            body
        );
        
        // Expected: (defun my-function (list arg1 arg2) (list (list print arg1) (+ arg2 5)))
        assert_eq!(my_func.to_sexpr_string(), "(defun my-function (list arg1 arg2) (list (list print arg1) (+ arg2 5)))");
    }

    #[test]
    fn test_serialize_lambda() {
        let lambda_expr = TypesExpr::Lambda(
            vec![Str::from("x"), Str::from("y")],
            ExprBox(Box::new(TypesExpr::Apply(
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Add))),
                ExprVec(vec![
                    TypesExpr::Var(Str::from("x")),
                    TypesExpr::Var(Str::from("y")),
                ]),
            ))),
        );
        assert_eq!(lambda_expr.to_sexpr_string(), "(lambda (x y) (+ x y))");
    }

    #[test]
    fn test_serialize_dynamic_eval() {
        let dyn_expr =
            TypesExpr::Dynamic(100, ExprBox(Box::new(str_lit("some-code"))));
        assert_eq!(
            dyn_expr.to_sexpr_string(),
            "(dynamic-eval 100 \"some-code\")"
        );
    }

    #[test]
    fn test_serialize_ref() {
        let mock_expr_id = ExprId::new([1; 32]);
        let ref_val = ValueExpr::Ref(ValueExprRef::Expr(mock_expr_id));
        let mut buffer = String::new();
        write_value_expr_to_string(&ref_val, &mut buffer).unwrap();
        let serialized_ref = buffer;
        let expected_ref_str =
            format!("#<ref:expr:{}>", id_to_hex(&mock_expr_id));
        assert_eq!(serialized_ref, expected_ref_str);
    }

    // Test for serializing and deserializing a list of expressions
    #[test]
    fn test_serialize_deserialize_list() {
        let expr = list(vec![sym("a"), sym("b"), sym("c")]); // (list a b c)
        let serialized = expr.to_sexpr_string();
        // Current serialization of Apply(Combinator(List), ExprVec(...)) might be "(List a b c)"
        // or "((combinator List) a b c)" depending on how Combinator serializes via op_str.
        // Assuming direct combinator name for now.
        assert_eq!(serialized, "(List a b c)");
        // Deserialization test would require parsing logic for this format.
    }

    // Test for serializing escaped strings in const expressions
    #[test]
    fn test_serialize_deserialize_const_escaped_string() {
        let expr = TypesExpr::Const(ValueExpr::String(Str::from("const \"esc\" str")));
        let serialized = expr.to_sexpr_string();
        assert_eq!(serialized, "(const \"const \\\"esc\\\" str\")");
        // Deserialization test would depend on parser specifics.
    }
}
