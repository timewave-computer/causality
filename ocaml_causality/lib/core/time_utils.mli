(* Time utilities interface for OCaml Causality *)

(* Current time functions *)
val current_time_int : unit -> int
val current_time_int64_ms : unit -> int64

(* Time arithmetic *)
val add_seconds_int : int -> int -> int

(* Time comparison *)
val is_expired_int : int -> int -> bool

(* Time difference *)
val time_diff_int : int -> int -> int

(* Time constants *)
val minute_int : int
val hour_int : int
val day_int : int
val week_int : int
