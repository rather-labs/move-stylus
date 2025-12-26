module 0x01::vec_vec_32;

const VECTOR_AS_CONST: vector<vector<u32>> = vector[vector[1u32, 2u32, 3u32], vector[4u32, 5u32, 6u32], vector[7u32, 8u32, 9u32]];

entry fun get_constant(): vector<vector<u32>> {
  VECTOR_AS_CONST
}

entry fun get_constant_local(): vector<vector<u32>> {
  let x: vector<vector<u32>> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_literal(): vector<vector<u32>> {
  vector[vector[1u32, 2u32, 3u32], vector[4u32, 5u32, 6u32], vector[7u32, 8u32, 9u32]]
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): vector<vector<u32>> {
  let x: vector<vector<u32>> = vector[vector[1u32, 2u32, 3u32], vector[4u32, 5u32, 6u32], vector[7u32, 8u32, 9u32]];
  let y = x;
  let _z = x;
  y
}

entry fun echo(x: vector<vector<u32>>): vector<vector<u32>> {
  x
}

entry fun vec_len(x: vector<vector<u32>>): u64 {
  x.length()
}

entry fun vec_pop_back(x: vector<vector<u32>>): vector<vector<u32>> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

entry fun vec_swap(x: vector<vector<u32>>, id1: u64, id2: u64): vector<vector<u32>> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

entry fun vec_push_back(x: vector<vector<u32>>, y: vector<u32>): vector<vector<u32>> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

entry fun vec_push_and_pop_back(x: vector<vector<u32>>, y: vector<u32>): vector<vector<u32>> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}

entry fun vec_push_back_to_element(x: vector<vector<u32>>, y: u32): vector<vector<u32>> {
  let mut w = x;
  w[0].push_back(y);
  w[0].push_back(y);
  w
}

entry fun misc_0(x: vector<vector<u32>>, y: u32): vector<vector<u32>> {
  let mut w = x;
  w[0].push_back(y);
  let mut a = w[1];
  a.swap(0, 1);
  a.pop_back();
  a.push_back(y);
  let z = vector[w[0], a];
  z
}

// This generates a VecUnpack instruction
entry fun vec_unpack(x: vector<vector<u32>>): vector<vector<u32>> {
    let mut z = vector[vector[3], vector[1], vector[4]];
    x.do!(|e| z.push_back(e));
    z
}
