module 0x01::ref_address;

public fun deref_address(x: address): address {
  let y = &x;
  *y
}

public fun deref_address_ref(y: &address): address {
  *y
}

public fun call_deref_address_ref(x: address): address {
    deref_address_ref(&x)
}

public fun deref_nested_address(x: address): address {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut address ): address {
 *x
}

public fun write_mut_ref(x: &mut address ): address {
 *x = @0x01;
 *x
}

public fun mut_borrow_local(): address {
 let mut x = @0x01;
 let y = &mut x;
 *y = @0x02;
 *y
}

public fun miscellaneous_0(): vector<address> {
 let mut x = @0x01;
 let y = x;
 x = @0x02;
 let w = x;
 vector[y, w, x]
}

public fun miscellaneous_1():  vector<address> {
  let mut x = @0x01;
  let y = x;
  x = @0x02;
  let z =  &mut x;
  let w = *z;
  *z = @0x03;
  vector[y, *z, w]
}


public fun freeze_ref(y: address): address {
    let mut x = @0x01;
    let x_mut_ref: &mut address = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &address = freeze(x_mut_ref); 
    *x_frozen_ref
}
