module 0x01::hello_world;

<<<<<<< HEAD
public fun vec_from_int(x: u32, y: u32): vector<u32> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec(x: vector<u32>, y: vector<u32>): vector<vector<u32>> {
  let z = vector[x, y];
  z
}


public fun vec_mix(x: vector<u32>, y: u32): vector<vector<u32>> {
  let w = vector[y, y];
  let z = vector[x, w];
  z
}
=======
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

>>>>>>> main
