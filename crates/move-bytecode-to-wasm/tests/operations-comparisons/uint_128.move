module 0x01::comparisons_u128;

public fun less_than_u128(a: u128, b: u128): bool {
    a < b
}

public fun less_than_eq_u128(a: u128, b: u128): bool {
    a <= b
}

public fun greater_than_u128(a: u128, b: u128): bool {
    a > b
}

public fun greater_eq_than_u128(a: u128, b: u128): bool {
    a >= b
}
