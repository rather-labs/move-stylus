module 0x01::vec_32;

const VECTOR_AS_CONST: vector<u32> = vector[1u32, 2u32, 3u32];

// This one exceeds the 1 byte length limit
const LARGE_VECTOR_AS_CONST: vector<u32> = vector[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
    33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
    49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
    113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128
];
entry fun get_constant(): vector<u32> {
  VECTOR_AS_CONST
}

entry fun get_constant_local(): vector<u32> {
  let x: vector<u32> = VECTOR_AS_CONST;
  x
}

entry fun get_large_constant(): vector<u32> {
  LARGE_VECTOR_AS_CONST
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

//////////
entry fun vec_append(x: &mut vector<u32>, y: vector<u32>): vector<u32> {
    vector::append(x, y);
    *x
}   

fun vec_append_(lhs: &mut vector<u32>, mut other: vector<u32>) {
    vector::reverse(&mut other);
    while (!vector::is_empty(&other)) vector::push_back(lhs, vector::pop_back(&mut other));
    vector::destroy_empty(other);
}

entry fun vec_append_2(x: &mut vector<u32>, y: vector<u32>): vector<u32> {
    vec_append_(x, y);
    *x
}

fun mutate_mut_ref_vector(x: &mut vector<u32>) {
    x.push_back(42u32);
    x.push_back(43u32);
    x.push_back(44u32);
}

entry fun test_mutate_mut_ref_vector(x: &mut vector<u32>): vector<u32> {
    mutate_mut_ref_vector(x);
    *x
}

entry fun test_mutate_mut_ref_vector_2(mut x: vector<u32>): vector<u32> {
    mutate_mut_ref_vector(&mut x);
    x
}

entry fun test_contains(v: &vector<u32>, e: &u32): bool {
    vector::contains<u32>(v, e)
}

entry fun test_remove(v: &mut vector<u32>, index: u64): vector<u32> {
    vector::remove<u32>(v, index);
    *v
}