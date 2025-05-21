module 0x01::uint_64;

public fun or(x: u64, y: u64): u64 {
    x | y
}

public fun xor(x: u64, y: u64): u64 {
    x ^ y
}

public fun and(x: u64, y: u64): u64 {
    x & y
}
