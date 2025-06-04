module 0x01::uint_32;

public fun deref_u32(x: u32): u32 {
  let y = &x;
  *y
}

public fun deref_u32_ref(y: &u32): u32 {
  *y
}

public fun call_deref_u32_ref(x: u32): u32 {
  deref_u32_ref(&x)
}

public fun deref_nested_u32(x: u32): u32 {
    let y = &x;
    let z = &*y;
    *z
}


public fun deref_mut_arg(x: &mut u32 ): u32 {
 *x
}

public fun write_mut_ref(x: &mut u32 ): u32 {
 *x = 1;
 *x
}

public fun mut_borrow_local(): u32 {
 let mut x = 1;
 let y = &mut x;
 *y = 2;
 *y
}

public fun freeze_ref(y: u32): u32 {
    let mut x = 1;
    let x_mut_ref: &mut u32 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u32 = freeze(x_mut_ref); 
    *x_frozen_ref
}