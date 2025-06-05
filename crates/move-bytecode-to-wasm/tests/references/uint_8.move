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

public fun deref_mut_arg(x: &mut u8 ): u8 {
 *x
}

public fun write_mut_ref(x: &mut u8 ): u8 {
 *x = 1;
 *x
}

public fun mut_borrow_local(z: &mut u8): u8 {
 let mut x = 1;
 let y = &mut x;
 *y = 2;
 *z = 3;
 *y
}

public fun freeze_ref(y: u8): u8 {
    let mut x = 1;
    let x_mut_ref: &mut u8 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u8 = freeze(x_mut_ref); 
    *x_frozen_ref
}