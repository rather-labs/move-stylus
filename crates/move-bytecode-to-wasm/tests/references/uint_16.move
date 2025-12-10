module 0x01::uint_16;

entry fun deref_u16(x: u16): u16 {
  let y = &x;
  *y
}

entry fun deref_u16_ref(y: &u16): u16 {
  *y
}

entry fun identity_u16_ref(x: &u16): &u16 {
    x
}

entry fun call_deref_u16_ref(x: u16): u16 {
  deref_u16_ref(&x)
}

entry fun deref_nested_u16(x: u16): u16 {
    let y = &x;
    let z = &*y;
    *z
}

#[allow(unused_mut_parameter)]
entry fun deref_mut_arg(x: &mut u16 ): u16 {
 *x
}

entry fun write_mut_ref(x: &mut u16 ): u16 {
 *x = 1;
 *x
}

entry fun miscellaneous_0(): vector<u16> {
 let mut x = 1;
 let y = x;
 x = 2;
 let w = x;
 x = 99;
 let z = &mut x;
 *z = 3;
 vector[y, w, *z]
}

entry fun miscellaneous_1():  vector<u16> {
  let mut x = 1;
  let y = x;
  x = 3;
  let z =  &mut x;
  let w = *z;
  *z = 2;
  vector[y, *z, w]
}

entry fun freeze_ref(y: u16 ): u16 {
    let mut x = 1;
    let x_mut_ref: &mut u16 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u16 = freeze(x_mut_ref);
    *x_frozen_ref
}
