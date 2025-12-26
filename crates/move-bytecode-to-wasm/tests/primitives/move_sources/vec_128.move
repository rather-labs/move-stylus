module 0x01::vec_128;

const VECTOR_AS_CONST: vector<u128> = vector[1u128, 2u128, 3u128];

entry fun get_constant(): vector<u128> {
  VECTOR_AS_CONST
}

entry fun get_constant_local(): vector<u128> {
  let x: vector<u128> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_literal(): vector<u128> {
  vector[1u128, 2u128, 3u128]
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): vector<u128> {
  let x: vector<u128> = vector[1u128, 2u128, 3u128];
  let y = x;
  let _z = x;
  y
}

entry fun vec_from_int(x: u128, y: u128): vector<u128> {
  let z = vector[x, y, x];
  z
}

entry fun vec_from_vec(x: vector<u128>, y: vector<u128>): vector<vector<u128>> {
  let z = vector[x, y];
  z
}

entry fun vec_from_vec_and_int(x: vector<u128>, y: u128): vector<vector<u128>> {
  let z = vector[x, vector[y, y]];
  z
}

entry fun echo(x: vector<u128>): vector<u128> {
  x
}

entry fun vec_pop_back(x: vector<u128>): vector<u128> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

entry fun vec_swap(x: vector<u128>, id1: u64, id2: u64): vector<u128> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

entry fun vec_push_back(x: vector<u128>, y: u128): vector<u128> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

entry fun vec_push_and_pop_back(x: vector<u128>, y: u128): vector<u128> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}

entry fun vec_len(x: vector<u128>): u64 {
  x.length()
}

// This generates a VecUnpack instruction
entry fun vec_unpack(x: vector<u128>): vector<u128> {
    let mut z = vector[3, 1, 4];
    x.do!(|e| z.push_back(e));
    z
}
