module 0x01::uint_16;

public fun deref_u16(x: u16): u16 {
  let y = &x;
  *y
}

public fun deref_u16_ref(y: &u16): u16 {
  *y
}

public fun call_deref_u16_ref(x: u16): u16 {
  deref_u16_ref(&x)
}

public fun deref_nested_u16(x: u16): u16 {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut u16 ): u16 {
 *x
}

public fun write_mut_ref(x: &mut u16 ): u16 {
 *x = 1;
 *x
}

public fun mut_borrow_local(): u16 {
 let mut x = 1;
 let y = &mut x;
 *y = 2;
 *y
}