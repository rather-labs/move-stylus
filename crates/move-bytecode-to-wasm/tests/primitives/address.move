module 0x01::address_type;

const ADDRESS_AS_CONST: address = @0x01;

public fun get_constant(): address {
  ADDRESS_AS_CONST
}

public fun get_constant_local(): address {
  let x: address = ADDRESS_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: address): address {
  let x: address = @0xFF;
  let y: address = @0x11;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): (address, address) {
  let x: address = @0x01;

  let y = x; // copy
  let mut z = x; // move
  identity(y);
  identity(z);

  z = @0x22;
  (y, z)
}

public fun echo(x: address): address {
  identity(x)
}

public fun echo_2(x: address, y: address): address {
  identity_2(x, y)
}

fun identity(x: address): address {
  x
}

fun identity_2(_x: address, y: address): address {
  y
}
