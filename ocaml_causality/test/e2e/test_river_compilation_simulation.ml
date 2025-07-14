(* E2E test for the new file-based compilation workflow (CLI only) *)

open Printf

(* Helper function to write to a file *)
let write_to_file path content =
  let oc = open_out path in
  fprintf oc "%s" content;
  close_out oc

(* Helper function to read from a file *)
let read_from_file path =
  let ic = open_in_bin path in
  let len = in_channel_length ic in
  let content = really_input_string ic len in
  close_in ic;
  content

(* Helper function to run the causality-cli *)
let run_causality_cli input_file output_file =
  let command = sprintf "causality compile -i %s -o %s" input_file output_file in
  let exit_code = Sys.command command in
  if exit_code <> 0 then
    failwith (sprintf "causality-cli failed with exit code %d" exit_code)

(* A simple Lisp program for testing *)
let test_lisp_program = "(pure 42)"

(* Main test runner *)
let run_cli_compilation_test () =
  printf "Starting CLI Compilation Test\n";
  printf "=============================\n";

  let sx_file = "test.sx" in
  let bc_file = "test.bc" in

  try
    (* 1. Write Lisp program to .sx file *)
    printf "1. Writing Lisp program to %s...\n" sx_file;
    write_to_file sx_file test_lisp_program;

    (* 2. Compile .sx to .bc using causality-cli *)
    printf "2. Compiling with causality-cli...\n";
    run_causality_cli sx_file bc_file;
    printf "   Compilation successful.\n";

    (* 3. Read bytecode from .bc file *)
    printf "3. Reading bytecode from %s...\n" bc_file;
    let bytecode = read_from_file bc_file in
    printf "   Read %d bytes of bytecode.\n" (String.length bytecode);

    (* 4. Validate bytecode is not empty *)
    if String.length bytecode = 0 then
      failwith "Bytecode file is empty";
    printf "   Bytecode validation passed.\n";

    printf "\n✅ CLI Compilation Test Passed!\n";
    (true, "CLI Compilation Test")
  with
  | ex ->
    printf "\n❌ Test Failed: %s\n" (Printexc.to_string ex);
    (false, "Test failed")

(* Cleanup function *)
let cleanup () =
  let files = ["test.sx"; "test.bc"] in
  List.iter (fun file ->
    if Sys.file_exists file then (
      Sys.remove file;
      printf "Cleaned up %s\n" file
    )
  ) files

let () =
  let (passed, msg) = run_cli_compilation_test () in
  cleanup ();
  if passed then
    printf "[SUCCESS] %s\n" msg
  else
    printf "[FAILURE] %s\n" msg;
  exit (if passed then 0 else 1)
