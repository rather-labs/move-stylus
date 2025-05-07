module 0x01::vec_uint32;

const INT_AS_CONST: u256 = 256256;

public fun get_constant(): u256 {
  INT_AS_CONST
}

public fun get_constant_local(): u256 {
  let x: u256 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: u256): u256 {
  let x: u256 = 100;
  let y: u256 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): u256 {
  let x: u256 = 100;
  
  let y = x; // copy
  let mut _z = x; // move
  identity(y);
  identity(_z);

  _z = 111;
  y
}

public fun echo(x: u256): u256 {
  identity(x)
}

public fun echo_2(x: u256, y: u256): u256 {
  identity_2(x, y)
}

fun identity(x: u256): u256 {
  x
}

fun identity_2(_x: u256, y: u256): u256 {
  y
}
