module 0x01::vec_64;

const VECTOR_AS_CONST: vector<u64> = vector[1u64, 2u64, 3u64];

public fun get_constant(): vector<u64> {
  VECTOR_AS_CONST
}

public fun get_constant_local(): vector<u64> {
  let x: vector<u64> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_literal(): vector<u64> {
  vector[1u64, 2u64, 3u64]
}


public fun vec_from_int(x: u64, y: u64): vector<u64> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec(x: vector<u64>, y: vector<u64>): vector<vector<u64>> {
  let z = vector[x, y];
  z
}

public fun vec_from_vec_and_int(x: vector<u64>, y: u64): vector<vector<u64>> {
  let z = vector[x, vector[y, y]];
  z
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<u64> {
  let x: vector<u64> = vector[1u64, 2u64, 3u64];
  let y = x; 
  let _z = x; 
  y
}

public fun echo(x: vector<u64>): vector<u64> {
  x
}

public fun ref(x: vector<u64>): vector<u64> {
  let y = &x;
  *y
}

public fun vec_len(x: vector<u64>): u64 {
  x.length()
}

public fun vec_pop_back(x: vector<u64>): vector<u64> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap(x: vector<u64>, id1: u64, id2: u64): vector<u64> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back(x: vector<u64>, y: u64): vector<u64> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

public fun vec_push_and_pop_back(x: vector<u64>, y: u64): vector<u64> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}
