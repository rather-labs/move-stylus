module 0x01::bool_type;

const BOOL_AS_CONST: bool = true;

public fun get_constant(): bool {
  BOOL_AS_CONST
}

public fun get_constant_local(): bool {
  let x: bool = BOOL_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: bool): bool {
  let x: bool = true;
  let y: bool = false;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): (bool, bool) {
  let x: bool = true;
  
  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = false;
  (y, z)
}

public fun echo(x: bool): bool {
  identity(x)
}

public fun echo_2(x: bool, y: bool): bool {
  identity_2(x, y)
}

fun identity(x: bool): bool {
  x
}

fun identity_2(_x: bool, y: bool): bool {
  y
}

public fun not_true(): bool {
  !BOOL_AS_CONST
}

public fun not(x: bool): bool {
  !x
}
