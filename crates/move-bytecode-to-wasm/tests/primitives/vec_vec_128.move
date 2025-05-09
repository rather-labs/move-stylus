module 0x01::vec_vec_128;

const VECTOR_AS_CONST: vector<vector<u128>> = vector[vector[1u128, 2u128, 3u128], vector[4u128, 5u128, 6u128], vector[7u128, 8u128, 9u128]];

public fun get_constant(): vector<vector<u128>> {
  VECTOR_AS_CONST
}

public fun get_constant_local(): vector<vector<u128>> {
  let x: vector<vector<u128>> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_literal(): vector<vector<u128>> {
  vector[vector[1u128, 2u128, 3u128], vector[4u128, 5u128, 6u128], vector[7u128, 8u128, 9u128]]
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<vector<u128>> {
  let x: vector<vector<u128>> = vector[vector[1u128, 2u128, 3u128], vector[4u128, 5u128, 6u128], vector[7u128, 8u128, 9u128]];
  let y = x; 
  let _z = x; 
  y
}

public fun echo(x: vector<vector<u128>>): vector<vector<u128>> {
  x
}