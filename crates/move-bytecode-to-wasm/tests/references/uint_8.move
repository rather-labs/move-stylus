module 0x01::uint_8;

public fun deref_u8(x: u8): u8 {
  let y = &x;
  *y
}

public fun deref_u8_ref(y: &u8): u8 {
  *y
}

public fun call_deref_u8_ref(x: u8): u8 {
  deref_u8_ref(&x)
}

public fun deref_nested_u8(x: u8): u8 {
    let y = &x;
    let z = &*y;
    *z
}
