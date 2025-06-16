module 0x01::hello_world;

public fun vec_push_back(): vector<vector<u32>> {
  let mut x = vector[vector[1, 2], vector[3, 4]];
  x[0].push_back(5);
  x
}

public fun vec_push_back_2(): vector<u32> {
  let mut x = vector[1, 2, 3, 4];
  x.push_back(5);
  x
}

// public fun vec_pop_back(x: vector<u32>): vector<u32> {
//   let mut z = x;
//   z.pop_back();
//   z
// }