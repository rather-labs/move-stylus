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

public fun vec_len(x: vector<vector<u128>>): u64 {
  x.length()
}

public fun vec_pop_back(x: vector<vector<u128>>): vector<vector<u128>> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap(x: vector<vector<u128>>, id1: u64, id2: u64): vector<vector<u128>> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back(x: vector<vector<u128>>, y: vector<u128>): vector<vector<u128>> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

public fun vec_push_back_to_element(x: vector<vector<u128>>, y: u128): vector<vector<u128>> {
  let mut w = x;
  w[0].push_back(y);
  w[0].push_back(y);
  w
}

public fun vec_push_and_pop_back(x: vector<vector<u128>>, y: vector<u128>): vector<vector<u128>> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}
  
  
public fun misc_0(x: vector<vector<u128>>, y: u128): vector<vector<u128>> {
  let mut w = x;
  w[0].push_back(y);
  let mut a = w[1];
  a.swap(0, 1);
  a.pop_back();
  a.push_back(y);
  let z = vector[w[0], a];
  z
}