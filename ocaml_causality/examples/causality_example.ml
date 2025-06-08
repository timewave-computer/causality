(* ------------ CAUSALITY OCAML INTEGRATION EXAMPLE ------------ *)
(* Purpose: Demonstrates OCaml integration with Causality framework *)

open Ocaml_causality_core
open Ocaml_causality_interop.Bindings

let () =
  Printf.printf "=== Causality OCaml Integration Example ===\n\n";

  (* Initialize the FFI subsystem *)
  (match Ocaml_causality_interop.Ffi.initialize_ffi () with
  | Ok () -> Printf.printf "FFI initialized successfully\n"
  | Error _err ->
      Printf.printf "FFI initialization failed\n";
      exit 1);

  (* Example 1: Working with LispValues *)
  Printf.printf "\n--- LispValue Examples ---\n";
  let values =
    [
      LispValue.unit
    ; LispValue.bool true
    ; LispValue.int 42L
    ; LispValue.string "Hello Causality"
    ; LispValue.symbol "test-symbol"
    ; LispValue.list [ LispValue.int 1L; LispValue.int 2L; LispValue.int 3L ]
    ]
  in

  List.iter
    (fun v -> Printf.printf "Value: %s\n" (LispValue.to_string_debug v))
    values;

  (* Example 2: Building Expressions *)
  Printf.printf "\n--- Expression Building ---\n";
  let ticket_id_expr = Expr.const_string "TICKET-001" in
  let event_name_expr = Expr.const_string "Causality Con 2025" in
  let owner_expr = Expr.const_string "ocaml_user_pk" in

  let issue_ticket_expr =
    Expr.apply
      (Expr.const (LispValue.symbol "issue-ticket"))
      [ ticket_id_expr; event_name_expr; owner_expr ]
  in

  Printf.printf "Issue ticket expression: %s\n"
    (Expr.to_string issue_ticket_expr);

  (* Compile and register the expression *)
  (match Expr.compile_and_register_expr issue_ticket_expr with
  | Ok expr_id ->
      Printf.printf "Expression compiled successfully: %s\n"
        (Bytes.to_string expr_id)
  | Error _err -> Printf.printf "Expression compilation failed\n");

  (* Example 3: Creating and Submitting Intents *)
  Printf.printf "\n--- Intent Creation and Submission ---\n";

  (* Create an intent to issue a new ticket *)
  let issue_intent =
    Intent.create ~name:"IssueNewTicket" ~domain_id:"Ticketing"
  in

  (* Add parameters to the intent *)
  Intent.add_parameter issue_intent (LispValue.string "Causality Con 2025");
  Intent.add_parameter issue_intent (LispValue.string "TICKET-001");
  Intent.add_parameter issue_intent (LispValue.string "ocaml_user_pk");

  (* Set the Lisp logic (using predefined expression) *)
  (match Expr.get_predefined_expr_id "issue_ticket_logic" with
  | Some expr_id ->
      Intent.set_lisp_logic issue_intent expr_id;
      Printf.printf "Using predefined issue ticket logic\n"
  | None -> Printf.printf "Warning: predefined issue ticket logic not found\n");

  (* Submit the intent *)
  (match Intent.submit issue_intent with
  | Ok () -> (
      Printf.printf "Issue ticket intent submitted successfully\n";

      (* Simulate getting the produced resource *)
      let new_ticket_resource_id = System.get_last_produced_resource_id () in
      Printf.printf "New ticket resource ID: %s\n"
        (Bytes.to_string new_ticket_resource_id);

      (* Example 4: Transfer the ticket *)
      Printf.printf "\n--- Ticket Transfer ---\n";
      let transfer_intent =
        Intent.create ~name:"TransferTicket" ~domain_id:"Ticketing"
      in

      Intent.add_input_resource transfer_intent new_ticket_resource_id;
      Intent.add_parameter transfer_intent
        (LispValue.string "another_ocaml_user_pk");

      (match Expr.get_predefined_expr_id "transfer_ticket_logic" with
      | Some expr_id -> Intent.set_lisp_logic transfer_intent expr_id
      | None -> Printf.printf "Warning: transfer ticket logic not found\n");

      match Intent.submit transfer_intent with
      | Ok () -> Printf.printf "Transfer ticket intent submitted successfully\n"
      | Error _err -> Printf.printf "Error transferring ticket\n")
  | Error _err -> Printf.printf "Error issuing ticket\n");

  (* Example 5: Resource Management *)
  Printf.printf "\n--- Resource Management ---\n";
  (match System.get_resource_by_id (Bytes.of_string "test_resource") with
  | Ok (Some resource) -> Printf.printf "Found resource: %s\n" resource.name
  | Ok None -> Printf.printf "Resource not found\n"
  | Error _err -> Printf.printf "Error getting resource\n");

  (* Example 6: System Monitoring *)
  Printf.printf "\n--- System Monitoring ---\n";
  (match System.get_system_metrics () with
  | Ok metrics -> Printf.printf "System metrics: %s\n" metrics
  | Error _err -> Printf.printf "Error getting system metrics\n");

  (* Example 7: Domain Information *)
  Printf.printf "\n--- Domain Information ---\n";
  let ticketing_domain_id = Bytes.of_string "Ticketing" in
  (match System.get_domain_info ticketing_domain_id with
  | Ok (Some domain) ->
      Printf.printf "Domain found: %s\n"
        (match domain with
        | VerifiableDomain d ->
            "VerifiableDomain(" ^ Bytes.to_string d.domain_id ^ ")"
        | ServiceDomain d ->
            "ServiceDomain(" ^ Bytes.to_string d.domain_id ^ ")"
        | ComputeDomain d ->
            "ComputeDomain(" ^ Bytes.to_string d.domain_id ^ ")")
  | Ok None -> Printf.printf "Domain not found\n"
  | Error _err -> Printf.printf "Error getting domain info\n");

  (* Cleanup *)
  Ocaml_causality_interop.Ffi.cleanup_ffi ();
  Printf.printf "\n=== Example completed ===\n"
