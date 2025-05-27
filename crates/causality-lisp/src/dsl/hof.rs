// crates/causality-lisp/src/dsl/hof.rs
// Purpose: Provides Higher-Order Functions for constructing Lisp S-expressions.

use crate::dsl::builders; // Primitive builders
use causality_core::lisp_adapter::expr_box;
use causality_types::primitive::string::Str;
use causality_types::expr::ast::{Expr as TypesExpr, ExprVec};
use causality_types::expr::value::ValueExpr;
use std::collections::HashMap;

// Use ExprBox type alias from lisp_adapter
use causality_core::lisp_adapter::ExprBox;

// --- Core Constructors (Atoms, Literals, Variables) ---

/// Creates a Lisp symbol expression.
/// Example: `sym("my-var")` -> `my-var` (Lisp symbol)
pub fn sym(name: impl Into<Str>) -> TypesExpr {
    builders::sym(name.into().as_str())
}

/// Creates a Lisp string literal expression.
/// Example: `str_lit("hello")` -> `"hello"` (Lisp string)
pub fn str_lit(value: impl Into<Str>) -> TypesExpr {
    builders::str_lit(value.into().as_str())
}

/// Creates a Lisp integer literal expression.
/// Example: `int_lit(42)` -> `42` (Lisp integer)
pub fn int_lit(value: i64) -> TypesExpr {
    builders::int_lit(value)
}

/// Creates a Lisp boolean literal expression.
/// Example: `bool_lit(true)` -> `true` (Lisp boolean)
pub fn bool_lit(value: bool) -> TypesExpr {
    builders::bool_lit(value)
}

/// Creates a Lisp `nil` expression.
/// Example: `nil()` -> `nil`
pub fn nil() -> TypesExpr {
    builders::nil()
}

/// Creates a Lisp keyword literal expression.
/// Example: `keyword_lit("foo")` -> `:foo`
pub fn keyword_lit(name: impl Into<Str>) -> TypesExpr {
    builders::keyword_lit(name.into())
}

/// Creates a Lisp constant expression from a `ValueExpr`.
/// Example: `value_const(ValueExpr::Int(10))` -> Lisp representation of `(ValueExpr::Int(10))`
pub fn value_const(value: ValueExpr) -> TypesExpr {
    TypesExpr::Const(value)
}

// --- Core Forms (Apply, List, Lambda) ---

/// Creates a Lisp function application: `(func arg1 arg2 ...)`
/// Example: `apply_expr(sym("+"), vec![int_lit(1), int_lit(2)])` -> `(+ 1 2)`
pub fn apply_expr(func: TypesExpr, args: Vec<TypesExpr>) -> TypesExpr {
    TypesExpr::Apply(ExprBox(Box::new(func)), ExprVec(args))
}

/// Creates a Lisp list literal: `(list item1 item2 ...)`
/// Note: This is an application of the `list` combinator/special form.
/// Example: `list_expr(vec![sym("a"), int_lit(1)])` -> `(list a 1)`
pub fn list_expr(elements: Vec<TypesExpr>) -> TypesExpr {
    builders::list(elements)
}

/// Creates an anonymous Lisp function (lambda).
/// `(fn (param1 param2 ...) body_expr)`
/// The `body_builder` closure receives a `Vec<TypesExpr>` where each element
/// is a Lisp symbol (`Expr::Var`) for the corresponding parameter name.
/// The closure should return a single `TypesExpr` for the lambda body.
pub fn lambda<S, F>(param_names: Vec<S>, body_builder: F) -> TypesExpr
where
    S: Into<Str>,
    F: FnOnce(Vec<TypesExpr>) -> TypesExpr,
{
    let params_str: Vec<Str> = param_names.into_iter().map(Into::into).collect();
    let param_sym_exprs: Vec<TypesExpr> = params_str
        .iter()
        .map(|p_name| sym(*p_name))
        .collect();

    let body_expr = body_builder(param_sym_exprs);
    TypesExpr::Lambda(params_str, expr_box::new(body_expr))
}

// --- LetScopeVars ---
// Helper struct providing convenient access to variables bound in a `let*` form.
// Passed to the body-generating closure of `let_star`.
#[derive(Debug)]
pub struct LetScopeVars {
    vars_map: HashMap<String, TypesExpr>,
}

impl LetScopeVars {
    /// Creates a new `LetScopeVars` instance from a set of binding names.
    /// Each variable name is mapped to its Lisp symbol expression.
    fn from_binding_names(binding_names: &[(Str, TypesExpr)]) -> Self {
        let mut vars_map = HashMap::new();
        for (name, _) in binding_names {
            vars_map.insert(name.to_string(), sym(*name));
        }
        Self { vars_map }
    }

    /// Retrieves the Lisp symbol expression for a bound variable by its name.
    /// Panics if the variable name is not found in the current scope.
    /// The returned `TypesExpr` is a clone and can be used multiple times.
    pub fn get(&self, name: &str) -> TypesExpr {
        self.vars_map
            .get(name)
            .unwrap_or_else(|| {
                panic!(
                    "Lisp DSL Error: Variable '{}' not found in let* scope.",
                    name
                )
            })
            .clone()
    }
}

