module 0x01::hello_world;

// Forces the compiler to store literals on locals
public fun get_literal(): vector<u32> {
  vector[1u32, 2u32, 3u32]
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<u32> {
  let x: vector<u32> = vector[1u32, 2u32, 3u32];
  let y = x; 
  let _z = x; 
  y
}

public fun echo(x: vector<u32>): vector<u32> {
  x
}

