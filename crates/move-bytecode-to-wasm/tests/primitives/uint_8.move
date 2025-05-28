module 0x01::uint_8;

const INT_AS_CONST: u8 = 88;

public fun get_constant(): u8 {
  INT_AS_CONST
}

public fun get_constant_local(): u8 {
  let x: u8 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: u8): u8 {
  let x: u8 = 100;
  let y: u8 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): (u8, u8) {
  let x: u8 = 100;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

public fun echo(x: u8): u8 {
  identity(x)
}

public fun echo_2(x: u8, y: u8): u8 {
  identity_2(x, y)
}

fun identity(x: u8): u8 {
  x
}

fun identity_2(_x: u8, y: u8): u8 {
  y
}

public fun sum(x: u8, y: u8): u8 {
    x + y
}

public fun sub(x: u8, y: u8): u8 {
    x - y
}

public fun div(x: u8, y: u8): u8 {
    x / y
}

public fun mul(x: u8, y: u8): u8 {
    x * y
}

public fun mod_(x: u8, y: u8): u8 {
    x % y
}
