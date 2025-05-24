module 0x01::hello_world;


public fun deref_u64(x: u64): u64 {
  let y = &x;
  *y
}

