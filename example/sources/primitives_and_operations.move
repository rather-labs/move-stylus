module hello_world::primitives_and_operations;

const BOOL_AS_CONST: bool = true;

public fun cast_u8(x: u16): u8 {
    x as u8
}

// Signer native type
public fun echo_signer_with_int(x: signer, y: u8): (u8, signer) {
    (y, x)
}

/// Arithmetic operations
public fun sum32(x: u32, y: u32): u32 {
    x + y
}

public fun sum128(x: u64, y: u64): u64 {
    x + y
}

public fun sub32(x: u32, y: u32): u32 {
    x - y
}

public fun sub128(x: u128, y: u128): u128 {
    x - y
}

public fun mul32(x: u32, y: u32): u32 {
    x * y
}

public fun mul128(x: u128, y: u128): u128 {
    x * y
}

public fun div32(x: u32, y: u32): u32 {
    x / y
}

public fun div128(x: u128, y: u128): u128 {
    x / y
}

public fun mod32(x: u32, y: u32): u32 {
    x % y
}

public fun mod128(x: u128, y: u128): u128 {
    x % y
}

// Bitwise operations
public fun or32(x: u32, y: u32): u32 {
    x | y
}

public fun xor32(x: u32, y: u32): u32 {
    x ^ y
}

public fun and32(x: u32, y: u32): u32 {
    x & y
}

public fun shift_left32(x: u32, slots: u8): u32 {
    x << slots
}

public fun shift_right32(x: u32, slots: u8): u32 {
    x >> slots
}

public fun or128(x: u128, y: u128): u128 {
    x | y
}

public fun xor128(x: u128, y: u128): u128 {
    x ^ y
}

public fun and128(x: u128, y: u128): u128 {
    x & y
}

public fun shift_left128(x: u128, slots: u8): u128 {
    x << slots
}

public fun shift_right128(x: u128, slots: u8): u128 {
    x >> slots
}

// Bool
public fun not_true(): bool {
  !BOOL_AS_CONST
}

public fun not(x: bool): bool {
  !x
}

public fun and(x: bool, y: bool): bool {
  x && y
}

public fun or(x: bool, y: bool): bool {
  x || y
}
