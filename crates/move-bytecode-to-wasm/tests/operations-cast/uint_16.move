module 0x01::uint_16;

public fun cast_down(x: u32): u16 {
    x as u16
}

public fun cast_up(x: u8): u16 {
    x as u16
}

public fun cast_from_u128(x: u128): u16 {
    x as u16
}

public fun cast_from_u256(x: u256): u16 {
    x as u16
}
