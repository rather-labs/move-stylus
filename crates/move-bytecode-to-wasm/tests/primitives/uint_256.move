module 0x01::uint_256;

const INT_AS_CONST: u256 = 256256;

entry fun get_constant(): u256 {
  INT_AS_CONST
}

entry fun get_constant_local(): u256 {
  let x: u256 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_local(_z: u256): u256 {
  let x: u256 = 100;
  let y: u256 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): (u256, u256) {
  let x: u256 = 100;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

entry fun echo(x: u256): u256 {
  identity(x)
}

entry fun echo_2(x: u256, y: u256): u256 {
  identity_2(x, y)
}

fun identity(x: u256): u256 {
  x
}

fun identity_2(_x: u256, y: u256): u256 {
  y
}

entry fun sum(x: u256, y: u256): u256 {
    x + y
}

entry fun sub(x: u256, y: u256): u256 {
    x - y
}

entry fun mul(x: u256, y: u256): u256 {
    x * y
}

entry fun div(x: u256, y: u256): u256 {
    x / y
}

entry fun mod_(x: u256, y: u256): u256 {
    x % y
}
