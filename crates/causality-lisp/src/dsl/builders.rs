// crates/causality-lisp/src/dsl/builders.rs
//! Defines builder functions for constructing canonical Lisp Expr instances.

// Remove LispExpr import, use canonical Expr from causality_types
// use super::types::LispExpr;
use causality_types::primitive::string::Str;
use causality_types::expr::ast::{
    Atom, AtomicCombinator, Expr as TypesExpr, ExprBox, ExprVec,
};
use causality_types::expr::value::ValueExpr;
// HashMap is removed as map() builder is removed for now.
// use std::collections::HashMap;

/// Creates a Lisp variable/symbol expression.
pub fn sym(name: impl Into<String>) -> TypesExpr {
    TypesExpr::Var(Str::from(name.into()))
}

/// Creates a Lisp string literal expression.
pub fn str_lit(value: impl Into<String>) -> TypesExpr {
    TypesExpr::Atom(Atom::String(Str::from(value.into())))
}

/// Creates a Lisp integer literal expression.
pub fn int_lit(value: i64) -> TypesExpr {
    TypesExpr::Atom(Atom::Integer(value))
}

/// Creates a Lisp boolean literal expression.
pub fn bool_lit(value: bool) -> TypesExpr {
    TypesExpr::Atom(Atom::Boolean(value))
}

/// Creates a Lisp nil/unit literal expression.
pub fn nil() -> TypesExpr {
    TypesExpr::Atom(Atom::Nil)
}

/// Creates a Lisp keyword literal expression from a string.
/// Example: `keyword_lit("foo")` -> `:foo` (Lisp keyword)
pub fn keyword_lit(name: impl Into<Str>) -> TypesExpr {
    // Use String variant with a prefix to indicate it's a keyword
    let keyword_name = name.into();
    TypesExpr::Const(ValueExpr::String(keyword_name))
}

/// Creates a Lisp `(list item1 item2 ...)` expression.
/// This constructs a data list using the List combinator.
pub fn list(items: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
        ExprVec(items),
    )
}

/// Macro to easily create Lisp `(list item1 item2 ...)` expressions.
/// Example: `lisp_list!(sym("a"), int_lit(1), str_lit("b"))`
/// becomes `(list a 1 "b")`
#[macro_export]
macro_rules! lisp_list {
    () => {
        // Empty list: (list)
        $crate::dsl::builders::list(Vec::new())
    };
    ( $( $x:expr ),* ) => {
        $crate::dsl::builders::list(vec![ $( $x ),* ])
    };
}

// The map() builder is removed for now. It will be re-added when AtomicCombinator::MakeMap is available
// or a clear strategy for alist construction is decided for the DSL.
/*
/// Creates a Lisp map (association list or hash table).
pub fn map(items: HashMap<String, TypesExpr>) -> TypesExpr {
    // Placeholder: This needs to construct an alist or use a make-map combinator
    // For now, let's serialize to an alist `((key . val) ...)`
    // This is complex to do with current Expr structure without helpers for `cons` or `dot_pair`.
    // Temporarily creating a list of lists: ( (key1 val1) (key2 val2) ... )
    let alist_items: Vec<TypesExpr> = items
        .into_iter()
        .map(|(k, v)| {
            // Each item is (key value)
            list(vec![str_lit(k), v]) // Assuming keys are strings for now
        })
        .collect();
    list(alist_items)
}
*/

//-----------------------------------------------------------------------------
// Function Definition Builders
//-----------------------------------------------------------------------------

/// Creates a (defun name (param1 param2 ...) body) expression.
/// 
/// Parameters:
/// - name: function name as string or symbol
/// - params: parameter list expression (created with `list`)
/// - body: body expression (often a sequence created with `begin`)
pub fn defun(
    name: TypesExpr,
    params: TypesExpr,
    body: TypesExpr,
) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Defun))),
        ExprVec(vec![
            name,
            params,
            body,
        ]),
    )
}

/// (begin expr1 expr2 ...)
/// Evaluates expressions in order, returns the value of the last expression.
/// This is typically represented as `(list expr1 expr2 ...)` and relies on interpreter
/// semantics for lists in an application context if not a direct data list.
/// Or, it could be a specific 'sequence' combinator if defined.
/// For now, mapping to (list ...) as a common representation for a sequence of forms.
pub fn begin(exprs: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
        ExprVec(exprs),
    )
}

