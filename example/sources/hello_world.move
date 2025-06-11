module 0x01::hello_world;

public fun vec_mut_borrow(): vector<u32> {
  let mut y = vector[1, 2, 3];
  let a = &mut y[0];
  let b = *a;
  *a = 0;
  *vector::borrow_mut(&mut y, 2) = b;
  y
}
