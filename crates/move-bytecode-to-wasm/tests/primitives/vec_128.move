module 0x01::vec_128;

const VECTOR_AS_CONST: vector<u128> = vector[1u128, 2u128, 3u128];

public fun get_constant(): vector<u128> {
  VECTOR_AS_CONST
}

public fun get_constant_local(): vector<u128> {
  let x: vector<u128> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_literal(): vector<u128> {
  vector[1u128, 2u128, 3u128]
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<u128> {
  let x: vector<u128> = vector[1u128, 2u128, 3u128];
  let y = x;
  let _z = x;
  y
}

public fun vec_from_int(x: u128, y: u128): vector<u128> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec(x: vector<u128>, y: vector<u128>): vector<vector<u128>> {
  let z = vector[x, y];
  z
}

public fun vec_from_vec_and_int(x: vector<u128>, y: u128): vector<vector<u128>> {
  let z = vector[x, vector[y, y]];
  z
}

public fun echo(x: vector<u128>): vector<u128> {
  x
}

public fun vec_pop_back(x: vector<u128>): vector<u128> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap(x: vector<u128>, id1: u64, id2: u64): vector<u128> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back(x: vector<u128>, y: u128): vector<u128> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

public fun vec_push_and_pop_back(x: vector<u128>, y: u128): vector<u128> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}

public fun vec_len(x: vector<u128>): u64 {
  x.length()
}
