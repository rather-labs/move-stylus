module 0x01::vector;

const VEC_AS_CONST: vector<u256> = vector[1, 2, 3];

public fun get_constant(): vector<u256> {
  VEC_AS_CONST
}

public fun get_constant_local(): vector<u256> {
  let x: vector<u256> = VEC_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: vector<u256>): vector<u256> {
  let x: vector<u256> = vector[1, 2, 3];
  let y: vector<u256> = vector[4, 5, 6];
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<u256> {
  let x: vector<u256> = vector[1, 2, 3];
  
  let y = x; // copy
  let mut _z = x; // move
  identity(y);
  identity(_z);

  _z = 111;
  y
}

public fun echo(x: vector<u256>): vector<u256> {
  identity(x)
}

public fun echo_2(x: vector<u256>, y: vector<u256>): vector<u256> {
  identity_2(x, y)
}

fun identity(x: vector<u256>): vector<u256> {
  x
}

fun identity_2(_x: vector<u256>, _y: vector<u256>): vector<u256> {
  vector[]
}
