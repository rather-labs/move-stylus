module 0x01::uint_128;

public fun deref_u128(x: u128): u128 {
  let y = &x;
  *y
}

public fun deref_u128_ref(y: &u128): u128 {
  *y
}

public fun identity_u128_ref(x: &u128): &u128 {
    x
}

public fun call_deref_u128_ref(x: u128): u128 {
    deref_u128_ref(&x)
}

public fun deref_nested_u128(x: u128): u128 {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut u128 ): u128 {
 *x
}

public fun write_mut_ref(x: &mut u128 ): u128 {
 *x = 1;
 *x
}


public fun miscellaneous_0(): vector<u128> {
 let mut x = 1;
 let y = x;
 x = 2;
 let w = x;
 x = 99;
 let z = &mut x;
 *z = 3;
 vector[y, w, *z]
}

public fun miscellaneous_1():  vector<u128> {
  let mut x = 1;
  let y = x;
  x = 3;
  let z =  &mut x;
  let w = *z;
  *z = 2;
  vector[y, *z, w]
}

public fun freeze_ref(y: u128): u128 {
    let mut x = 1;
    let x_mut_ref: &mut u128 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u128 = freeze(x_mut_ref); 
    *x_frozen_ref
}