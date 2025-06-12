module 0x01::hello_world;

public fun test(x: u32): vector<u32> {
  let mut y =  x;
  let mut z = y;
  let w = &mut z;
  y = 2;
  z = 3;
  vector[x, y, z, *w]
}
