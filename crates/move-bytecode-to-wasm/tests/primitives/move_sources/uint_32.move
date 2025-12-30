module 0x01::uint_32;

const INT_AS_CONST: u32 = 3232;

entry fun get_constant(): u32 {
  INT_AS_CONST
}

entry fun get_constant_local(): u32 {
  let x: u32 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_local(_z: u32): u32 {
  let x: u32 = 100;
  let y: u32 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): (u32, u32) {
  let x: u32 = 100;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

entry fun echo(x: u32): u32 {
  identity(x)
}

entry fun echo_2(x: u32, y: u32): u32 {
  identity_2(x, y)
}

fun identity(x: u32): u32 {
  x
}

fun identity_2(_x: u32, y: u32): u32 {
  y
}

entry fun sum(x: u32, y: u32): u32 {
    x + y
}

entry fun sub(x: u32, y: u32): u32 {
    x - y
}

entry fun div(x: u32, y: u32): u32 {
    x / y
}

entry fun mul(x: u32, y: u32): u32 {
    x * y
}

entry fun mod_(x: u32, y: u32): u32 {
    x % y
}
