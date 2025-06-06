(** Layer 1: Content-Addressed Lambda Calculus Terms
    
    This module provides content-addressed linear lambda calculus with
    the 11 core primitives and extended AST for practical programming.
*)

open Causality_system.System_content_addressing

(** {1 Content-Addressed Expression System} *)

(** Content-addressed expression identifier *)
type expr_id = EntityId.t

(** Core linear lambda calculus primitives *)
type core_term =
  (* Unit type operations *)
  | Unit                                      (* Unit introduction *)
  | LetUnit of expr_id * expr_id             (* Unit elimination: let () = e1 in e2 *)
  
  (* Tensor product (pairs) *)
  | Tensor of expr_id * expr_id              (* Pair creation: (e1, e2) *)
  | LetTensor of expr_id * expr_id           (* Pair elimination: let (x,y) = e1 in e2 *)
  
  (* Sum types (disjoint union) *)
  | Inl of expr_id                           (* Left injection *)
  | Inr of expr_id                           (* Right injection *)
  | Case of expr_id * expr_id * expr_id      (* Pattern matching: case e of inl x => e1 | inr y => e2 *)
  
  (* Linear functions *)
  | Lambda of string * expr_id               (* Function definition: λx. e *)
  | Apply of expr_id * expr_id               (* Function application: e1 e2 *)
  
  (* Resource management *)
  | Alloc of expr_id                         (* Resource allocation *)
  | Consume of expr_id                       (* Resource consumption *)

(** Extended AST for practical programming *)
type extended_term =
  (* Core primitives *)
  | Core of core_term
  
  (* Row type operations *)
  | ReadField of expr_id * string            (* Record field access *)
  | UpdateField of expr_id * string * expr_id (* Record field update *)
  | Project of expr_id * string list         (* Row projection *)
  | Restrict of expr_id * string list        (* Row restriction *)
  | Extend of expr_id * string * expr_id     (* Row extension *)
  | Diff of expr_id * string list            (* Row difference *)
  
  (* Convenience forms *)
  | Symbol of string                          (* Symbol literal *)
  | Int of int                               (* Integer literal *)
  | Bool of bool                             (* Boolean literal *)
  | Quote of expr_id                         (* Quoted expression *)
  | List of expr_id list                     (* List construction *)
  | Let of string * expr_id * expr_id        (* Local binding *)
  | If of expr_id * expr_id * expr_id        (* Conditional *)

(** Content-addressed expression with structural sharing *)
type expression = {
  content : extended_term;
  expr_id : expr_id;
  sub_expressions : expr_id list;
}

(** Helper function to extract sub-expression IDs *)
let get_sub_expressions (term : extended_term) : expr_id list =
  match term with
  | Core (Unit) -> []
  | Core (LetUnit (e1, e2)) -> [e1; e2]
  | Core (Tensor (e1, e2)) -> [e1; e2]
  | Core (LetTensor (e1, e2)) -> [e1; e2]
  | Core (Inl e) -> [e]
  | Core (Inr e) -> [e]
  | Core (Case (e, e1, e2)) -> [e; e1; e2]
  | Core (Lambda (_, e)) -> [e]
  | Core (Apply (e1, e2)) -> [e1; e2]
  | Core (Alloc e) -> [e]
  | Core (Consume e) -> [e]
  | ReadField (e, _) -> [e]
  | UpdateField (e1, _, e2) -> [e1; e2]
  | Project (e, _) -> [e]
  | Restrict (e, _) -> [e]
  | Extend (e1, _, e2) -> [e1; e2]
  | Diff (e, _) -> [e]
  | Symbol _ | Int _ | Bool _ -> []
  | Quote e -> [e]
  | List exprs -> exprs
  | Let (_, e1, e2) -> [e1; e2]
  | If (e1, e2, e3) -> [e1; e2; e3]

(** {1 Expression Store for Content Addressing} *)

