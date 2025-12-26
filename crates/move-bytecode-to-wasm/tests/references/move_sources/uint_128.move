module 0x01::references_uint_128;

entry fun deref_u128(x: u128): u128 {
  let y = &x;
  *y
}

entry fun deref_u128_ref(y: &u128): u128 {
  *y
}

entry fun identity_u128_ref(x: &u128): &u128 {
    x
}

entry fun call_deref_u128_ref(x: u128): u128 {
    deref_u128_ref(&x)
}

entry fun deref_nested_u128(x: u128): u128 {
    let y = &x;
    let z = &*y;
    *z
}

#[allow(unused_mut_parameter)]
entry fun deref_mut_arg(x: &mut u128 ): u128 {
 *x
}

entry fun write_mut_ref(x: &mut u128 ): u128 {
 *x = 1;
 *x
}


entry fun miscellaneous_0(): vector<u128> {
 let mut x = 1;
 let y = x;
 x = 2;
 let w = x;
 x = 99;
 let z = &mut x;
 *z = 3;
 vector[y, w, *z]
}

entry fun miscellaneous_1():  vector<u128> {
  let mut x = 1;
  let y = x;
  x = 3;
  let z =  &mut x;
  let w = *z;
  *z = 2;
  vector[y, *z, w]
}

entry fun freeze_ref(y: u128): u128 {
    let mut x = 1;
    let x_mut_ref: &mut u128 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u128 = freeze(x_mut_ref);
    *x_frozen_ref
}
