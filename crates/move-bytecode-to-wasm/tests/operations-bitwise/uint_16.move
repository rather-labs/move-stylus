module 0x01::uint_16;

public fun or(x: u16, y: u16): u16 {
    x | y
}

public fun xor(x: u16, y: u16): u16 {
    x ^ y
}

public fun and(x: u16, y: u16): u16 {
    x & y
}
