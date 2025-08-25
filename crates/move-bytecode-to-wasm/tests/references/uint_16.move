module 0x01::uint_16;

public fun deref_u16(x: u16): u16 {
  let y = &x;
  *y
}

public fun deref_u16_ref(y: &u16): u16 {
  *y
}

public fun identity_u16_ref(x: &u16): &u16 {
    x
}

public fun call_deref_u16_ref(x: u16): u16 {
  deref_u16_ref(&x)
}

public fun deref_nested_u16(x: u16): u16 {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut u16 ): u16 {
 *x
}

public fun write_mut_ref(x: &mut u16 ): u16 {
 *x = 1;
 *x
}


public fun miscellaneous_0(): vector<u16> {
 let mut x = 1;
 let y = x;
 x = 2;
 let w = x;
 x = 99;
 let z = &mut x;
 *z = 3;
 vector[y, w, *z]
}

public fun miscellaneous_1():  vector<u16> {
  let mut x = 1;
  let y = x;
  x = 3;
  let z =  &mut x;
  let w = *z;
  *z = 2;
  vector[y, *z, w]
}

public fun freeze_ref(y: u16 ): u16 {
    let mut x = 1;
    let x_mut_ref: &mut u16 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u16 = freeze(x_mut_ref); 
    *x_frozen_ref
}