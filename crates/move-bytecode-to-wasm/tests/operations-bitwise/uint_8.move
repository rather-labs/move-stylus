module 0x01::uint_8;

public fun or(x: u8, y: u8): u8 {
    x | y
}

public fun xor(x: u8, y: u8): u8 {
    x ^ y
}

public fun and(x: u8, y: u8): u8 {
    x & y
}

public fun shift_left(x: u8, slots: u8): u8 {
    x << slots
}

public fun shift_right(x: u8, slots: u8): u8 {
    x >> slots
}
