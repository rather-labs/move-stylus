module 0x01::equality;

public fun eq_u256(x: u256, y: u256): bool {
    x == y
}

public fun eq_u128(x: u128, y: u128): bool {
    x == y
}

public fun eq_u64(x: u64, y: u64): bool {
    x == y
}

public fun eq_u32(x: u32, y: u32): bool {
    x == y
}

public fun eq_u16(x: u16, y: u16): bool {
    x == y
}

public fun eq_u8(x: u8, y: u8): bool {
    x == y
}

public fun eq_address(x: address, y: address): bool {
    x == y
}

public fun neq_u256(x: u256, y: u256): bool {
    x != y
}

public fun neq_u128(x: u128, y: u128): bool {
    x != y
}

public fun neq_u64(x: u64, y: u64): bool {
    x != y
}

public fun neq_u32(x: u32, y: u32): bool {
    x != y
}

public fun neq_u16(x: u16, y: u16): bool {
    x != y
}

public fun neq_u8(x: u8, y: u8): bool {
    x != y
}

public fun neq_address(x: address, y: address): bool {
    x != y
}
