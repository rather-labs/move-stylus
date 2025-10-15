module 0x01::comparisons_u128;

entry fun less_than_u128(a: u128, b: u128): bool {
    a < b
}

entry fun less_than_eq_u128(a: u128, b: u128): bool {
    a <= b
}

entry fun greater_than_u128(a: u128, b: u128): bool {
    a > b
}

entry fun greater_eq_than_u128(a: u128, b: u128): bool {
    a >= b
}
