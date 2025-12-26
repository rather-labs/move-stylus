module 0x01::bitwise_uint_64;

entry fun or(x: u64, y: u64): u64 {
    x | y
}

entry fun xor(x: u64, y: u64): u64 {
    x ^ y
}

entry fun and(x: u64, y: u64): u64 {
    x & y
}

entry fun shift_left(x: u64, slots: u8): u64 {
    x << slots
}

entry fun shift_right(x: u64, slots: u8): u64 {
    x >> slots
}
