(* ------------ S-EXPRESSION SERIALIZATION ------------ *)
(* Purpose: S-expression serialization for Causality types *)

(* ------------ TYPE DEFINITIONS ------------ *)

(** S-expression types *)
type sexpr =
  | Atom of string
  | List of sexpr list
  | String of string
  | Number of int
  | Symbol of string

(** Pretty printing options *)
type print_options = {
  indent : int;
  max_width : int;
  compact : bool;
}

let default_print_options = {
  indent = 2;
  max_width = 80;
  compact = false;
}

(* ------------ PARSING ------------ *)

(** Parse error type *)
exception ParseError of string * int (* message, position *)

(** Tokenizer for S-expressions *)
type token =
  | LeftParen
  | RightParen
  | AtomToken of string
  | StringToken of string
  | NumberToken of int
  | EOF

(** Tokenize a string into tokens *)
let tokenize (input : string) : token list =
  let len = String.length input in
  let rec tokenize_loop pos acc =
    if pos >= len then List.rev (EOF :: acc)
    else
      let c = input.[pos] in
      match c with
      | ' ' | '\t' | '\n' | '\r' -> tokenize_loop (pos + 1) acc
      | '(' -> tokenize_loop (pos + 1) (LeftParen :: acc)
      | ')' -> tokenize_loop (pos + 1) (RightParen :: acc)
      | '"' -> 
          let end_pos = String.index_from input (pos + 1) '"' in
          let str_content = String.sub input (pos + 1) (end_pos - pos - 1) in
          tokenize_loop (end_pos + 1) (StringToken str_content :: acc)
      | '0'..'9' | '-' ->
          let rec read_number start_pos =
            if start_pos >= len then start_pos
            else
              match input.[start_pos] with
              | '0'..'9' | '-' -> read_number (start_pos + 1)
              | _ -> start_pos
          in
          let end_pos = read_number pos in
          let num_str = String.sub input pos (end_pos - pos) in
          let num = int_of_string num_str in
          tokenize_loop end_pos (NumberToken num :: acc)
      | _ ->
          let rec read_atom start_pos =
            if start_pos >= len then start_pos
            else
              match input.[start_pos] with
              | ' ' | '\t' | '\n' | '\r' | '(' | ')' | '"' -> start_pos
              | _ -> read_atom (start_pos + 1)
          in
          let end_pos = read_atom pos in
          let atom_str = String.sub input pos (end_pos - pos) in
          tokenize_loop end_pos (AtomToken atom_str :: acc)
  in
  try
    tokenize_loop 0 []
  with
  | Not_found -> raise (ParseError ("Unterminated string", 0))
  | Invalid_argument _ -> raise (ParseError ("Invalid number", 0))

(** Parse tokens into S-expressions *)
let parse (tokens : token list) : sexpr =
  let rec parse_sexpr tokens =
    match tokens with
    | [] -> raise (ParseError ("Unexpected end of input", 0))
    | EOF :: _ -> raise (ParseError ("Unexpected end of input", 0))
    | LeftParen :: rest ->
        let (items, remaining) = parse_list rest [] in
        (List items, remaining)
    | RightParen :: _ -> raise (ParseError ("Unexpected closing parenthesis", 0))
    | AtomToken s :: rest -> (Atom s, rest)
    | StringToken s :: rest -> (String s, rest)
    | NumberToken n :: rest -> (Number n, rest)
  
  and parse_list tokens acc =
    match tokens with
    | [] -> raise (ParseError ("Unterminated list", 0))
    | EOF :: _ -> raise (ParseError ("Unterminated list", 0))
    | RightParen :: rest -> (List.rev acc, rest)
    | _ ->
        let (item, remaining) = parse_sexpr tokens in
        parse_list remaining (item :: acc)
  in
  
  let (result, remaining) = parse_sexpr tokens in
  match remaining with
  | [EOF] -> result
  | [] -> result
  | _ -> raise (ParseError ("Extra tokens after expression", 0))

