module 0x01::vec_32;

const VECTOR_AS_CONST: vector<u32> = vector[1u32, 2u32, 3u32];

public fun get_constant(): vector<u32> {
  VECTOR_AS_CONST
}

public fun get_constant_local(): vector<u32> {
  let x: vector<u32> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_literal(): vector<u32> {
  vector[1u32, 2u32, 3u32]
}

public fun vec_from_int(x: u32, y: u32): vector<u32> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec(x: vector<u32>, y: vector<u32>): vector<vector<u32>> {
  let z = vector[x, y];
  z
}

public fun vec_from_vec_and_int(x: vector<u32>, y: u32): vector<vector<u32>> {
  let z = vector[x, vector[y, y]];
  z
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<u32> {
  let x: vector<u32> = vector[1u32, 2u32, 3u32];
  let y = x; 
  let _z = x; 
  y
}

public fun echo(x: vector<u32>): vector<u32> {
  x
}

public fun ref(x: vector<u32>): vector<u32> {
  let y = &x;
  *y
}

public fun vec_len(x: vector<u32>): u64 {
  x.length()
}

public fun vec_pop_back(x: vector<u32>): vector<u32> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap(x: vector<u32>, id1: u64, id2: u64): vector<u32> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back(x: vector<u32>, y: u32): vector<u32> {
  let mut z = x;
  z.push_back(y);
  // z.push_back(y);
  z
}

public fun vec_push_and_pop_back(x: vector<u32>, y: u32): vector<u32> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}