// --- Combinator Builders ---

/// Creates an (if condition then_expr else_expr) expression.
pub fn if_(
    condition: TypesExpr,
    then_expr: TypesExpr,
    else_expr: TypesExpr,
) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::If))),
        ExprVec(vec![condition, then_expr, else_expr]),
    )
}

/// Creates an (and expr1 expr2 ... exprN) expression.
pub fn and_(expressions: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::And))),
        ExprVec(expressions),
    )
}

/// Creates an (or expr1 expr2 ...) expression that evaluates expressions until one is truthy.
pub fn or_(expressions: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Or))),
        ExprVec(expressions),
    )
}

/// Creates a (not expr) expression.
pub fn not_(expression: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Not))),
        ExprVec(vec![expression]),
    )
}

/// Creates an (eq? expr1 expr2) expression.
/// Note: `eq?` is often `eq` or `equal` in Lisp. Using `AtomicCombinator::Eq`.
pub fn eq_(expr1: TypesExpr, expr2: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Eq))),
        ExprVec(vec![expr1, expr2]),
    )
}

// --- Arithmetic Combinator Builders ---

/// Creates an (+ expr1 expr2 ... exprN) expression.
pub fn add(expressions: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Add))),
        ExprVec(expressions),
    )
}

/// Creates a (- expr1 expr2?) expression. (Lisp '-' can be unary or binary)
pub fn sub(expressions: Vec<TypesExpr>) -> TypesExpr {
    // TODO: Add validation or specific builders for unary/binary versions if AST/combinator implies fixed arity.
    // For now, allows variadic, matching Lisp flexibility, though AtomicCombinator::Sub might imply specific arity.
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Sub))),
        ExprVec(expressions),
    )
}

/// Creates a (* expr1 expr2 ... exprN) expression.
pub fn mul(expressions: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Mul))),
        ExprVec(expressions),
    )
}

/// Creates a (/ expr1 expr2?) expression. (Lisp '/' can be unary or binary)
pub fn div(expressions: Vec<TypesExpr>) -> TypesExpr {
    // TODO: Add validation or specific builders for unary/binary versions.
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Div))),
        ExprVec(expressions),
    )
}

//-----------------------------------------------------------------------------
// Symbol Binding and Definition
//-----------------------------------------------------------------------------

/// Creates a let expression: (let name value body)
/// This binds a name to a value in the scope of the body expression
pub fn let_(name: TypesExpr, value: TypesExpr, body: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Let))),
        ExprVec(vec![name, value, body]),
    )
}

/// Creates a (define symbol value) expression.
///
/// Parameters:
/// - symbol: symbol to define
/// - value: expression that will be evaluated and bound to the symbol
pub fn define(symbol: TypesExpr, value: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Define))),
        ExprVec(vec![symbol, value]),
    )
}

/// Creates a (quote expr) expression that prevents evaluation of the quoted expression.
///
/// Parameters:
/// - expr: expression to quote
pub fn quote(expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Quote))),
        ExprVec(vec![expr]),
    )
}

//-----------------------------------------------------------------------------
// Function Application
//-----------------------------------------------------------------------------

/// Creates a function application expression (func arg1 arg2 ...)
/// 
/// Parameters:
/// - func: function expression (usually a symbol)
/// - args: vector of argument expressions
pub fn apply(func: TypesExpr, args: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(func)),
        ExprVec(args),
    )
}

//-----------------------------------------------------------------------------
// Comparison Combinators
//-----------------------------------------------------------------------------

/// Creates a (> expr1 expr2) expression.
pub fn gt(expr1: TypesExpr, expr2: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Gt))),
        ExprVec(vec![expr1, expr2]),
    )
}

/// Creates a (< expr1 expr2) expression.
pub fn lt(expr1: TypesExpr, expr2: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Lt))),
        ExprVec(vec![expr1, expr2]),
    )
}

