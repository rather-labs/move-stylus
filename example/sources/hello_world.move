module 0x01::hello_world;

public fun vec_push_back(x: vector<u32>, y: u32): vector<u32> {
  let mut z = x;
  z.push_back(y);
  z
}
