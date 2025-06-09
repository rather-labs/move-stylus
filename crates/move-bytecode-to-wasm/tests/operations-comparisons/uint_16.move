module 0x01::comparisons_u16;

public fun less_than_u16(a: u16, b: u16): bool {
    a < b
}

public fun less_than_eq_u16(a: u16, b: u16): bool {
    a <= b
}

public fun greater_than_u16(a: u16, b: u16): bool {
    a > b
}

public fun greater_eq_than_u16(a: u16, b: u16): bool {
    a >= b
}