/// Creates a (>= expr1 expr2) expression.
pub fn gte(expr1: TypesExpr, expr2: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Gte))),
        ExprVec(vec![expr1, expr2]),
    )
}

/// Creates a (<= expr1 expr2) expression.
pub fn lte(expr1: TypesExpr, expr2: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Lte))),
        ExprVec(vec![expr1, expr2]),
    )
}

// --- Data Access Combinator Builders ---

/// Creates a (get-context-value key_str) expression.
/// `key_str` should be a string literal naming the context value.
pub fn get_context_value(key: impl Into<String>) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(
            AtomicCombinator::GetContextValue,
        ))),
        ExprVec(vec![str_lit(key)]), // key is passed as a Lisp string literal to the combinator
    )
}

/// Creates a (get-field target_expr field_name_str) expression.
/// `field_name_str` should be a string literal naming the field.
pub fn get_field(
    target_expr: TypesExpr,
    field_name: impl Into<String>,
) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::GetField))),
        ExprVec(vec![target_expr, str_lit(field_name)]),
    )
}

// --- Effect Status Combinator Builders ---

/// Creates a (completed effect_ref_expr) expression.
/// `effect_ref_expr` is an expression that should evaluate to the effect reference/ID.
pub fn completed(effect_ref_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Completed))),
        ExprVec(vec![effect_ref_expr]),
    )
}

// --- List Operation Combinator Builders ---

/// Creates a (nth index list) expression.
pub fn nth(index_expr: TypesExpr, list_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Nth))),
        ExprVec(vec![index_expr, list_expr]),
    )
}

/// Creates a (len list) expression.
pub fn len(list_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Length))),
        ExprVec(vec![list_expr]),
    )
}

/// Creates a (cons item list) expression.
pub fn cons(item_expr: TypesExpr, list_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Cons))),
        ExprVec(vec![item_expr, list_expr]),
    )
}

/// Creates a (first list) or (car list) expression.
pub fn first(list_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Car))),
        ExprVec(vec![list_expr]),
    )
}

/// Creates a (rest list) or (cdr list) expression.
pub fn rest(list_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Cdr))),
        ExprVec(vec![list_expr]),
    )
}

// --- Map/Struct Operation Combinator Builders ---

/// Creates a (make-map '((key1 . val1) (key2 . val2) ...)) expression.
/// The `key_val_pairs` are expected to be (key, value) `TypesExpr` tuples.
/// The map is constructed by the MakeMap combinator.
pub fn make_map(key_val_pairs: Vec<(TypesExpr, TypesExpr)>) -> TypesExpr {
    // Convert Vec<(TypesExpr, TypesExpr)> into a Lisp list of pairs for make-map:
    // '((key1 . val1) (key2 . val2) ...)
    // Each pair (k . v) can be represented as (list k v) if we don't have a dot primitive.
    // Let's assume `make-map` expects a list of 2-element lists: ( (k1 v1) (k2 v2) ... )
    let list_of_pairs: Vec<TypesExpr> = key_val_pairs
        .into_iter()
        .map(|(k, v)| list(vec![k, v])) // Each pair becomes (list k v)
        .collect();

    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MakeMap))),
        // make-map takes one argument: the list of key-value pair lists.
        ExprVec(vec![list(list_of_pairs)]),
    )
}

/// Creates a (map-get key map) expression.
pub fn get_map_value(key_expr: TypesExpr, map_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MapGet))),
        ExprVec(vec![key_expr, map_expr]),
    )
}

/// Creates a (has-key? key map) expression.
pub fn has_key(key_expr: TypesExpr, map_expr: TypesExpr) -> TypesExpr {
    TypesExpr::Apply(
        ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MapHasKey))),
        ExprVec(vec![key_expr, map_expr]),
    )
}

#[cfg(test)]
mod tests {
    use super::*; // Import builders
    use causality_types::primitive::string::Str;
    use causality_types::expr::ast::{
        Atom, AtomicCombinator, Expr as TypesExpr, ExprBox, ExprVec,
    };

    #[test]
    fn test_sym_builder() {
        assert_eq!(sym("test-sym"), TypesExpr::Var(Str::from("test-sym")));
    }

