module 0x01::uint_32;

public fun or(x: u32, y: u32): u32 {
    x | y
}

public fun xor(x: u32, y: u32): u32 {
    x ^ y
}

public fun and(x: u32, y: u32): u32 {
    x & y
}

public fun shift_left(x: u32, slots: u8): u32 {
    x << slots
}

public fun shift_right(x: u32, slots: u8): u32 {
    x >> slots
}
