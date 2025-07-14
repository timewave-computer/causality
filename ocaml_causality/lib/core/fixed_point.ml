(* ZK-compatible fixed point arithmetic using Zarith *)
(* Uses arbitrary precision integers for ZK compatibility *)

(* Fixed point representation: Z.t with 6 decimal places *)
(* Example: 1.234567 is represented as Z.of_int 1234567 *)
let scale_factor = Z.of_int 1000000

(* Type for fixed point numbers - arbitrary precision integers *)
type t = Z.t

(* Constants *)
let zero = Z.zero
let one = scale_factor
let scale = scale_factor

(* Convert from integer to fixed point *)
let of_int i = Z.mul (Z.of_int i) scale_factor

(* Convert from string representation like "1.234567" *)
let of_string s =
  try
    let parts = String.split_on_char '.' s in
    match parts with
    | [whole] -> 
        let whole_z = Z.of_string whole in
        Z.mul whole_z scale_factor
    | [whole; frac] ->
        let whole_z = Z.of_string whole in
        let whole_scaled = Z.mul whole_z scale_factor in
        
        (* Pad or truncate fractional part to 6 digits *)
        let frac_padded = 
          let len = String.length frac in
          if len >= 6 then String.sub frac 0 6
          else frac ^ String.make (6 - len) '0'
        in
        let frac_z = Z.of_string frac_padded in
        
        (* Add or subtract based on sign of whole part *)
        if Z.sign whole_z >= 0 then
          Z.add whole_scaled frac_z
        else
          Z.sub whole_scaled frac_z
    | _ -> failwith "Invalid fixed point string format"
  with
  | _ -> failwith "Invalid fixed point string format"

(* Convert to integer (truncates fractional part) *)
let to_int fp = Z.to_int (Z.div fp scale_factor)

(* Convert to string representation *)
let to_string fp =
  let whole_part = Z.div fp scale_factor in
  let remainder = Z.sub fp (Z.mul whole_part scale_factor) in
  let whole_str = Z.to_string whole_part in
  let frac_str = Z.to_string (Z.abs remainder) in
  (* Pad fractional part to 6 digits *)
  let frac_padded = 
    let len = String.length frac_str in
    if len >= 6 then String.sub frac_str 0 6
    else String.make (6 - len) '0' ^ frac_str
  in
  whole_str ^ "." ^ frac_padded

(* Basic arithmetic operations *)
let add = Z.add
let sub = Z.sub  
let neg = Z.neg

(* Multiplication with proper scaling *)
let mul a b = 
  let result = Z.mul a b in
  Z.div result scale_factor

(* Division with proper scaling *)
let div a b = 
  if Z.equal b Z.zero then failwith "Division by zero"
  else
    let scaled_a = Z.mul a scale_factor in
    Z.div scaled_a b

(* Comparison operations *)
let equal = Z.equal
let compare = Z.compare
let lt a b = Z.lt a b
let le a b = Z.leq a b  
let gt a b = Z.gt a b
let ge a b = Z.geq a b

(* Min and max *)
let min a b = if Z.lt a b then a else b
let max a b = if Z.gt a b then a else b

(* Absolute value *)
let abs = Z.abs

(* Power function (integer exponent) *)
let rec pow base exp =
  if exp = 0 then one
  else if exp = 1 then base
  else if exp < 0 then 
    let pos_pow = pow base (-exp) in
    div one pos_pow
  else
    (* For positive exponents, use repeated multiplication *)
    let rec power_helper acc b e =
      if e = 0 then acc
      else if e = 1 then mul acc b
      else if e mod 2 = 0 then
        power_helper acc (mul b b) (e / 2)
      else
        power_helper (mul acc b) (mul b b) ((e - 1) / 2)
    in
    power_helper one base exp

(* Modular arithmetic *)
let modulo a m_int = 
  let m = Z.of_int m_int in
  Z.erem a m

(* Check if number is valid for ZK circuits *)
let is_valid_for_zk fp =
  (* Most ZK systems work with numbers up to ~2^253 *)
  let max_bits = 253 in
  Z.numbits (Z.abs fp) <= max_bits

(* Serialization *)
let to_bytes fp = 
  let str = Z.to_string fp in
  Bytes.of_string str

let of_bytes bytes = 
  let str = Bytes.to_string bytes in
  Z.of_string str

(* Hash function *)
let hash fp = Hashtbl.hash (Z.to_string fp)

(* Pretty printing *)
let pp fmt fp = Format.fprintf fmt "%s" (to_string fp)

(* Additional utility functions *)
let is_zero fp = Z.equal fp Z.zero
let is_one fp = Z.equal fp one

(* Square root using Newton's method *)
let sqrt fp =
  if Z.lt fp Z.zero then failwith "Square root of negative number"
  else if Z.equal fp Z.zero then Z.zero
  else
    let rec newton_step x =
      let x_plus_fp_div_x = Z.add x (div fp x) in
      let next = Z.div x_plus_fp_div_x (Z.of_int 2) in
      let diff = Z.abs (Z.sub next x) in
      if Z.lt diff (Z.of_int 10) then next (* Precision: 0.00001 *)
      else newton_step next
    in
    newton_step fp

(* Random element generation for testing *)
let random () = 
  let random_int = Random.int 1000000 in
  Z.of_int random_int

(* Inverse (for non-zero elements) *)
let inverse fp = 
  if Z.equal fp Z.zero then failwith "Cannot invert zero"
  else div one fp

(* Additional convenience functions *)
let of_rational num den =
  if den = 0 then failwith "Division by zero in rational"
  else
    let num_z = Z.of_int num in
    let den_z = Z.of_int den in
    let scaled_num = Z.mul num_z scale_factor in
    Z.div scaled_num den_z

(* Common fractions *)
let half = Z.div scale_factor (Z.of_int 2)
let quarter = Z.div scale_factor (Z.of_int 4)
let tenth = Z.div scale_factor (Z.of_int 10)

(* Percentage operations *)
let percent p = div (Z.of_int p) (Z.of_int 100)
let apply_percent fp p = mul fp (percent p)

(* Basis points (1/10000) *)
let basis_points bp = div (Z.of_int bp) (Z.of_int 10000)
let apply_basis_points fp bp = mul fp (basis_points bp) 