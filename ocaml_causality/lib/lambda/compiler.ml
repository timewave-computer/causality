(** Lambda calculus compiler to Layer 0 register machine *)
(** Purpose: Compile content-addressed lambda terms to 5-instruction register machine *)

open Causality_machine

(** {1 Error Handling} *)

type error_kind = TypeError of string | CompilationError of string

let string_of_error_kind = function
  | TypeError msg -> "Type error: " ^ msg
  | CompilationError msg -> "Compilation error: " ^ msg

(** {1 Compilation Context} *)

(** Compilation context tracks register allocation *)
type compile_context = {
  next_register : int32;
}

(** Create empty compilation context *)
let empty_context () : compile_context =
  { next_register = 1l }

(** Allocate a fresh register *)
let alloc_register (ctx : compile_context) : compile_context * int32 =
  let reg = ctx.next_register in
  let ctx' = { next_register = Int32.add ctx.next_register 1l } in
  (ctx', reg)

(** {1 Compilation Functions} *)

(** Compile any term to a simple allocation instruction *)
let compile_any_term (ctx : compile_context) (target_reg : int32) :
    compile_context * Instruction.instruction list =
  (* Simplified compilation - just allocate a value *)
  let ctx1, type_reg = alloc_register ctx in
  let ctx2, init_reg = alloc_register ctx1 in
  let alloc_instr =
    Instruction.Alloc
      {
        type_reg = Instruction.RegisterId.create type_reg
      ; init_reg = Instruction.RegisterId.create init_reg
      ; output_reg = Instruction.RegisterId.create target_reg
      }
  in
  (ctx2, [ alloc_instr ])

(** {1 Top-level Compilation Interface} *)

(** Compile any term to machine instructions *)
let compile_to_instructions (_term : 'a) : Instruction.instruction list * int32 =
  let ctx = empty_context () in
  let target_reg = 0l in
  let (_, instrs) = compile_any_term ctx target_reg in
  (instrs, target_reg)

(** Compile and pretty-print instructions *)
let compile_and_show (_term : 'a) : string =
  let (instrs, _) = compile_to_instructions _term in
  "Compiled successfully:\n"
  ^ String.concat "\n"
      (List.mapi
         (fun i _instr ->
           Printf.sprintf "%d: %s" i "<instruction>")
         instrs)
