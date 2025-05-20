module 0x01::uint_256;

public fun cast_up(x: u16): u256 {
  x as u256
}

public fun cast_up_u64(x: u64): u256 {
  x as u256
}

public fun cast_up_u128(x: u128): u256 {
  x as u256
}
