module 0x01::bool;

public fun deref_bool(x: bool): bool {
  let y = &x;
  *y
}

public fun deref_bool_ref(y: &bool): bool {
  *y
}

public fun identity_bool_ref(x: &bool): &bool {
    x
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

public fun miscellaneous_0(): vector<bool> {
 let mut x = true;
 let w = x;
 let y = &mut x;
 *y = false;
 vector[*y, w, x]
}

public fun miscellaneous_1(): vector<bool> {
  let mut x = true;
  let y = x;
  x = false;
  let z =  &mut x;
  let w = *z;
  *z = true;
  vector[y, *z, w]
}

public fun freeze_ref(y: bool): bool {
    let mut x = true;
    let x_mut_ref: &mut bool = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &bool = freeze(x_mut_ref); 
    *x_frozen_ref
}