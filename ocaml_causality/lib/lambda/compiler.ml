(** Layer 1 to Layer 0 Compiler

    This module compiles content-addressed linear lambda calculus terms to
    register machine instructions, providing the bridge between the functional
    and imperative layers. *)

open Term
open Causality_machine
open Causality_system.System_errors
open Causality_system.System_content_addressing

(** Helper for monadic operations *)
let ( >>= ) = Result.bind

(** {1 Compilation Context} *)

module VarMap = Map.Make (String)
(** Variable to register mapping *)

type compile_context = {
    next_reg : int32
  ; var_map : int32 VarMap.t
  ; labels : string list
  ; store : ExpressionStore.t
}
(** Compilation context *)

(** Create empty compilation context *)
let empty_context (store : ExpressionStore.t) : compile_context =
  {
    next_reg = 1l
  ; (* Start from register 1, reserve 0 for special use *)
    var_map = VarMap.empty
  ; labels = []
  ; store
  }

(** Allocate a new register *)
let alloc_register (ctx : compile_context) : compile_context * int32 =
  let reg = ctx.next_reg in
  let new_ctx = { ctx with next_reg = Int32.add ctx.next_reg 1l } in
  (new_ctx, reg)

(** Bind variable to register *)
let bind_var (ctx : compile_context) (var : string) (reg : int32) :
    compile_context =
  { ctx with var_map = VarMap.add var reg ctx.var_map }

(** Look up variable register *)
let lookup_var (ctx : compile_context) (var : string) : int32 option =
  VarMap.find_opt var ctx.var_map

(** Generate unique label *)
let gen_label (ctx : compile_context) (prefix : string) :
    compile_context * string =
  let label = prefix ^ "_" ^ string_of_int (List.length ctx.labels) in
  let new_ctx = { ctx with labels = label :: ctx.labels } in
  (new_ctx, label)

(** {1 Compilation Functions} *)

