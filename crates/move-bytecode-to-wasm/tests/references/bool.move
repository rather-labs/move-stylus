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

public fun deref_mut_arg(x: &mut bool ): bool {
 *x
}

public fun write_mut_ref(x: &mut bool ): bool {
 *x = true;
 *x
}
