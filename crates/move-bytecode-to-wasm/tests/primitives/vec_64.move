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

