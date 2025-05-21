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