    #[test]
    fn test_str_lit_builder() {
        assert_eq!(
            str_lit("hello"),
            TypesExpr::Atom(Atom::String(Str::from("hello")))
        );
    }

    #[test]
    fn test_int_lit_builder() {
        assert_eq!(int_lit(123), TypesExpr::Atom(Atom::Integer(123)));
    }

    #[test]
    fn test_bool_lit_builder() {
        assert_eq!(bool_lit(true), TypesExpr::Atom(Atom::Boolean(true)));
        assert_eq!(bool_lit(false), TypesExpr::Atom(Atom::Boolean(false)));
    }

    #[test]
    fn test_nil_builder() {
        assert_eq!(nil(), TypesExpr::Atom(Atom::Nil));
    }

    #[test]
    fn test_list_builder() {
        let items = vec![sym("a"), int_lit(1)];
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
            ExprVec(vec![
                TypesExpr::Var(Str::from("a")),
                TypesExpr::Atom(Atom::Integer(1)),
            ]),
        );
        assert_eq!(list(items), expected);

        let empty_list_expr = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
            ExprVec(vec![]),
        );
        assert_eq!(list(vec![]), empty_list_expr);
    }

    #[test]
    fn test_lisp_list_macro() {
        let expected_empty = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
            ExprVec(vec![]),
        );
        assert_eq!(lisp_list!(), expected_empty);

        let item1 = sym("item1");
        let item2 = int_lit(42);
        let expected_items = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
            ExprVec(vec![item1.clone(), item2.clone()]),
        );
        assert_eq!(lisp_list!(item1, item2), expected_items);
    }

    #[test]
    fn test_defun_builder() {
        let func_name = "my-func";
        let args = vec!["x", "y"];
        let _body_exprs = vec![lisp_list!(sym("print"), sym("x"))];

        // Expected structure: (defun my-func (list x y) (list (print x)))
        // sym("defun")
        // sym("my-func")
        // (list sym("x") sym("y"))
        // (list (list sym("print") sym("x")))

        let expected_params_list = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
            ExprVec(vec![sym("x"), sym("y")]),
        );

        let _expected_body_element = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
            ExprVec(vec![TypesExpr::Apply(
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::List))),
                ExprVec(vec![sym("print"), sym("x")]),
            )]),
        );
        // Correction: defun body is a sequence of expressions, not a list of expressions for this builder's output
        // The builder does: apply_args.extend(body);
        // So body expressions are directly added as args to the main 'defun' Apply node.
        let single_body_expr = lisp_list!(sym("print"), sym("x"));

        let expected = TypesExpr::Apply(
            ExprBox(Box::new(sym("defun"))),
            ExprVec(vec![
                sym(func_name),
                expected_params_list,     // This is (list x y)
                single_body_expr.clone(), // This is (list print x)
            ]),
        );

        assert_eq!(defun(sym(func_name), list(args.iter().map(|arg| sym(*arg)).collect()), single_body_expr), expected);
    }

    #[test]
    fn test_if_builder() {
        let cond = bool_lit(true);
        let then_e = str_lit("yes");
        let else_e = str_lit("no");
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::If))),
            ExprVec(vec![cond.clone(), then_e.clone(), else_e.clone()]),
        );
        assert_eq!(if_(cond, then_e, else_e), expected);
    }

    #[test]
    fn test_and_builder() {
        let exprs = vec![bool_lit(true), sym("var")];
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::And))),
            ExprVec(vec![bool_lit(true), sym("var")]),
        );
        assert_eq!(and_(exprs), expected);
        assert_eq!(
            and_(vec![]),
            TypesExpr::Apply(
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::And))),
                ExprVec(vec![]),
            )
        );
    }

    #[test]
    fn test_or_builder() {
        let exprs = vec![bool_lit(false), sym("var")];
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Or))),
            ExprVec(vec![bool_lit(false), sym("var")]),
        );
        assert_eq!(or_(exprs), expected);
        assert_eq!(
            or_(vec![]),
            TypesExpr::Apply(
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Or))),
                ExprVec(vec![]),
            )
        );
    }

    #[test]
    fn test_not_builder() {
        let expr = bool_lit(true);
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Not))),
            ExprVec(vec![expr.clone()]),
        );
        assert_eq!(not_(expr), expected);
    }

    #[test]
    fn test_eq_builder() {
        let expr1 = int_lit(1);
        let expr2 = int_lit(1);
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Eq))),
            ExprVec(vec![expr1.clone(), expr2.clone()]),
        );
        assert_eq!(eq_(expr1, expr2), expected);
    }

    // --- Tests for Arithmetic Combinator Builders ---
    #[test]
    fn test_add_builder() {
        let exprs = vec![int_lit(1), int_lit(2), int_lit(3)];
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Add))),
            ExprVec(vec![int_lit(1), int_lit(2), int_lit(3)]),
        );
        assert_eq!(add(exprs), expected);
        assert_eq!(
            add(vec![]),
            TypesExpr::Apply(
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Add))),
                ExprVec(vec![]),
            )
        );
    }

    #[test]
    fn test_sub_builder() {
        let exprs_binary = vec![int_lit(5), int_lit(2)];
        let expected_binary = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Sub))),
            ExprVec(vec![int_lit(5), int_lit(2)]),
        );
        assert_eq!(sub(exprs_binary), expected_binary);

        let expr_unary = vec![int_lit(5)];
        let expected_unary = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Sub))),
            ExprVec(vec![int_lit(5)]),
        );
        assert_eq!(sub(expr_unary), expected_unary);
    }

    #[test]
    fn test_mul_builder() {
        let exprs = vec![int_lit(2), int_lit(3), int_lit(4)];
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Mul))),
            ExprVec(vec![int_lit(2), int_lit(3), int_lit(4)]),
        );
        assert_eq!(mul(exprs), expected);
        assert_eq!(
            mul(vec![]),
            TypesExpr::Apply(
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Mul))),
                ExprVec(vec![]),
            )
        ); // Multiplication identity is usually 1, but (mul) is valid Lisp
    }

    #[test]
    fn test_div_builder() {
        let exprs_binary = vec![int_lit(10), int_lit(2)];
        let expected_binary = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Div))),
            ExprVec(vec![int_lit(10), int_lit(2)]),
        );
        assert_eq!(div(exprs_binary), expected_binary);

        let expr_unary = vec![int_lit(5)]; // e.g. for 1/5
        let expected_unary = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Div))),
            ExprVec(vec![int_lit(5)]),
        );
        assert_eq!(div(expr_unary), expected_unary);
    }

    // --- Tests for Comparison Combinator Builders ---
    #[test]
    fn test_gt_builder() {
        let expr1 = int_lit(5);
        let expr2 = int_lit(2);
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Gt))),
            ExprVec(vec![expr1.clone(), expr2.clone()]),
        );
        assert_eq!(gt(expr1, expr2), expected);
    }

    #[test]
    fn test_lt_builder() {
        let expr1 = int_lit(2);
        let expr2 = int_lit(5);
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Lt))),
            ExprVec(vec![expr1.clone(), expr2.clone()]),
        );
        assert_eq!(lt(expr1, expr2), expected);
    }

    #[test]
    fn test_gte_builder() {
        let expr1 = int_lit(5);
        let expr2 = int_lit(5);
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Gte))),
            ExprVec(vec![expr1.clone(), expr2.clone()]),
        );
        assert_eq!(gte(expr1, expr2), expected);
    }

    #[test]
    fn test_lte_builder() {
        let expr1 = int_lit(2);
        let expr2 = int_lit(2);
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Lte))),
            ExprVec(vec![expr1.clone(), expr2.clone()]),
        );
        assert_eq!(lte(expr1, expr2), expected);
    }

    // --- Tests for Data Access Combinator Builders ---
    #[test]
    fn test_get_context_value_builder() {
        let key = "my-context-key";
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(
                AtomicCombinator::GetContextValue,
            ))),
            ExprVec(vec![str_lit(key)]),
        );
        assert_eq!(get_context_value(key), expected);
    }

    #[test]
    fn test_get_field_builder() {
        let target = sym("my-map");
        let field_name = "my-field";
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::GetField))),
            ExprVec(vec![target.clone(), str_lit(field_name)]),
        );
        assert_eq!(get_field(target, field_name), expected);
    }

    // --- Test for Effect Status Combinator Builder ---
    #[test]
    fn test_completed_builder() {
        let effect_ref = str_lit("effect-id-123");
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Completed))),
            ExprVec(vec![effect_ref.clone()]),
        );
        assert_eq!(completed(effect_ref), expected);
    }

    // --- Tests for List Operation Combinator Builders ---
    #[test]
    fn test_nth_builder() {
        let index = int_lit(0);
        let l = lisp_list!(sym("a"), sym("b"));
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Nth))),
            ExprVec(vec![index.clone(), l.clone()]),
        );
        assert_eq!(nth(index, l), expected);
    }

    #[test]
    fn test_len_builder() {
        let l = lisp_list!(sym("a"), sym("b"));
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Length))),
            ExprVec(vec![l.clone()]),
        );
        assert_eq!(len(l), expected);
    }

    #[test]
    fn test_cons_builder() {
        let item = sym("x");
        let l = lisp_list!(sym("a"), sym("b"));
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Cons))),
            ExprVec(vec![item.clone(), l.clone()]),
        );
        assert_eq!(cons(item, l), expected);
    }

    #[test]
    fn test_first_builder() {
        let l = lisp_list!(sym("a"), sym("b"));
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Car))),
            ExprVec(vec![l.clone()]),
        );
        assert_eq!(first(l), expected);
    }

    #[test]
    fn test_rest_builder() {
        let l = lisp_list!(sym("a"), sym("b"));
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Cdr))),
            ExprVec(vec![l.clone()]),
        );
        assert_eq!(rest(l), expected);
    }

    // --- Tests for Map/Struct Operation Combinator Builders ---
    #[test]
    fn test_make_map_builder() {
        let pairs = vec![
            (str_lit("key1"), int_lit(1)),
            (str_lit("key2"), bool_lit(true)),
        ];

        let cons_pairs = vec![
            cons(str_lit("key1"), int_lit(1)),
            cons(str_lit("key2"), bool_lit(true)),
        ];
        let list_of_cons_pairs = list(cons_pairs);

        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MakeMap))),
            ExprVec(vec![list_of_cons_pairs]),
        );
        assert_eq!(make_map(pairs), expected);

        // Test with empty pairs
        let empty_pairs: Vec<(TypesExpr, TypesExpr)> = Vec::new();
        let expected_empty_map = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MakeMap))),
            ExprVec(vec![list(vec![])]), // (make-map (list))
        );
        assert_eq!(make_map(empty_pairs), expected_empty_map);
    }

    #[test]
    fn test_get_map_value_builder() {
        let key = str_lit("my-key");
        let map_var = sym("my-map-data");
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MapGet))),
            ExprVec(vec![key.clone(), map_var.clone()]),
        );
        assert_eq!(get_map_value(key, map_var), expected);
    }

    #[test]
    fn test_has_key_builder() {
        let key = str_lit("check-key");
        let map_var = sym("some-map");
        let expected = TypesExpr::Apply(
            ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::MapHasKey))),
            ExprVec(vec![key.clone(), map_var.clone()]),
        );
        assert_eq!(has_key(key, map_var), expected);
    }

    #[test]
    fn test_serialize_defun_like_structure() {
        use crate::dsl::serializer::SExprSerializable;

        // Create a body sequence with begin
        let body = begin(vec![
            lisp_list!(sym("print"), sym("arg1")), // (list print arg1)
            TypesExpr::Apply(
                // (+ arg2 5)
                ExprBox(Box::new(TypesExpr::Combinator(AtomicCombinator::Add))),
                ExprVec(vec![sym("arg2"), int_lit(5)]),
            ),
        ]);

        // Create parameter list
        let params = list(vec![sym("arg1"), sym("arg2")]);

        // defun builder creates (defun name (list arg1 arg2) body)
        let my_func = defun(
            sym("my-function"),
            params,
            body
        );

        // Expected: (defun my-function (list arg1 arg2) (begin (list print arg1) (+ arg2 5)))
        assert_eq!(
            my_func.to_sexpr_string(),
            "(defun my-function (list arg1 arg2) (begin (list print arg1) (+ arg2 5)))"
        );
    }
}
