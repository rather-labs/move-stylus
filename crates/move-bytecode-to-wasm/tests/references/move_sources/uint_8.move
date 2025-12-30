module 0x01::references_uint_8;

entry fun deref_u8(x: u8): u8 {
  let y = &x;
  *y
}

entry fun deref_u8_ref(y: &u8): u8 {
  *y
}

entry fun identity_u8_ref(x: &u8): &u8 {
    x
}

entry fun call_deref_u8_ref(x: u8): u8 {
  deref_u8_ref(&x)
}

entry fun deref_nested_u8(x: u8): u8 {
    let y = &x;
    let z = &*y;
    *z
}

#[allow(unused_mut_parameter)]
entry fun deref_mut_arg(x: &mut u8 ): u8 {
 *x
}

entry fun write_mut_ref(x: &mut u8 ): u8 {
 *x = 1;
 *x
}


entry fun miscellaneous_0(): vector<u8> {
 let mut x = 1;
 let y = x;
 x = 2;
 let w = x;
 x = 99;
 let z = &mut x;
 *z = 3;
 vector[y, w, *z]
}

entry fun miscellaneous_1():  vector<u8> {
  let mut x = 1;
  let y = x;
  x = 3;
  let z =  &mut x;
  let w = *z;
  *z = 2;
  vector[y, *z, w]
}

entry fun freeze_ref(y: u8): u8 {
    let mut x = 1;
    let x_mut_ref: &mut u8 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u8 = freeze(x_mut_ref);
    *x_frozen_ref
}

entry fun unpack_ref_u8_misc(x: &u8, y: &u8, z: &u8): u8 {
  *x + *y + *z
}