// --- Special Forms HOFs & Builders ---

/// Creates a Lisp `(defun fn-name (param1 ...) body1 body2 ...)` expression.
pub fn def_fn<SFn, SP, F>(
    fn_name: SFn,
    param_names: Vec<SP>,
    body_builder: F,
) -> TypesExpr
where
    SFn: Into<Str>,
    SP: Into<Str> + Clone,
    F: FnOnce(Vec<TypesExpr>) -> Vec<TypesExpr>,
{
    let fn_name_sym_expr = sym(fn_name.into());
    let params_as_str: Vec<Str> =
        param_names.iter().map(|p| p.clone().into()).collect();

    let param_sym_exprs: Vec<TypesExpr> = params_as_str
        .iter()
        .map(|p_name| sym(*p_name))
        .collect();

    let body_exprs: Vec<TypesExpr> = body_builder(param_sym_exprs);

    let lisp_params_list_items: Vec<TypesExpr> =
        params_as_str.into_iter().map(sym).collect();
    let lisp_params_list_expr = list_expr(lisp_params_list_items);

    let mut defun_args = vec![fn_name_sym_expr, lisp_params_list_expr];
    defun_args.extend(body_exprs);
    apply_expr(sym("defun"), defun_args)
}

// --- LetBindingsBuilder ---
#[derive(Default)]
pub struct LetBindingsBuilder {
    bindings: Vec<(Str, TypesExpr)>,
}

impl LetBindingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind<S: Into<Str>>(mut self, name: S, value_expr: TypesExpr) -> Self {
        self.bindings.push((name.into(), value_expr));
        self
    }

    // Consumes the builder and takes the body closure
    pub fn build<F>(self, body_builder: F) -> TypesExpr
    where
        F: FnOnce(LetScopeVars) -> Vec<TypesExpr>,
    {
        let lisp_bindings_list_items: Vec<TypesExpr> = self
            .bindings
            .iter()
            .map(|(name_str, val_expr)| {
                list_expr(vec![sym(*name_str), val_expr.clone()])
            })
            .collect();
        let lisp_bindings_list_expr = list_expr(lisp_bindings_list_items);

        let scope = LetScopeVars::from_binding_names(&self.bindings);

        let body_exprs: Vec<TypesExpr> = body_builder(scope);

        let mut let_star_args = vec![lisp_bindings_list_expr];
        let_star_args.extend(body_exprs);
        apply_expr(sym("let*"), let_star_args)
    }
}

// --- IfExpressionBuilder ---
pub struct IfExpressionBuilder {
    condition: TypesExpr,
    then_branch: Option<TypesExpr>,
    else_branch: Option<TypesExpr>,
}

impl IfExpressionBuilder {
    pub fn new(condition: TypesExpr) -> Self {
        Self {
            condition,
            then_branch: None,
            else_branch: None,
        }
    }

    pub fn then_branch(mut self, expr: TypesExpr) -> Self {
        self.then_branch = Some(expr);
        self
    }

    pub fn else_branch(mut self, expr: TypesExpr) -> Self {
        self.else_branch = Some(expr);
        self
    }

    pub fn build(self) -> TypesExpr {
        let then_b = self
            .then_branch
            .expect("IfExpressionBuilder: then_branch must be set.");
        if let Some(else_b) = self.else_branch {
            apply_expr(sym("if"), vec![self.condition, then_b, else_b])
        } else {
            apply_expr(sym("if"), vec![self.condition, then_b]) // Lisp 'if' without else often evals to nil
        }
    }
}

// --- Host Function Wrappers ---

// --- Combinator Wrappers (Host Functions & Standard Library) ---

/// `(and expr1 expr2 ...)`
pub fn and_exprs(exprs: Vec<TypesExpr>) -> TypesExpr {
    apply_expr(sym("and"), exprs)
}

/// `(or expr1 expr2 ...)`
pub fn or_exprs(exprs: Vec<TypesExpr>) -> TypesExpr {
    apply_expr(sym("or"), exprs)
}

/// `(not expr)`
pub fn not_expr(expr: TypesExpr) -> TypesExpr {
    apply_expr(sym("not"), vec![expr])
}

/// `(eq lhs rhs)` or `(eq? lhs rhs)` - using "eq" as per spec example
pub fn eq_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("eq"), vec![lhs, rhs])
}

// Arithmetic
pub fn add_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("+"), vec![lhs, rhs])
}
pub fn sub_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("-"), vec![lhs, rhs])
}
pub fn mul_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("*"), vec![lhs, rhs])
}
pub fn div_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("/"), vec![lhs, rhs])
}

// Comparisons
pub fn gt_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym(">"), vec![lhs, rhs])
}
pub fn lt_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("<"), vec![lhs, rhs])
}
pub fn gte_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym(">="), vec![lhs, rhs])
}
pub fn lte_expr(lhs: TypesExpr, rhs: TypesExpr) -> TypesExpr {
    apply_expr(sym("<="), vec![lhs, rhs])
}

