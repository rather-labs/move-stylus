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

// Comparison operations
public fun less_than_u32(a: u32, b: u32): bool {
    a < b
}

public fun less_than_eq_u32(a: u32, b: u32): bool {
    a <= b
}

public fun greater_than_u32(a: u32, b: u32): bool {
    a > b
}

public fun greater_eq_than_u32(a: u32, b: u32): bool {
    a >= b
}

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

// Vector operations
public fun vec_from_int32(x: u32, y: u32): vector<u32> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec_and_int32(x: vector<u32>, y: u32): vector<vector<u32>> {
  let z = vector[x, vector[y, y]];
  z
}

public fun vec_len32(x: vector<u32>): u64 {
  x.length()
}

public fun vec_pop_back32(x: vector<u32>): vector<u32> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap32(x: vector<u32>, id1: u64, id2: u64): vector<u32> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back32(x: vector<u32>, y: u32): vector<u32> {
  let mut z = x;
  z.push_back(y);
  z
}

public fun vec_from_int128(x: u128, y: u128): vector<u128> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec_and_int128(x: vector<u128>, y: u128): vector<vector<u128>> {
  let z = vector[x, vector[y, y]];
  z
}

public fun vec_pop_back128(x: vector<u128>): vector<u128> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap128(x: vector<u128>, id1: u64, id2: u64): vector<u128> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back128(x: vector<u128>, y: u128): vector<u128> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

public fun vec_len(x: vector<u128>): u64 {
  x.length()
}