(** Expression store for content addressing *)
module ExpressionStore = struct
  type t = (expr_id, expression) Hashtbl.t
  
  let create () : t = Hashtbl.create 1024
  
  let store (store : t) (content : extended_term) : expr_id =
    let expr_id = EntityId.from_content content in
    let sub_expressions = get_sub_expressions content in
    let expr = { content; expr_id; sub_expressions } in
    Hashtbl.replace store expr_id expr;
    expr_id
  
  let retrieve (store : t) (id : expr_id) : expression option =
    Hashtbl.find_opt store id
  
  let contains (store : t) (id : expr_id) : bool =
    Hashtbl.mem store id
end

(** {1 Smart Constructors} *)

(** Smart constructors for building content-addressed expressions *)
module Term = struct
  (** Core primitive constructors *)
  let unit (store : ExpressionStore.t) : expr_id =
    ExpressionStore.store store (Core Unit)
  
  let let_unit (store : ExpressionStore.t) (e1 : expr_id) (e2 : expr_id) : expr_id =
    ExpressionStore.store store (Core (LetUnit (e1, e2)))
  
  let tensor (store : ExpressionStore.t) (e1 : expr_id) (e2 : expr_id) : expr_id =
    ExpressionStore.store store (Core (Tensor (e1, e2)))
  
  let let_tensor (store : ExpressionStore.t) (e1 : expr_id) (e2 : expr_id) : expr_id =
    ExpressionStore.store store (Core (LetTensor (e1, e2)))
  
  let inl (store : ExpressionStore.t) (e : expr_id) : expr_id =
    ExpressionStore.store store (Core (Inl e))
  
  let inr (store : ExpressionStore.t) (e : expr_id) : expr_id =
    ExpressionStore.store store (Core (Inr e))
  
  let case (store : ExpressionStore.t) (e : expr_id) (e1 : expr_id) (e2 : expr_id) : expr_id =
    ExpressionStore.store store (Core (Case (e, e1, e2)))
  
  let lambda (store : ExpressionStore.t) (var : string) (body : expr_id) : expr_id =
    ExpressionStore.store store (Core (Lambda (var, body)))
  
  let apply (store : ExpressionStore.t) (fn : expr_id) (arg : expr_id) : expr_id =
    ExpressionStore.store store (Core (Apply (fn, arg)))
  
  let alloc (store : ExpressionStore.t) (e : expr_id) : expr_id =
    ExpressionStore.store store (Core (Alloc e))
  
  let consume (store : ExpressionStore.t) (e : expr_id) : expr_id =
    ExpressionStore.store store (Core (Consume e))
  
  (** Convenience constructors *)
  let symbol (store : ExpressionStore.t) (s : string) : expr_id =
    ExpressionStore.store store (Symbol s)
  
  let int (store : ExpressionStore.t) (i : int) : expr_id =
    ExpressionStore.store store (Int i)
  
  let bool (store : ExpressionStore.t) (b : bool) : expr_id =
    ExpressionStore.store store (Bool b)
  
  let if_then_else (store : ExpressionStore.t) (cond : expr_id) (then_branch : expr_id) (else_branch : expr_id) : expr_id =
    ExpressionStore.store store (If (cond, then_branch, else_branch))
  
  let let_bind (store : ExpressionStore.t) (var : string) (value : expr_id) (body : expr_id) : expr_id =
    ExpressionStore.store store (Let (var, value, body))
  
  let list (store : ExpressionStore.t) (exprs : expr_id list) : expr_id =
    ExpressionStore.store store (List exprs)
  
  (** Row type operations *)
  let read_field (store : ExpressionStore.t) (record : expr_id) (field : string) : expr_id =
    ExpressionStore.store store (ReadField (record, field))
  
  let update_field (store : ExpressionStore.t) (record : expr_id) (field : string) (value : expr_id) : expr_id =
    ExpressionStore.store store (UpdateField (record, field, value))
  
  let extend_record (store : ExpressionStore.t) (record : expr_id) (field : string) (value : expr_id) : expr_id =
    ExpressionStore.store store (Extend (record, field, value))
end

(** {1 Analysis and Utilities} *)

