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

public fun sum(x: u256, y: u256): u256 {
    x + y
}

public fun sum_overflow(x: u256): u256 {
   115792089237316195423570985008687907853269984665640564039457584007913129639935 + x
}
