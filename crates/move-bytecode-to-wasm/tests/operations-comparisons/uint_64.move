module 0x01::comparisons_u64;

public fun less_than_u64(a: u64, b: u64): bool {
    a < b
}

public fun less_than_eq_u64(a: u64, b: u64): bool {
    a <= b
}

public fun greater_than_u64(a: u64, b: u64): bool {
    a > b
}

public fun greater_eq_than_u64(a: u64, b: u64): bool {
    a >= b
}