// Data Access & Construction (Host Functions generally)

/// `(get-context-value key_str)`
pub fn get_context_value_expr(key_str: impl Into<Str>) -> TypesExpr {
    call_host_expr("get-context-value", vec![str_lit(key_str)])
}

/// `(get-field target_expr field_name_str)`
pub fn get_field_expr(
    target_expr: TypesExpr,
    field_name_str: impl Into<Str>,
) -> TypesExpr {
    call_host_expr("get-field", vec![target_expr, str_lit(field_name_str)])
}

/// `(completed effect_ref)`
pub fn completed_expr(effect_ref: TypesExpr) -> TypesExpr {
    // effect_ref could be string or int
    call_host_expr("completed", vec![effect_ref])
}

// List Operations (Host Functions or special forms if `list` symbol implies it)
// `list_expr` (above) is for `(list item1 ...)`

/// `(nth index_expr list_expr)`
pub fn nth_expr(index_expr: TypesExpr, list_expr: TypesExpr) -> TypesExpr {
    call_host_expr("nth", vec![index_expr, list_expr])
}

/// `(len list_expr)`
pub fn len_expr(list_expr: TypesExpr) -> TypesExpr {
    call_host_expr("len", vec![list_expr])
}

/// `(cons head_expr tail_list_expr)`
pub fn cons_expr(head_expr: TypesExpr, tail_list_expr: TypesExpr) -> TypesExpr {
    call_host_expr("cons", vec![head_expr, tail_list_expr])
}

/// `(first list_expr)` - often sugar for `(nth 0 list_expr)`
pub fn first_expr(list_expr: TypesExpr) -> TypesExpr {
    call_host_expr("first", vec![list_expr])
}

/// `(rest list_expr)`
pub fn rest_expr(list_expr: TypesExpr) -> TypesExpr {
    call_host_expr("rest", vec![list_expr])
}

// Map/Struct Operations (Host Functions)

/// `(make-map key1_expr val1_expr key2_expr val2_expr ...)`
/// Takes a flat vector of alternating key/value TypesExpr.
pub fn make_map_expr(key_value_pairs: Vec<TypesExpr>) -> TypesExpr {
    call_host_expr("make-map", key_value_pairs)
}

/// Creates a map from a vector of (KeyExpr, ValueExpr) pairs.
/// Generic for any TypesExpr used as keys.
pub fn make_map_from_key_expr_pairs(
    pairs: Vec<(TypesExpr, TypesExpr)>,
) -> TypesExpr {
    let mut flat_args = Vec::new();
    for (key_expr, val_expr) in pairs {
        flat_args.push(key_expr);
        flat_args.push(val_expr);
    }
    make_map_expr(flat_args)
}



/// Convenience for building map with Lisp string literal keys from Rust strings.
/// Example: `make_map_with_string_keys(vec![("key1", expr1), ...])` -> `(make-map "key1" expr1 ...)`
pub fn make_map_with_string_keys<S: Into<Str>>(
    pairs: Vec<(S, TypesExpr)>,
) -> TypesExpr {
    let mut flat_args = Vec::new();
    for (key_str, val_expr) in pairs {
        flat_args.push(str_lit(key_str.into()));
        flat_args.push(val_expr);
    }
    make_map_expr(flat_args)
}

/// Convenience for building map with Lisp keyword literal keys from Rust strings.
/// Example: `make_map_with_keyword_keys(vec![("key1", expr1), ...])` -> `(make-map :key1 expr1 ...)`
pub fn make_map_with_keyword_keys<S: Into<Str>>(
    pairs: Vec<(S, TypesExpr)>,
) -> TypesExpr {
    let mut flat_args = Vec::new();
    for (key_str, val_expr) in pairs {
        flat_args.push(keyword_lit(key_str.into()));
        flat_args.push(val_expr);
    }
    make_map_expr(flat_args)
}

/// `(get key_expr map_or_struct_expr)`
pub fn get_expr(key_expr: TypesExpr, map_or_struct_expr: TypesExpr) -> TypesExpr {
    call_host_expr("get", vec![key_expr, map_or_struct_expr])
}

/// `(has-key? key_expr map_or_struct_expr)`
pub fn has_key_expr(
    key_expr: TypesExpr,
    map_or_struct_expr: TypesExpr,
) -> TypesExpr {
    call_host_expr("has-key?", vec![key_expr, map_or_struct_expr])
}

/// Generic host function call: `(fn_name_symbol arg1 arg2 ...)`
pub fn call_host_expr(fn_name: impl Into<Str>, args: Vec<TypesExpr>) -> TypesExpr {
    apply_expr(sym(fn_name.into()), args)
}

// --- Dynamic Expression Form ---

/// `(dynamic N expr)`
pub fn dynamic_expr(step_bound: u32, expr: TypesExpr) -> TypesExpr {
    TypesExpr::Dynamic(step_bound, expr_box::new(expr))
}
