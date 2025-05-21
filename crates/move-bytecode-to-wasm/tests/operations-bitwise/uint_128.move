module 0x01::uint_128;

public fun or(x: u128, y: u128): u128 {
    x | y
}

public fun xor(x: u128, y: u128): u128 {
    x ^ y
}

public fun and(x: u128, y: u128): u128 {
    x & y
}