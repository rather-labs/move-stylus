module 0x01::vec_128;

// Forces the compiler to store literals on locals
public fun get_literal(): vector<u128> {
  vector[1u128, 2u128, 3u128]
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<u128> {
  let x: vector<u128> = vector[1u128, 2u128, 3u128];
  let y = x; 
  let _z = x; 
  y
}

public fun echo(x: vector<u128>): vector<u128> {
  x
}