(** Compile content-addressed expression to register machine instructions *)
let rec compile_expression (ctx : compile_context) (expr_id : expr_id)
    (target_reg : int32) :
    (compile_context * Instruction.instruction list, error_kind) result =
  match ExpressionStore.retrieve ctx.store expr_id with
  | None -> Error (TypeError ("Unknown expression: " ^ EntityId.to_hex expr_id))
  | Some expr -> (
      match expr.content with
      (* Core primitive compilation *)
      | Core Unit ->
          (* Load unit value into target register *)
          Ok (ctx, [])
          (* Simplified - would need LoadImmediate instruction *)
      | Core (LetUnit (e1, e2)) ->
          let ctx1, temp_reg = alloc_register ctx in
          compile_expression ctx1 e1 temp_reg >>= fun (ctx2, instrs1) ->
          compile_expression ctx2 e2 target_reg >>= fun (ctx3, instrs2) ->
          Ok (ctx3, instrs1 @ instrs2)
      | Core (Tensor (e1, e2)) ->
          let ctx1, reg1 = alloc_register ctx in
          let ctx2, reg2 = alloc_register ctx1 in
          compile_expression ctx2 e1 reg1 >>= fun (ctx3, instrs1) ->
          compile_expression ctx3 e2 reg2 >>= fun (ctx4, instrs2) ->
          (* Create tensor using built-in operation *)
          let tensor_instr =
            Instruction.Apply
              {
                fn_reg = Instruction.RegisterId.create 0l
              ; (* Built-in tensor function *)
                arg_reg = Instruction.RegisterId.create reg1
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in
          Ok (ctx4, instrs1 @ instrs2 @ [ tensor_instr ])
      | Core (LetTensor (pair_expr, body_expr)) ->
          let ctx1, pair_reg = alloc_register ctx in
          let ctx2, left_reg = alloc_register ctx1 in
          let ctx3, right_reg = alloc_register ctx2 in
          compile_expression ctx3 pair_expr pair_reg >>= fun (ctx4, instrs1) ->
          (* Destructure the pair *)
          let destructure_instr =
            Instruction.Match
              {
                sum_reg = Instruction.RegisterId.create pair_reg
              ; left_reg = Instruction.RegisterId.create left_reg
              ; right_reg = Instruction.RegisterId.create right_reg
              ; left_label = "tensor_left"
              ; right_label = "tensor_right"
              }
          in

          compile_expression ctx4 body_expr target_reg
          >>= fun (ctx5, instrs2) ->
          Ok (ctx5, instrs1 @ [ destructure_instr ] @ instrs2)
      | Core (Inl e) ->
          let ctx1, val_reg = alloc_register ctx in
          compile_expression ctx1 e val_reg >>= fun (ctx2, instrs) ->
          let inl_instr =
            Instruction.Apply
              {
                fn_reg = Instruction.RegisterId.create 0l
              ; (* Built-in inl function *)
                arg_reg = Instruction.RegisterId.create val_reg
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in
          Ok (ctx2, instrs @ [ inl_instr ])
      | Core (Inr e) ->
          let ctx1, val_reg = alloc_register ctx in
          compile_expression ctx1 e val_reg >>= fun (ctx2, instrs) ->
          let inr_instr =
            Instruction.Apply
              {
                fn_reg = Instruction.RegisterId.create 0l
              ; (* Built-in inr function *)
                arg_reg = Instruction.RegisterId.create val_reg
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in
          Ok (ctx2, instrs @ [ inr_instr ])
      | Core (Case (scrutinee, left_branch, right_branch)) ->
          let ctx1, sum_reg = alloc_register ctx in
          let ctx2, left_reg = alloc_register ctx1 in
          let ctx3, right_reg = alloc_register ctx2 in
          let ctx4, left_label = gen_label ctx3 "case_left" in
          let ctx5, right_label = gen_label ctx4 "case_right" in
          let ctx6, end_label = gen_label ctx5 "case_end" in

          compile_expression ctx6 scrutinee sum_reg
          >>= fun (ctx7, scrutinee_instrs) ->
          let match_instr =
            Instruction.Match
              {
                sum_reg = Instruction.RegisterId.create sum_reg
              ; left_reg = Instruction.RegisterId.create left_reg
              ; right_reg = Instruction.RegisterId.create right_reg
              ; left_label
              ; right_label
              }
          in

          let left_label_instr = Instruction.LabelMarker left_label in
          compile_expression ctx7 left_branch target_reg
          >>= fun (ctx8, left_instrs) ->
          let right_label_instr = Instruction.LabelMarker right_label in
          compile_expression ctx8 right_branch target_reg
          >>= fun (ctx9, right_instrs) ->
          let end_label_instr = Instruction.LabelMarker end_label in

          let all_instrs =
            scrutinee_instrs
            @ [ match_instr; left_label_instr ]
            @ left_instrs @ [ right_label_instr ] @ right_instrs
            @ [ end_label_instr ]
          in

          Ok (ctx9, all_instrs)
      | Core (Lambda (_, _)) ->
          (* Create a function closure - simplified for now *)
          Ok (ctx, [])
          (* Would need function table and closure creation *)
      | Core (Apply (fn_expr, arg_expr)) ->
          let ctx1, fn_reg = alloc_register ctx in
          let ctx2, arg_reg = alloc_register ctx1 in
          compile_expression ctx2 fn_expr fn_reg >>= fun (ctx3, fn_instrs) ->
          compile_expression ctx3 arg_expr arg_reg >>= fun (ctx4, arg_instrs) ->
          let apply_instr =
            Instruction.Apply
              {
                fn_reg = Instruction.RegisterId.create fn_reg
              ; arg_reg = Instruction.RegisterId.create arg_reg
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in
          Ok (ctx4, fn_instrs @ arg_instrs @ [ apply_instr ])
      | Core (Alloc e) ->
          let ctx1, val_reg = alloc_register ctx in
          let ctx2, type_reg = alloc_register ctx1 in
          compile_expression ctx2 e val_reg >>= fun (ctx3, instrs) ->
          let alloc_instr =
            Instruction.Alloc
              {
                type_reg = Instruction.RegisterId.create type_reg
              ; val_reg = Instruction.RegisterId.create val_reg
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in
          Ok (ctx3, instrs @ [ alloc_instr ])
      | Core (Consume e) ->
          let ctx1, res_reg = alloc_register ctx in
          compile_expression ctx1 e res_reg >>= fun (ctx2, instrs) ->
          let consume_instr =
            Instruction.Consume
              {
                resource_reg = Instruction.RegisterId.create res_reg
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in
          Ok (ctx2, instrs @ [ consume_instr ])
      (* Extended language features *)
      | Symbol _ ->
          (* Load symbol as immediate value - simplified *)
          Ok (ctx, [])
      | Int _ ->
          (* Load integer as immediate value - simplified *)
          Ok (ctx, [])
      | Bool _ ->
          (* Load boolean as immediate value - simplified *)
          Ok (ctx, [])
      | Let (var, value_expr, body_expr) ->
          let ctx1, val_reg = alloc_register ctx in
          compile_expression ctx1 value_expr val_reg
          >>= fun (ctx2, val_instrs) ->
          let ctx3 = bind_var ctx2 var val_reg in
          compile_expression ctx3 body_expr target_reg
          >>= fun (ctx4, body_instrs) -> Ok (ctx4, val_instrs @ body_instrs)
      | If (cond_expr, then_expr, else_expr) ->
          let ctx1, cond_reg = alloc_register ctx in

          compile_expression ctx1 cond_expr cond_reg
          >>= fun (ctx5, cond_instrs) ->
          (* Conditional branch using Select instruction *)
          let ctx6, then_reg = alloc_register ctx5 in
          let ctx7, else_reg = alloc_register ctx6 in

          compile_expression ctx7 then_expr then_reg
          >>= fun (ctx8, then_instrs) ->
          compile_expression ctx8 else_expr else_reg
          >>= fun (ctx9, else_instrs) ->
          let select_instr =
            Instruction.Select
              {
                cond_reg = Instruction.RegisterId.create cond_reg
              ; true_reg = Instruction.RegisterId.create then_reg
              ; false_reg = Instruction.RegisterId.create else_reg
              ; out_reg = Instruction.RegisterId.create target_reg
              }
          in

          Ok (ctx9, cond_instrs @ then_instrs @ else_instrs @ [ select_instr ])
      | _ ->
          (* Other extended features not yet implemented *)
          Error (TypeError "Extended language feature not yet implemented"))

(** {1 Top-level Compilation Interface} *)

(** Compile a content-addressed expression to machine instructions *)
let compile (store : ExpressionStore.t) (expr_id : expr_id) :
    (Instruction.instruction list, error_kind) result =
  let ctx = empty_context store in
  let target_reg = 0l in
  (* Main result register *)
  compile_expression ctx expr_id target_reg >>= fun (_, instrs) -> Ok instrs

(** Compile and pretty-print instructions *)
let compile_and_show (store : ExpressionStore.t) (expr_id : expr_id) : string =
  match compile store expr_id with
  | Ok instrs ->
      "Compiled successfully:\n"
      ^ String.concat "\n"
          (List.mapi
             (fun i _ ->
               Printf.sprintf "%d: %s" i
                 "instruction" (* Would need pretty printer for instructions *))
             instrs)
  | Error err -> "Compilation failed: " ^ string_of_error_kind err
