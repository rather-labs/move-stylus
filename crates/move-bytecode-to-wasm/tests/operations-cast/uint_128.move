module 0x01::uint_128;

public fun cast_up(x: u16): u128 {
  x as u128
}

public fun cast_up_u64(x: u64): u128 {
  x as u128
}

public fun cast_from_u256(x: u256): u128 {
  x as u128
}
