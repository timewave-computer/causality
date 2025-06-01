(*
 * Combinators Module
 *
 * This module implements the evaluation logic for atomic combinators
 * in the expression language. These combinators provide the primitive
 * operations that expressions can perform.
 *)

open Ast

(* ------------ COMBINATOR EVALUATION ------------ *)

(** Evaluate a combinator with arguments *)
let eval_combinator comb args =
  match comb, args with
  (* Core combinators *)
  | S, [_f; _g; _x] ->
      (* S combinator: S f g x = (f x) (g x) *)
      (* Will be implemented in the expression evaluator *)
      VUnit
  | K, [x; _] ->
      (* K combinator: K x y = x *)
      x
  | I, [x] ->
      (* I combinator: I x = x *)
      x
  | C, [_f; _x; _y] ->
      (* C combinator: C f x y = f y x (swap args) *)
      (* Will be implemented in the expression evaluator *)
      VUnit

  (* Control flow *)
  | If, [_cond; _then_expr; _else_expr] ->
      (* If expression evaluation will be handled by the evaluator *)
      VUnit
  | Let, [_bindings; _body] ->
      (* Let expression evaluation will be handled by the evaluator *)
      VUnit
  | LetStar, [_bindings; _body] ->
      (* Let* expression evaluation will be handled by the evaluator *)
      VUnit

  (* Boolean logic *)
  | And, _args ->
      (* Evaluate each argument, return first false or last true *)
      VAtom (Boolean true) (* Placeholder *)
  | Or, _args ->
      (* Evaluate each argument, return first true or last false *)
      VAtom (Boolean false) (* Placeholder *)
  | Not, [VAtom (Boolean b)] ->
      VAtom (Boolean (not b))
  | Not, _ ->
      VAtom (Boolean false) (* Default for non-boolean *)

  (* Comparison *)
  | Eq, [a; b] ->
      VAtom (Boolean (a = b)) (* Simple structural equality *)
  | Gt, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Boolean (Int64.compare a b > 0))
  | Lt, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Boolean (Int64.compare a b < 0))
  | Gte, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Boolean (Int64.compare a b >= 0))
  | Lte, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Boolean (Int64.compare a b <= 0))

  (* Arithmetic *)
  | Add, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Integer (Int64.add a b))
  | Sub, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Integer (Int64.sub a b))
  | Mul, [VAtom (Integer a); VAtom (Integer b)] ->
      VAtom (Integer (Int64.mul a b))
  | Div, [VAtom (Integer a); VAtom (Integer b)] ->
      if Int64.equal b Int64.zero then
        VUnit (* Division by zero *)
      else
        VAtom (Integer (Int64.div a b))

  (* List operations *)
  | List, args ->
      VList args
  | Nth, [VList list; VAtom (Integer idx)] ->
      let idx_int = Int64.to_int idx in
      if idx_int < 0 || idx_int >= List.length list then
        VUnit (* Out of bounds *)
      else
        List.nth list idx_int
  | Length, [VList list] ->
      VAtom (Integer (Int64.of_int (List.length list)))
  | Cons, [head; VList tail] ->
      VList (head :: tail)
  | Car, [VList (head :: _)] ->
      head
  | Car, _ ->
      VUnit (* Empty list or not a list *)
  | Cdr, [VList (_ :: tail)] ->
      VList tail
  | Cdr, _ ->
      VList [] (* Empty list or not a list *)

  (* Map operations *)
  | MakeMap, [VList entries] ->
      let process_entry = function
        | VList [VAtom (String key); value] -> Some (key, value)
        | _ -> None
      in
      let map_entries = List.filter_map process_entry entries in
      VMap map_entries
  | MapGet, [VMap map; VAtom (String key)] ->
      (match List.assoc_opt key map with
       | Some value -> value
       | None -> VUnit)
  | MapHasKey, [VMap map; VAtom (String key)] ->
      VAtom (Boolean (List.mem_assoc key map))

  (* Definition and quoting *)
  | Define, _ ->
      (* Define will be handled by the evaluator *)
      VUnit
  | Defun, _ ->
      (* Defun will be handled by the evaluator *)
      VUnit
  | Quote, [expr] ->
      (* Quote returns its argument unevaluated *)
      expr

  (* Context access *)
  | GetContextValue, [VAtom (String _key)] ->
      (* Will be handled by the evaluator with context *)
      VUnit
  | GetField, [obj; VAtom (String field)] ->
      (* Field access will depend on the object type *)
      (match obj with
       | VMap map -> 
           (match List.assoc_opt field map with
            | Some value -> value
            | None -> VUnit)
       | _ -> VUnit)
  | Completed, _ ->
      (* Status check will be handled by execution context *)
      VAtom (Boolean false)

  (* Fallback for invalid combinations *)
  | _, _ ->
      VUnit (* Invalid combinator application *)

(* ------------ COMBINATOR UTILITIES ------------ *)

(** Check if a combinator requires special evaluation rules *)
let is_special_form = function
  | If | Let | LetStar | Define | Defun | Quote -> true
  | _ -> false

(** Get the expected number of arguments for a combinator *)
let expected_args = function
  | S -> 3
  | K -> 2
  | I -> 1
  | C -> 3
  | If -> 3
  | Let -> 2
  | LetStar -> 2
  | And -> -1 (* Variable *)
  | Or -> -1 (* Variable *)
  | Not -> 1
  | Eq -> 2
  | Gt -> 2
  | Lt -> 2
  | Gte -> 2
  | Lte -> 2
  | Add -> 2
  | Sub -> 2
  | Mul -> 2
  | Div -> 2
  | List -> -1 (* Variable *)
  | Nth -> 2
  | Length -> 1
  | Cons -> 2
  | Car -> 1
  | Cdr -> 1
  | MakeMap -> 1
  | MapGet -> 2
  | MapHasKey -> 2
  | Define -> 2
  | Defun -> 3
  | Quote -> 1
  | GetContextValue -> 1
  | GetField -> 2
  | Completed -> 0

(** Get the string representation of a combinator *)
let combinator_to_string = function
  | S -> "S"
  | K -> "K"
  | I -> "I"
  | C -> "C"
  | If -> "if"
  | Let -> "let"
  | LetStar -> "let*"
  | And -> "and"
  | Or -> "or"
  | Not -> "not"
  | Eq -> "="
  | Gt -> ">"
  | Lt -> "<"
  | Gte -> ">="
  | Lte -> "<="
  | Add -> "+"
  | Sub -> "-"
  | Mul -> "*"
  | Div -> "/"
  | List -> "list"
  | Nth -> "nth"
  | Length -> "length"
  | Cons -> "cons"
  | Car -> "car"
  | Cdr -> "cdr"
  | MakeMap -> "make-map"
  | MapGet -> "map-get"
  | MapHasKey -> "map-has-key"
  | Define -> "define"
  | Defun -> "defun"
  | Quote -> "quote"
  | GetContextValue -> "get-context-value"
  | GetField -> "get-field"
  | Completed -> "completed" 