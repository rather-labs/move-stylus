module 0x01::comparisons_u16;

entry fun less_than_u16(a: u16, b: u16): bool {
    a < b
}

entry fun less_than_eq_u16(a: u16, b: u16): bool {
    a <= b
}

entry fun greater_than_u16(a: u16, b: u16): bool {
    a > b
}

entry fun greater_eq_than_u16(a: u16, b: u16): bool {
    a >= b
}
