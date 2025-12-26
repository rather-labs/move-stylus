module 0x01::references_uint_32;

entry fun deref_u32(x: u32): u32 {
  let y = &x;
  *y
}

entry fun deref_u32_ref(y: &u32): u32 {
  *y
}

entry fun identity_u32_ref(x: &u32): &u32 {
    x
}

entry fun call_deref_u32_ref(x: u32): u32 {
  deref_u32_ref(&x)
}

entry fun deref_nested_u32(x: u32): u32 {
    let y = &x;
    let z = &*y;
    *z
}


#[allow(unused_mut_parameter)]
entry fun deref_mut_arg(x: &mut u32 ): u32 {
 *x
}

entry fun write_mut_ref(x: &mut u32 ): u32 {
 *x = 1;
 *x
}


entry fun miscellaneous_0(): vector<u32> {
 let mut x = 1;
 let y = x;
 x = 2;
 let w = x;
 x = 99;
 let z = &mut x;
 *z = 3;
 vector[y, w, *z]
}

entry fun miscellaneous_1():  vector<u32> {
  let mut x = 1;
  let y = x;
  x = 3;
  let z =  &mut x;
  let w = *z;
  *z = 2;
  vector[y, *z, w]
}

entry fun freeze_ref(y: u32): u32 {
    let mut x = 1;
    let x_mut_ref: &mut u32 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u32 = freeze(x_mut_ref);
    *x_frozen_ref
}
