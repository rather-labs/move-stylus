module 0x01::uint_32;

public fun or(x: u32, y: u32): u32 {
    x | y
}

public fun xor(x: u32, y: u32): u32 {
    x ^ y
}

public fun and(x: u32, y: u32): u32 {
    x & y
}
