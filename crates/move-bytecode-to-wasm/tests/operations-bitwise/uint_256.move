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
