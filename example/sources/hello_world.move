module 0x01::hello_world;

public fun miscellaneous_0(): vector<bool> {
 let mut x = true;
 let y = &mut x;
 *y = false;
 vector[*y, x]
}

public fun miscellaneous_1(): vector<bool> {
  let mut x = true;
  let y = x;
  x = false;
  let z =  &mut x;
  let w = *z;
  *z = true;
  vector[y, *z, w]
}