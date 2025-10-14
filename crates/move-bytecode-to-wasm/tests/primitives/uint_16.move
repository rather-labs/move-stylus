module 0x01::uint_16;

const INT_AS_CONST: u16 = 1616;

entry fun get_constant(): u16 {
  INT_AS_CONST
}

entry fun get_constant_local(): u16 {
  let x: u16 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
entry fun get_local(_z: u16): u16 {
  let x: u16 = 100;
  let y: u16 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
entry fun get_copied_local(): (u16, u16) {
  let x: u16 = 100;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

entry fun echo(x: u16): u16 {
  identity(x)
}

entry fun echo_2(x: u16, y: u16): u16 {
  identity_2(x, y)
}

fun identity(x: u16): u16 {
  x
}

fun identity_2(_x: u16, y: u16): u16 {
  y
}

entry fun sum(x: u16, y: u16): u16 {
    x + y
}

entry fun sub(x: u16, y: u16): u16 {
    x - y
}

entry fun div(x: u16, y: u16): u16 {
    x / y
}

entry fun mul(x: u16, y: u16): u16 {
    x * y
}

entry fun mod_(x: u16, y: u16): u16 {
    x % y
}
