module 0x01::vec_32;

const VECTOR_AS_CONST: vector<u32> = vector[1u32, 2u32, 3u32];

entry fun get_constant(): vector<u32> {
  VECTOR_AS_CONST
}

entry fun get_constant_local(): vector<u32> {
  let x: vector<u32> = VECTOR_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_literal(): vector<u32> {
  vector[1u32, 2u32, 3u32]
}

entry fun vec_from_int(x: u32, y: u32): vector<u32> {
  let z = vector[x, y, x];
  z
}

entry fun vec_from_vec(x: vector<u32>, y: vector<u32>): vector<vector<u32>> {
  let z = vector[x, y];
  z
}

entry fun vec_from_vec_and_int(x: vector<u32>, y: u32): vector<vector<u32>> {
  let z = vector[x, vector[y, y]];
  z
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): vector<u32> {
  let x: vector<u32> = vector[1u32, 2u32, 3u32];
  let y = x;
  let _z = x;
  y
}

entry fun echo(x: vector<u32>): vector<u32> {
  x
}

entry fun ref(x: vector<u32>): vector<u32> {
  let y = &x;
  *y
}

entry fun vec_len(x: vector<u32>): u64 {
  x.length()
}

entry fun vec_pop_back(x: vector<u32>): vector<u32> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

entry fun vec_swap(x: vector<u32>, id1: u64, id2: u64): vector<u32> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

entry fun vec_push_back(x: vector<u32>, y: u32): vector<u32> {
  let mut z = x;
  z.push_back(y);
  z
}

entry fun vec_push_and_pop_back(x: vector<u32>, y: u32): vector<u32> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}

// This generates a VecUnpack instruction
entry fun vec_unpack(x: vector<u32>): vector<u32> {
    let mut z = vector[3, 1, 4];
    x.do!(|e| z.push_back(e));
    z
}

entry fun cumulative_sum(x: vector<u32>): u32 {
    let mut sum = 0u32;
    let mut i = 0;
    while (i < vector::length(&x)) {
      sum = sum + *vector::borrow(&x, i);
      i = i + 1;
    };
    sum
}

 