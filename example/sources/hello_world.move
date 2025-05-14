module 0x01::hello_world;

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

public fun demo_ref(x: u8): u8 {
  let y = &x;
  *y
}

public fun demo_ref_vec(x: vector<u32>): vector<u32> {
  let y = &x;
  let z = vector[1u32, 2, 3];
  let w = &z;
  let c = 123;
  let d = &c;   
  *w
}

public fun demo_multiple_refs(x: vector<u32>, y: vector<u32>): (vector<u32>, vector<u32>) {
  let a = &x;
  let b = &y;
  let v1 = *b;
  let v2 = *a;
  (v1, v2)
}


