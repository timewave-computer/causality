(* Purpose: DSL compilation framework with clear separation of concerns *)

open Ocaml_causality_core

(** Compilation phases *)
type compilation_phase = 
  | Parse
  | Validate  
  | Optimize
  | Generate

(** Compilation context *)
type compilation_context = {
  phase: compilation_phase;
  domain_id: Identifiers.domain_id;
  optimization_level: int;
  target_backend: string;
  metadata: (string * string) list;
}

(** Schema definition for DSL structures *)
type schema_field = {
  field_name: string;
  field_type: string;
  required: bool;
  default_value: string option;
}

type schema_definition = {
  schema_id: string;
  schema_name: string;
  version: string;
  fields: schema_field list;
  constraints: string list;
}

(** Compilation result *)
type compilation_result = {
  success: bool;
  output: string;
  errors: string list;
  warnings: string list;
  artifacts: (string * string) list; (* name, content *)
}

(** AST transformation rules *)
type transformation_rule = {
  rule_id: string;
  rule_name: string;
  pattern: string;
  replacement: string;
  conditions: string list;
}

(** Parse DSL source code into AST *)
val parse_dsl : string -> compilation_context -> (Ast.expr, string) result

(** Validate parsed AST against schema *)
val validate_ast : Ast.expr -> schema_definition list -> (bool * string list)

(** Apply optimization transformations *)
val optimize_ast : Ast.expr -> transformation_rule list -> Ast.expr

(** Generate code from optimized AST *)
val generate_code : Ast.expr -> compilation_context -> compilation_result

(** Create schema definition *)
val create_schema : 
  schema_id:string ->
  schema_name:string ->
  version:string ->
  fields:schema_field list ->
  constraints:string list ->
  schema_definition

(** Validate schema compatibility *)
val validate_schema_compatibility : schema_definition -> schema_definition -> bool

(** Create compilation context *)
val create_compilation_context :
  domain_id:Identifiers.domain_id ->
  optimization_level:int ->
  target_backend:string ->
  compilation_context

(** Full compilation pipeline *)
val compile_dsl :
  source:string ->
  schemas:schema_definition list ->
  context:compilation_context ->
  compilation_result 