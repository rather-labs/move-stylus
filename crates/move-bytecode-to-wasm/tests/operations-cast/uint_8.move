module 0x01::uint_8;

public fun cast_down(x: u16): u8 {
    x as u8
}

public fun cast_from_u128(x: u128): u8 {
    x as u8
}

public fun cast_from_u256(x: u256): u8 {
    x as u8
}
