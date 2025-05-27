module 0x01::uint_256;

public fun or(x: u256, y: u256): u256 {
    x | y
}

public fun xor(x: u256, y: u256): u256 {
    x ^ y
}

public fun and(x: u256, y: u256): u256 {
    x & y
}

public fun shift_left(x: u256, slots: u8): u256 {
    x << slots
}

public fun shift_right(x: u256, slots: u8): u256 {
    x >> slots
}
