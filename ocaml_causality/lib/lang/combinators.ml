(* ------------ ATOMIC COMBINATORS ------------ *)
(* Purpose: Atomic combinators and function primitives *)

open Ocaml_causality_core
open Expr
open Value

(* ------------ COMBINATOR DEFINITIONS ------------ *)

(* Basic atomic combinators *)
let identity = Expr.lambda [LispValue.symbol "x"] (Expr.const (LispValue.symbol "x"))

let const_combinator value = 
  Expr.lambda [LispValue.symbol "_"] (Expr.const value)

let compose f g = 
  Expr.lambda [LispValue.symbol "x"] 
    (Expr.apply f [Expr.apply g [Expr.const (LispValue.symbol "x")]])

let flip f = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply f [Expr.const (LispValue.symbol "y"); Expr.const (LispValue.symbol "x")])

let curry f =
  Expr.lambda [LispValue.symbol "x"]
    (Expr.lambda [LispValue.symbol "y"]
      (Expr.apply f [Expr.const (LispValue.list [LispValue.symbol "x"; LispValue.symbol "y"])]))

let uncurry f =
  Expr.lambda [LispValue.symbol "pair"]
    (Expr.let_binding "x" (Expr.apply (Expr.const (LispValue.symbol "car")) [Expr.const (LispValue.symbol "pair")])
      (Expr.let_binding "y" (Expr.apply (Expr.const (LispValue.symbol "cdr")) [Expr.const (LispValue.symbol "pair")])
        (Expr.apply (Expr.apply f [Expr.const (LispValue.symbol "x")]) [Expr.const (LispValue.symbol "y")])))

(* ------------ FUNCTION PRIMITIVES ------------ *)

(* Arithmetic primitives *)
let add_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol "+")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

let sub_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol "-")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

let mul_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol "*")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

let div_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol "/")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

(* Comparison primitives *)
let eq_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol "=")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

let lt_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol "<")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

let gt_primitive = 
  Expr.lambda [LispValue.symbol "x"; LispValue.symbol "y"]
    (Expr.apply (Expr.const (LispValue.symbol ">")) 
      [Expr.const (LispValue.symbol "x"); Expr.const (LispValue.symbol "y")])

(* List primitives *)
let cons_primitive = 
  Expr.lambda [LispValue.symbol "head"; LispValue.symbol "tail"]
    (Expr.apply (Expr.const (LispValue.symbol "cons")) 
      [Expr.const (LispValue.symbol "head"); Expr.const (LispValue.symbol "tail")])

let car_primitive = 
  Expr.lambda [LispValue.symbol "list"]
    (Expr.apply (Expr.const (LispValue.symbol "car")) 
      [Expr.const (LispValue.symbol "list")])

let cdr_primitive = 
  Expr.lambda [LispValue.symbol "list"]
    (Expr.apply (Expr.const (LispValue.symbol "cdr")) 
      [Expr.const (LispValue.symbol "list")])

let length_primitive = 
  Expr.lambda [LispValue.symbol "list"]
    (Expr.apply (Expr.const (LispValue.symbol "length")) 
      [Expr.const (LispValue.symbol "list")])

(* ------------ COMBINATOR UTILITIES ------------ *)

(* Combinator composition and validation functions *)
let validate_combinator expr =
  try
    let _ = Expr.to_string expr in
    let free_vars = Expr.free_variables expr in
    List.length free_vars = 0  (* Combinators should be closed expressions *)
  with
  | _ -> false

let compose_combinators combinators =
  match combinators with
  | [] -> identity
  | [single] -> single
  | first :: rest ->
      List.fold_left compose first rest

let apply_combinator combinator args =
  List.fold_left (fun acc arg -> Expr.apply acc [arg]) combinator args

let partial_apply combinator args =
  apply_combinator combinator args

(* Higher-order combinators *)
let map_combinator f = 
  Expr.lambda [LispValue.symbol "list"]
    (Expr.apply (Expr.const (LispValue.symbol "map")) 
      [f; Expr.const (LispValue.symbol "list")])

let filter_combinator predicate = 
  Expr.lambda [LispValue.symbol "list"]
    (Expr.apply (Expr.const (LispValue.symbol "filter")) 
      [predicate; Expr.const (LispValue.symbol "list")])

let fold_combinator f init = 
  Expr.lambda [LispValue.symbol "list"]
    (Expr.apply (Expr.const (LispValue.symbol "fold")) 
      [f; Expr.const init; Expr.const (LispValue.symbol "list")])

(* ------------ EVALUATION ------------ *)

(* Combinator evaluation functions *)
let eval_combinator ctx combinator args =
  let applied = apply_combinator combinator args in
  Expr.eval_expr ctx applied

let eval_primitive_combinator name args =
  match name, args with
  | "+", [Int a; Int b] -> Ok (Int (Int64.add a b))
  | "-", [Int a; Int b] -> Ok (Int (Int64.sub a b))
  | "*", [Int a; Int b] -> Ok (Int (Int64.mul a b))
  | "/", [Int a; Int b] when b <> 0L -> Ok (Int (Int64.div a b))
  | "=", [a; b] -> Ok (Bool (LispValue.equal a b))
  | "<", [Int a; Int b] -> Ok (Bool (Int64.compare a b < 0))
  | ">", [Int a; Int b] -> Ok (Bool (Int64.compare a b > 0))
  | "cons", [head; List tail] -> Ok (List (head :: tail))
  | "car", [List (head :: _)] -> Ok head
  | "car", [List []] -> Ok Unit
  | "cdr", [List (_ :: tail)] -> Ok (List tail)
  | "cdr", [List []] -> Ok (List [])
  | "length", [List l] -> Ok (Int (Int64.of_int (List.length l)))
  | _, _ -> Error (FFIError ("Unknown or invalid primitive: " ^ name))

(* Combinator registry *)
module CombinatorRegistry = struct
  type t = (string * Expr.t) list ref

  let create () = ref []

  let register registry name combinator =
    registry := (name, combinator) :: !registry

  let lookup registry name =
    List.assoc_opt name !registry

  let list_combinators registry =
    List.map fst !registry
end

(* Default combinator registry *)
let default_registry = CombinatorRegistry.create ()

let () =
  let open CombinatorRegistry in
  register default_registry "identity" identity;
  register default_registry "compose" (compose identity identity);
  register default_registry "flip" (flip identity);
  register default_registry "curry" (curry identity);
  register default_registry "uncurry" (uncurry identity);
  register default_registry "add" add_primitive;
  register default_registry "sub" sub_primitive;
  register default_registry "mul" mul_primitive;
  register default_registry "div" div_primitive;
  register default_registry "eq" eq_primitive;
  register default_registry "lt" lt_primitive;
  register default_registry "gt" gt_primitive;
  register default_registry "cons" cons_primitive;
  register default_registry "car" car_primitive;
  register default_registry "cdr" cdr_primitive;
  register default_registry "length" length_primitive 