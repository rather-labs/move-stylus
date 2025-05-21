module 0x01::uint_256;

public fun deref_u256(x: u256): u256 {
  let y = &x;
  *y
}

public fun deref_u256_ref(y: &u256): u256 {
  *y
}

public fun call_deref_u256_ref(x: u256): u256 {
    deref_u256_ref(&x)
}

public fun deref_nested_u256(x: u256): u256 {
    let y = &x;
    let z = &*y;
    *z
}
