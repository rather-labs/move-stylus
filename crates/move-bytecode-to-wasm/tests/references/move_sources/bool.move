module 0x01::references_bool;

entry fun deref_bool(x: bool): bool {
  let y = &x;
  *y
}

entry fun deref_bool_ref(y: &bool): bool {
  *y
}

entry fun identity_bool_ref(x: &bool): &bool {
    x
}

entry fun call_deref_bool_ref(x: bool): bool {
  deref_bool_ref(&x)
}

entry fun deref_nested_bool(x: bool): bool {
    let y = &x;
    let z = &*y;
    *z
}

#[allow(unused_mut_parameter)]
entry fun deref_mut_arg(x: &mut bool ): bool {
 *x
}

entry fun write_mut_ref(x: &mut bool ): bool {
 *x = true;
 *x
}

entry fun miscellaneous_0(): vector<bool> {
 let mut x = true;
 let w = x;
 let y = &mut x;
 *y = false;
 vector[*y, w, x]
}

entry fun miscellaneous_1(): vector<bool> {
  let mut x = true;
  let y = x;
  x = false;
  let z =  &mut x;
  let w = *z;
  *z = true;
  vector[y, *z, w]
}

entry fun freeze_ref(y: bool): bool {
    let mut x = true;
    let x_mut_ref: &mut bool = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &bool = freeze(x_mut_ref);
    *x_frozen_ref
}
