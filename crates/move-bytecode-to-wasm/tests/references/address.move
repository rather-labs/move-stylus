module 0x01::ref_address;

public fun deref_address(x: address): address {
  let y = &x;
  *y
}

public fun deref_address_ref(y: &address): address {
  *y
}

public fun call_deref_address_ref(x: address): address {
    deref_address_ref(&x)
}

public fun deref_nested_address(x: address): address {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut address ): address {
 *x
}

public fun write_mut_ref(x: &mut address ): address {
 *x = @0x7890abcdef1234567890abcdef1234567890abcd;
 *x
}
