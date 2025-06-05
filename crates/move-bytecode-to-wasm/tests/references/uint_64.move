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

public fun deref_mut_arg(x: &mut u64 ): u64 {
 *x
}

public fun write_mut_ref(x: &mut u64 ): u64 {
 *x = 1;
 *x
}

public fun mut_borrow_local(): u64 {
 let mut x = 1;
 let y = &mut x;
 *y = 2;
 *y
}

public fun freeze_ref(y: u64): u64 {
    let mut x = 1;
    let x_mut_ref: &mut u64 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u64 = freeze(x_mut_ref); 
    *x_frozen_ref
}