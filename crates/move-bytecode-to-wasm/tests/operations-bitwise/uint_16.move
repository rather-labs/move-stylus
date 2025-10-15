module 0x01::uint_16;

entry fun or(x: u16, y: u16): u16 {
    x | y
}

entry fun xor(x: u16, y: u16): u16 {
    x ^ y
}

entry fun and(x: u16, y: u16): u16 {
    x & y
}

entry fun shift_left(x: u16, slots: u8): u16 {
    x << slots
}

entry fun shift_right(x: u16, slots: u8): u16 {
    x >> slots
}
