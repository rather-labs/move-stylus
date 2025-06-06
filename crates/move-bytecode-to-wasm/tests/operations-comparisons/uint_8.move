module 0x01::comparisons_u8;

public fun less_than_u8(a: u8, b: u8): bool {
    a < b
}

public fun less_than_eq_u8(a: u8, b: u8): bool {
    a <= b
}

public fun greater_than_u8(a: u8, b: u8): bool {
    a > b
}

public fun greater_eq_than_u8(a: u8, b: u8): bool {
    a >= b
}
