module 0x01::uint_128;

entry fun or(x: u128, y: u128): u128 {
    x | y
}

entry fun xor(x: u128, y: u128): u128 {
    x ^ y
}

entry fun and(x: u128, y: u128): u128 {
    x & y
}

entry fun shift_left(x: u128, slots: u8): u128 {
    x << slots
}

entry fun shift_right(x: u128, slots: u8): u128 {
    x >> slots
}
