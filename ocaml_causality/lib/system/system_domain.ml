(** Domain system for organizing resources and capabilities *)

(** Capability levels *)
type capability_level = 
  | Read 
  | Write 
  | Execute 
  | Admin

(** A capability represents permission to perform operations *)
type capability = {
  name : string;
  level : capability_level;
}

(** A domain represents a context for resource management and capability enforcement *)
type domain = {
  id : bytes;
  name : string;
  capabilities : capability list;
}

(** Main type for module interface *)
type t = domain

(** Create a new domain *)
let create (name : string) (capabilities : capability list) : domain =
  let id = Bytes.create 32 in
  { id; name; capabilities }

(** Create the default domain with basic capabilities *)
let default_domain () : domain =
  let capabilities = [
    { name = "read"; level = Read };
    { name = "write"; level = Write };
    { name = "execute"; level = Execute };
  ] in
  create "default" capabilities

(** Check if this domain has a specific capability *)
let has_capability (domain : domain) (capability_name : string) : bool =
  List.exists (fun cap -> cap.name = capability_name) domain.capabilities

(** Get a capability by name *)
let get_capability (domain : domain) (name : string) : capability option =
  List.find_opt (fun cap -> cap.name = name) domain.capabilities 