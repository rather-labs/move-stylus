module 0x01::imm_ref;
public fun ref_u8(x: u8): u8 {
  let y = &x;
  *y
}

public fun ref_u16(x: u16): u16 {
  let y = &x;
  *y
}

public fun ref_u32(x: u32): u32 {
  let y = &x;
  *y
}


public fun ref_u64(x: u64): u64 {
  let y = &x;
  *y
}


public fun ref_u128(x: u128): u128 {
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




