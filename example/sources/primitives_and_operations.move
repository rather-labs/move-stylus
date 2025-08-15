module hello_world::primitives_and_operations;

const BOOL_AS_CONST: bool = true;

public fun cast_u8(x: u128): u8 {
    x as u8
}

/// Arithmetic operations
public fun sum_u256(x: u256, y: u256): u256 {
    x + y
}

public fun sub_u128(x: u128, y: u128): u128 {
    x - y
}

public fun mul_u64(x: u64, y: u64): u64 {
    x * y
}

public fun div_u32(x: u32, y: u32): u32 {
    x / y
}

public fun mod_u16(x: u16, y: u16): u16 {
    x % y
}

// Bitwise operations
public fun or_u256(x: u256, y: u256): u256 {
    x | y
}

public fun xor_u128(x: u128, y: u128): u128 {
    x ^ y
}

public fun and_u64(x: u64, y: u64): u64 {
    x & y
}

public fun shift_left_u32(x: u32, slots: u8): u32 {
    x << slots
}

public fun shift_right_u16(x: u16, slots: u8): u16 {
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

// Comparison operations
public fun less_than_u256(a: u256, b: u256): bool {
    a < b
}

public fun less_than_eq_u128(a: u128, b: u128): bool {
    a <= b
}

public fun greater_than_u64(a: u64, b: u64): bool {
    a > b
}

public fun greater_than_eq_u32(a: u32, b: u32): bool {
    a >= b
}

// Vector operations
public fun vec_from_u256(x: u256, y: u256): vector<u256> {
  let z = vector[x, y, x];
  z
}

public fun vec_len_u128(x: vector<u128>): u64 {
  x.length()
}

public fun vec_pop_back_u64(x: vector<u64>): vector<u64> {
  let mut y = x;
  y.pop_back();
  y
}

public fun vec_swap_u32(x: vector<u32>, id1: u64, id2: u64): vector<u32> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back_u16(x: vector<u16>, y: u16): vector<u16> {
  let mut z = x;
  z.push_back(y);
  z
}
