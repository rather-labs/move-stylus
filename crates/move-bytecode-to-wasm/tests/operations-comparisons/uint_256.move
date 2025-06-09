module 0x01::comparisons_u256;

public fun less_than_u256(a: u256, b: u256): bool {
    a < b
}

public fun less_than_eq_u256(a: u256, b: u256): bool {
    a <= b
}

public fun greater_than_u256(a: u256, b: u256): bool {
    a > b
}

public fun greater_eq_than_u256(a: u256, b: u256): bool {
    a >= b
}

