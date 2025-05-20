module 0x01::uint_64;

public fun deref_u64(x: u64): u64 {
  let y = &x;
  *y
}

public fun deref_u64_ref(y: &u64): u64 {
  *y
}

public fun call_deref_u64_ref(x: u64): u64 {
    deref_u64_ref(&x)
}

public fun deref_nested_u64(x: u64): u64 {
    let y = &x;
    let z = &*y;
    *z
}
