module 0x01::uint_32;

entry fun or(x: u32, y: u32): u32 {
    x | y
}

entry fun xor(x: u32, y: u32): u32 {
    x ^ y
}

entry fun and(x: u32, y: u32): u32 {
    x & y
}

entry fun shift_left(x: u32, slots: u8): u32 {
    x << slots
}

entry fun shift_right(x: u32, slots: u8): u32 {
    x >> slots
}
