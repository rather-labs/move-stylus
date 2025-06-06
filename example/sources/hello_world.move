module 0x01::hello_world;

public fun vec_pop_back(x: vector<u32>): vector<u32> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}
