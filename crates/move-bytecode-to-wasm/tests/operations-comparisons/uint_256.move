module 0x01::comparisons_u256;

entry fun less_than_u256(a: u256, b: u256): bool {
    a < b
}

entry fun less_than_eq_u256(a: u256, b: u256): bool {
    a <= b
}

entry fun greater_than_u256(a: u256, b: u256): bool {
    a > b
}

entry fun greater_eq_than_u256(a: u256, b: u256): bool {
    a >= b
}

