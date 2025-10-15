module 0x01::uint_256;

entry fun or(x: u256, y: u256): u256 {
    x | y
}

entry fun xor(x: u256, y: u256): u256 {
    x ^ y
}

entry fun and(x: u256, y: u256): u256 {
    x & y
}

entry fun shift_left(x: u256, slots: u8): u256 {
    x << slots
}

entry fun shift_right(x: u256, slots: u8): u256 {
    x >> slots
}
