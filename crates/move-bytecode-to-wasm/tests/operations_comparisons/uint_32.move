module 0x01::comparisons_u32;

entry fun less_than_u32(a: u32, b: u32): bool {
    a < b
}

entry fun less_than_eq_u32(a: u32, b: u32): bool {
    a <= b
}

entry fun greater_than_u32(a: u32, b: u32): bool {
    a > b
}

entry fun greater_eq_than_u32(a: u32, b: u32): bool {
    a >= b
}
