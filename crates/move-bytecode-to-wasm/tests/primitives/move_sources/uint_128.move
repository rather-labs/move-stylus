module 0x01::uint_128;

const INT_AS_CONST: u128 = 128128;

entry fun get_constant(): u128 {
  INT_AS_CONST
}

entry fun get_constant_local(): u128 {
  let x: u128 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_local(_z: u128): u128 {
  let x: u128 = 100;
  let y: u128 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): (u128, u128) {
  let x: u128 = 100;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

entry fun echo(x: u128): u128 {
  identity(x)
}

entry fun echo_2(x: u128, y: u128): u128 {
  identity_2(x, y)
}

fun identity(x: u128): u128 {
  x
}

fun identity_2(_x: u128, y: u128): u128 {
  y
}

entry fun sum(x: u128, y: u128): u128 {
    x + y
}

entry fun sub(x: u128, y: u128): u128 {
    x - y
}

entry fun mul(x: u128, y: u128): u128 {
    x * y
}

entry fun div(x: u128, y: u128): u128 {
    x / y
}

entry fun mod_(x: u128, y: u128): u128 {
    x % y
}
