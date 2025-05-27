module 0x01::hello_world;

// Forces the compiler to store literals on locals
public fun get_copied_local(z: u64): u64 {
  let x = z;
  x
}


