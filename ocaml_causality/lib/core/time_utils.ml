(* Time utilities for OCaml Causality - provides fixed-point timestamps *)

(* Get current time as integer timestamp (seconds since epoch) *)
let current_time_int () : int =
  Unix.time () |> int_of_float

(* Get current time as Int64 timestamp (milliseconds since epoch) *)
let current_time_int64_ms () : int64 =
  Unix.time () |> ( *. ) 1000.0 |> Int64.of_float

(* Add seconds to an integer timestamp *)
let add_seconds_int (timestamp : int) (seconds : int) : int =
  timestamp + seconds

(* Time comparison *)
let is_expired_int (timestamp : int) (current_time : int) : bool =
  timestamp < current_time

(* Time difference *)
let time_diff_int (time1 : int) (time2 : int) : int =
  abs (time1 - time2)

(* Time constants *)
let minute_int = 60
let hour_int = 3600
let day_int = 86400
let week_int = 604800
