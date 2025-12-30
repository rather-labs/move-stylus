module 0x01::cast_uint_128;

entry fun cast_up(x: u16): u128 {
  x as u128
}

entry fun cast_up_u64(x: u64): u128 {
  x as u128
}

entry fun cast_from_u256(x: u256): u128 {
  x as u128
}
