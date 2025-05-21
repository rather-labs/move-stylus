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


