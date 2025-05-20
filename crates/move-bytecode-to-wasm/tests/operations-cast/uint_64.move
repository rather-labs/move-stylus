module 0x01::uint_64;

public fun cast_up(x: u32): u64 {
    x as u64
}

public fun cast_from_u128(x: u128): u64 {
    x as u64
}

public fun cast_from_u256(x: u256): u64 {
    x as u64
}
