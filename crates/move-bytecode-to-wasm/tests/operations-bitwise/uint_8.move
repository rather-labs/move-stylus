module 0x01::uint_8;

entry fun or(x: u8, y: u8): u8 {
    x | y
}

entry fun xor(x: u8, y: u8): u8 {
    x ^ y
}

entry fun and(x: u8, y: u8): u8 {
    x & y
}

entry fun shift_left(x: u8, slots: u8): u8 {
    x << slots
}

entry fun shift_right(x: u8, slots: u8): u8 {
    x >> slots
}
