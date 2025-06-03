module 0x01::uint_128;

public fun deref_u128(x: u128): u128 {
  let y = &x;
  *y
}

public fun deref_u128_ref(y: &u128): u128 {
  *y
}

public fun call_deref_u128_ref(x: u128): u128 {
    deref_u128_ref(&x)
}

public fun deref_nested_u128(x: u128): u128 {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut u128 ): u128 {
 *x
}

public fun write_mut_ref(x: &mut u128 ): u128 {
 *x = 1;
 *x
}