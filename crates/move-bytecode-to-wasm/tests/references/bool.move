module 0x01::bool;

public fun deref_bool(x: bool): bool {
  let y = &x;
  *y
}

public fun deref_bool_ref(y: &bool): bool {
  *y
}

public fun call_deref_bool_ref(x: bool): bool {
  deref_bool_ref(&x)
}

public fun deref_nested_bool(x: bool): bool {
    let y = &x;
    let z = &*y;
    *z
}