(** Parse a string into an S-expression *)
let parse_string (input : string) : sexpr =
  let tokens = tokenize input in
  parse tokens

(* ------------ GENERATION ------------ *)

(** Convert S-expression to string *)
let to_string ?(options = default_print_options) (expr : sexpr) : string =
  let rec to_string_helper depth expr =
    match expr with
    | Atom s -> s
    | Symbol s -> s
    | String s -> "\"" ^ s ^ "\""
    | Number n -> string_of_int n
    | List items ->
        let indent_str = if options.compact then "" else String.make (depth * options.indent) ' ' in
        let items_str = List.map (to_string_helper (depth + 1)) items in
        if options.compact then
          "(" ^ String.concat " " items_str ^ ")"
        else
          let total_length = List.fold_left (fun acc s -> acc + String.length s + 1) 0 items_str in
          if total_length <= options.max_width then
            "(" ^ String.concat " " items_str ^ ")"
          else
            "(" ^ String.concat ("\n" ^ indent_str) items_str ^ ")"
  in
  to_string_helper 0 expr

(** Convert various OCaml types to S-expressions *)
let of_int (n : int) : sexpr = Number n
let of_string (s : string) : sexpr = String s
let of_atom (s : string) : sexpr = Atom s
let of_symbol (s : string) : sexpr = Symbol s
let of_list (items : sexpr list) : sexpr = List items

(** Convert S-expression back to OCaml types *)
let to_int = function
  | Number n -> Some n
  | _ -> None

let to_string_value = function
  | String s -> Some s
  | _ -> None

let to_atom = function
  | Atom s -> Some s
  | _ -> None

let to_symbol = function
  | Symbol s -> Some s
  | _ -> None

let to_list = function
  | List items -> Some items
  | _ -> None

(* ------------ UTILITIES ------------ *)

(** Check if S-expression is an atom *)
let is_atom = function
  | Atom _ -> true
  | _ -> false

(** Check if S-expression is a list *)
let is_list = function
  | List _ -> true
  | _ -> false

(** Get the length of a list S-expression *)
let list_length = function
  | List items -> Some (List.length items)
  | _ -> None

(** Get the nth element of a list S-expression *)
let list_nth (expr : sexpr) (n : int) : sexpr option =
  match expr with
  | List items -> 
      if n >= 0 && n < List.length items then
        Some (List.nth items n)
      else
        None
  | _ -> None

(** Map a function over a list S-expression *)
let list_map (f : sexpr -> sexpr) = function
  | List items -> List (List.map f items)
  | other -> other

(** Fold over a list S-expression *)
let list_fold_left (f : 'a -> sexpr -> 'a) (acc : 'a) = function
  | List items -> List.fold_left f acc items
  | _ -> acc

(** Find an element in a list S-expression *)
let list_find (predicate : sexpr -> bool) = function
  | List items -> 
      (try Some (List.find predicate items)
       with Not_found -> None)
  | _ -> None

(** Convert Causality types to S-expressions *)
let entity_id_to_sexpr (id : bytes) : sexpr =
  let hex_string = Bytes.to_string id in
  List [Atom "entity-id"; String hex_string]

let timestamp_to_sexpr (ts : int64) : sexpr =
  List [Atom "timestamp"; Number (Int64.to_int ts)]

(** Convert S-expressions back to Causality types *)
let sexpr_to_entity_id = function
  | List [Atom "entity-id"; String hex_string] -> 
      Some (Bytes.of_string hex_string)
  | _ -> None

let sexpr_to_timestamp = function
  | List [Atom "timestamp"; Number n] -> 
      Some (Int64.of_int n)
  | _ -> None

(** Pretty print with custom formatting *)
let pretty_print ?(indent = 2) ?(max_width = 80) (expr : sexpr) : string =
  let options = { indent; max_width; compact = false } in
  to_string ~options expr

(** Compact print (single line) *)
let compact_print (expr : sexpr) : string =
  let options = { default_print_options with compact = true } in
  to_string ~options expr
