module 0x01::uint_32;

public fun cast_down(x: u64): u32 {
    x as u32
}

public fun cast_up(x: u16): u32 {
    x as u32
}

public fun cast_from_u128(x: u128): u32 {
    x as u32
}

public fun cast_from_u256(x: u256): u32 {
    x as u32
}
