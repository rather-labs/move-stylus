module 0x01::uint_16;

public fun or(x: u16, y: u16): u16 {
    x | y
}

public fun xor(x: u16, y: u16): u16 {
    x ^ y
}

public fun and(x: u16, y: u16): u16 {
    x & y
}

public fun shift_left(x: u16, slots: u8): u16 {
    x << slots
}

public fun shift_right(x: u16, slots: u8): u16 {
    x >> slots
}
