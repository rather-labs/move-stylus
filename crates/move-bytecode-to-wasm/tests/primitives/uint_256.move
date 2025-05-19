module 0x01::uint_256;

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
public fun get_copied_local(): (u256, u256) {
  let x: u256 = 100;
  
  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
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

public fun cast_u256_up(x: u16): u256 {
  x as u256
}

public fun cast_u256_up_u64(x: u64): u256 {
  x as u256
}

public fun cast_u256_up_u128(x: u128): u256 {
  x as u256
}
