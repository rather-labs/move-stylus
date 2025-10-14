module 0x01::comparisons_u64;

entry fun less_than_u64(a: u64, b: u64): bool {
    a < b
}

entry fun less_than_eq_u64(a: u64, b: u64): bool {
    a <= b
}

entry fun greater_than_u64(a: u64, b: u64): bool {
    a > b
}

entry fun greater_eq_than_u64(a: u64, b: u64): bool {
    a >= b
}
