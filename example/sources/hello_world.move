module 0x01::hello_world;

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