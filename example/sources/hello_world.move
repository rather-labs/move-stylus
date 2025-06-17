module 0x01::hello_world;

public fun miscellaneous_0(x: vector<vector<u32>>, y: u32): vector<vector<u32>> {
  let mut w = x;
  w[0].push_back(y);
  let mut a = w[0];
  a.swap(0, 1);
  a.pop_back();
  a.push_back(y);
  let z = vector[w[0], a];
  z
}