(** Free variable analysis *)
module FreeVars = struct
  let rec free_vars (store : ExpressionStore.t) (expr_id : expr_id) : string list =
    match ExpressionStore.retrieve store expr_id with
    | None -> []
    | Some expr ->
      match expr.content with
      | Core (Lambda (var, body)) ->
        List.filter (fun v -> not (String.equal v var)) (free_vars store body)
      | Let (var, value, body) ->
        free_vars store value @ 
        List.filter (fun v -> not (String.equal v var)) (free_vars store body)
      | _ -> 
        (* For other expressions, collect free variables from sub-expressions *)
        List.concat_map (free_vars store) expr.sub_expressions
        |> List.sort_uniq String.compare
end

(** Pretty printing *)
module Pretty = struct
  let rec term_to_string (store : ExpressionStore.t) (expr_id : expr_id) : string =
    match ExpressionStore.retrieve store expr_id with
    | None -> Printf.sprintf "Unknown(%s)" (EntityId.to_hex expr_id)
    | Some expr ->
      match expr.content with
      | Core Unit -> "()"
      | Core (LetUnit (e1, e2)) -> 
        Printf.sprintf "(let () = %s in %s)" 
          (term_to_string store e1) (term_to_string store e2)
      | Core (Tensor (e1, e2)) -> 
        Printf.sprintf "(%s, %s)" 
          (term_to_string store e1) (term_to_string store e2)
      | Core (Lambda (var, body)) -> 
        Printf.sprintf "(λ%s. %s)" var (term_to_string store body)
      | Core (Apply (fn, arg)) -> 
        Printf.sprintf "(%s %s)" 
          (term_to_string store fn) (term_to_string store arg)
      | Symbol s -> s
      | Int i -> string_of_int i
      | Bool b -> string_of_bool b
      | Let (var, value, body) ->
        Printf.sprintf "(let %s = %s in %s)"
          var (term_to_string store value) (term_to_string store body)
      | If (cond, then_branch, else_branch) ->
        Printf.sprintf "(if %s then %s else %s)"
          (term_to_string store cond) (term_to_string store then_branch) (term_to_string store else_branch)
      | _ -> "complex_term"
end

(** {1 Compilation Helpers} *)

(** The 11 Core Lambda Calculus Primitives for Layer 0 compilation *)
module CorePrimitives = struct
  
  (** 1. Unit: Unit type introduction *)
  let unit_intro (store : ExpressionStore.t) : expr_id =
    Term.unit store
  
  (** 2. LetUnit: Unit type elimination (sequencing) *)
  let unit_elim (store : ExpressionStore.t) (computation : expr_id) (continuation : expr_id) : expr_id =
    Term.let_unit store computation continuation
  
  (** 3. Tensor: Product type introduction *)
  let product_intro (store : ExpressionStore.t) (left : expr_id) (right : expr_id) : expr_id =
    Term.tensor store left right
  
  (** 4. LetTensor: Product type elimination *)
  let product_elim (store : ExpressionStore.t) (pair : expr_id) (body : expr_id) : expr_id =
    Term.let_tensor store pair body
  
  (** 5. Inl: Sum type introduction (left) *)
  let sum_intro_left (store : ExpressionStore.t) (term : expr_id) : expr_id =
    Term.inl store term
  
  (** 6. Inr: Sum type introduction (right) *)
  let sum_intro_right (store : ExpressionStore.t) (term : expr_id) : expr_id =
    Term.inr store term
  
  (** 7. Case: Sum type elimination *)
  let sum_elim (store : ExpressionStore.t) (scrutinee : expr_id) (left_branch : expr_id) (right_branch : expr_id) : expr_id =
    Term.case store scrutinee left_branch right_branch
  
  (** 8. Lambda: Linear function introduction *)
  let function_intro (store : ExpressionStore.t) (param : string) (body : expr_id) : expr_id =
    Term.lambda store param body
  
  (** 9. Apply: Linear function elimination *)
  let function_elim (store : ExpressionStore.t) (func : expr_id) (arg : expr_id) : expr_id =
    Term.apply store func arg
  
  (** 10. Alloc: Resource allocation *)
  let resource_alloc (store : ExpressionStore.t) (value : expr_id) : expr_id =
    Term.alloc store value
  
  (** 11. Consume: Resource consumption *)
  let resource_consume (store : ExpressionStore.t) (resource : expr_id) : expr_id =
    Term.consume store resource
end 