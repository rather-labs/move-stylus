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