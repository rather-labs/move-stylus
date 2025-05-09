module 0x01::uint_64;

const INT_AS_CONST: u64 = 6464;

public fun get_constant(): u64 {
  INT_AS_CONST
}

public fun get_constant_local(): u64 {
  let x: u64 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: u64): u64 {
  let x: u64 = 100;
  let y: u64 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): (u64, u64) {
  let x: u64 = 100;
  
  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
  (y, z)
}

public fun echo(x: u64): u64 {
  identity(x)
}

public fun echo_2(x: u64, y: u64): u64 {
  identity_2(x, y)
}

fun identity(x: u64): u64 {
  x
}

fun identity_2(_x: u64, y: u64): u64 {
  y
}
