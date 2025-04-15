module 0x01::hello_world;

// An empty vector of bool elements.
const EMPTY_VECTOR: vector<bool> = vector[];

// A vector of u8 elements.
const VECTOR_U8: vector<u8> = vector[1, 1, 1, 1, 1, 1];
const VECTOR_U16: vector<u16> = vector[10, 20, 30];
const VECTOR_U32: vector<u32> = vector[10, 20, 30];
const VECTOR_U64: vector<u64> = vector[10, 20, 30];
const VECTOR_U128: vector<u128> = vector[10, 20, 30];
const VECTOR_U256: vector<u256> = vector[10, 20, 30];
const VECTOR_ADDRESS: vector<address> = vector[@0x01, @0x02, @0x03];
const VECTOR_BOOLEAN: vector<bool> = vector[true, false, true];

// A vector of vector<u8> elements.
const VECTOR_VECTOR_U8: vector<vector<u8>> = vector[
  vector[10, 20],
  vector[30, 40]
];

public fun get_empty_vector(): vector<bool> {
  EMPTY_VECTOR
}

public fun get_vector_u8(): vector<u8> {
  VECTOR_U8
}

public fun get_vector_vector_u8(): vector<vector<u8>> {
  VECTOR_VECTOR_U8
}

// Forces the compiler to store literals on locals
public fun get_local(_z: vector<u8>): vector<u8> {
  let x: vector<u8> = vector[10, 20, 30];
  let y: vector<u8> = vector[11, 22, 33];
  identity(x);
  identity_3(x, y);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): (vector<u8>, vector<u8>) {
  let x: vector<u8> = vector[10, 20, 30];

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = vector[11, 22, 33];
  (y, z)
}

public fun echo(x: vector<u8>): vector<u8> {
  identity(x)
}

public fun echo_2(x: vector<u8>, y: vector<u8>): vector<u8> {
  identity_2(x, y)
}

fun identity(x: vector<u8>): vector<u8> {
  x
}

fun identity_2(_x: vector<u8>, y: vector<u8>): vector<u8> {
  y
}

fun identity_3(_x: vector<u8>, y: vector<u8>): (vector<u8>, vector<u8>) {
  (y, y)
}
