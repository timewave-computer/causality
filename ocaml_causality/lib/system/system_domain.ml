(** Domain system for organizing resources and capabilities *)

(** Capability levels *)
type capability_level = Read | Write | Execute | Admin

type capability = { name : string; level : capability_level }
(** A capability represents permission to perform operations *)

type domain = { id : bytes; name : string; capabilities : capability list }
(** A domain represents a context for resource management and capability
    enforcement *)

(** Create a new domain *)
let create (name : string) (capabilities : capability list) : domain =
  let id = Bytes.create 32 in
  { id; name; capabilities }

(** Create the default domain with basic capabilities *)
let default_domain () : domain =
  let capabilities : capability list =
    [
      { name = "read"; level = Read }
    ; { name = "write"; level = Write }
    ; { name = "execute"; level = Execute }
    ]
  in
  create "default" capabilities

(** Check if this domain has a specific capability *)
let has_capability (domain : domain) (capability_name : string) : bool =
  let caps : capability list = domain.capabilities in
  List.exists (fun (cap : capability) -> cap.name = capability_name) caps

(** Get a capability by name *)
let get_capability (domain : domain) (name : string) : capability option =
  List.find_opt (fun (cap : capability) -> cap.name = name) domain.capabilities
