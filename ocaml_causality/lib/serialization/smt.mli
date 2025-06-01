(** Sparse Merkle Tree (SMT) Interface
 *
 * This module defines the interface for a simplified Sparse Merkle Tree (SMT)
 * implementation. It provides key-value storage with cryptographic hashing,
 * suitable for use in systems requiring verifiable data structures.
 *)

(** The type of a hash digest. *)
type hash = bytes

(** The type of an SMT. It is abstract. *)
type t

(** [create ()] creates a new, empty SMT. *)
val create : unit -> t

(** [store smt key data] stores the [data] associated with [key] in the [smt].
    Returns a new SMT with the updated root hash reflecting the change.
    The key is a string, and data is bytes. *)
val store : t -> string -> bytes -> t

(** [get smt key] retrieves the data associated with [key] from the [smt].
    Returns [Some data] if the key exists, or [None] otherwise. *)
val get : t -> string -> bytes option

(** [has smt key] checks if the [key] exists in the [smt].
    Returns [true] if the key exists, [false] otherwise. *)
val has : t -> string -> bool

(** [remove smt key] removes the [key] and its associated data from the [smt].
    Returns a new SMT. Note: In this simplified version, removing a key
    resets the root to an empty hash, which is not standard SMT behavior
    but reflects the legacy implementation's approach. *)
val remove : t -> string -> t

(** [get_root smt] returns the current root hash of the [smt]. *)
val get_root : t -> hash

(** [hash_to_hex hash_bytes] converts a byte-represented hash to its hexadecimal string. *)
val hash_to_hex : hash -> string

(** [sha256_hash data] computes the SHA256 hash of the given [data]. *)
val sha256_hash : bytes -> hash

(* TEG-specific operations - These are kept for compatibility but might be
   better handled at a higher level or refactored. *)

(** [teg_key domain_id entity_type entity_id] constructs a standardized key
    string for TEG entities. *)
val teg_key : string -> string -> string -> string

(** [store_teg_data smt domain_id entity_type entity_id data] stores TEG entity
    data using the standardized TEG key. Returns the updated SMT and the key used. *)
val store_teg_data : t -> string -> string -> string -> bytes -> t * string

(** [get_teg_data smt domain_id entity_type entity_id] retrieves TEG entity data. *)
val get_teg_data : t -> string -> string -> string -> bytes option

(** [has_teg_data smt domain_id entity_type entity_id] checks for TEG entity data. *)
val has_teg_data : t -> string -> string -> string -> bool

(** [store_content_addressed smt domain_id entity_type data] stores data by its
    content hash, using the TEG keying scheme. Returns the updated SMT and
    the content ID (hex hash) of the data. *)
val store_content_addressed : t -> string -> string -> bytes -> t * string

(** [get_content_addressed smt domain_id entity_type content_id] retrieves
    content-addressed data. *)
val get_content_addressed : t -> string -> string -> string -> bytes option

(** [batch_store smt key_data_pairs] stores multiple key-value pairs in the SMT.
    [key_data_pairs] is a list of (string * bytes). *)
val batch_store : t -> (string * bytes) list -> t

(** [content_addressable_teg_key domain_id entity_type data] generates a TEG key
    based on the content hash of the data. *)
val content_addressable_teg_key : string -> string -> bytes -> string 