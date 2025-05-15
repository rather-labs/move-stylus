module 0x01::imm_ref;

public fun ref_u8(x: u8): u8 {
  let y = &x;
  *y
}

public fun ref_u64(x: u64): u64 {
  let y = &x;
  *y
}

public fun ref_u256(x: u256): u256 {
  let y = &x;
  *y
}

public fun ref_bool(x: bool): bool {
  let y = &x;
  *y
}

public fun ref_address(x: address): address {
  let y = &x;
  *y
}

public fun ref_vec_u8(x: vector<u8>): vector<u8> {
  let y = &x;
  *y
}

public fun ref_vec_u64(x: vector<u64>): vector<u64> {
  let y = &x;
  *y
}

public fun ref_vec_u256(x: vector<u256>): vector<u256> {
  let y = &x;
  *y
}

public fun ref_u8_arg(y: &u8): u8 {
  *y
}

public fun ref_vec_u8_arg(y: &vector<u8>): vector<u8> {
  *y
}

public fun ref_vec_u128_arg(y: &vector<u128>): vector<u128> {
  *y
}

public fun call_ref_u8_internal(x: u8): u8 {
  ref_u8_arg(&x)
}

public fun call_ref_vec_u8_internal(x: vector<u8>): vector<u8> {
  ref_vec_u8_arg(&x)
}

public fun call_ref_vec_u128_internal(x: vector<u128>): vector<u128> {
  ref_vec_u128_arg(&x)
}
