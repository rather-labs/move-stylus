module 0x01::uint_128;

const INT_AS_CONST: u128 = 128128;

public fun get_constant(): u128 {
  INT_AS_CONST
}

public fun get_constant_local(): u128 {
  let x: u128 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: u128): u128 {
  let x: u128 = 100;
  let y: u128 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): (u128, u128) {
  let x: u128 = 100;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

public fun echo(x: u128): u128 {
  identity(x)
}

public fun echo_2(x: u128, y: u128): u128 {
  identity_2(x, y)
}

fun identity(x: u128): u128 {
  x
}

fun identity_2(_x: u128, y: u128): u128 {
  y
}

public fun sum(x: u128, y: u128): u128 {
    x + y
}

public fun sub(x: u128, y: u128): u128 {
    x - y
}

public fun mul(x: u128, y: u128): u128 {
    x * y
}

public fun div(x: u128, y: u128): u128 {
    x / y
}

public fun mod_(x: u128, y: u128): u128 {
    x % y
}
