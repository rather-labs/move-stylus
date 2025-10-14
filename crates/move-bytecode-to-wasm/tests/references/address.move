module 0x01::ref_address;

entry fun deref_address(x: address): address {
  let y = &x;
  *y
}

entry fun deref_address_ref(y: &address): address {
  *y
}

entry fun identity_address_ref(x: &address): &address {
    x
}

entry fun call_deref_address_ref(x: address): address {
    deref_address_ref(&x)
}

entry fun deref_nested_address(x: address): address {
    let y = &x;
    let z = &*y;
    *z
}

entry fun deref_mut_arg(x: &mut address ): address {
 *x
}

entry fun write_mut_ref(x: &mut address ): address {
 *x = @0x01;
 *x
}

entry fun mut_borrow_local(): address {
 let mut x = @0x01;
 let y = &mut x;
 *y = @0x02;
 *y
}

entry fun miscellaneous_0(): vector<address> {
 let mut x = @0x01;
 let y = x;
 x = @0x02;
 let w = x;
 vector[y, w, x]
}

entry fun miscellaneous_1():  vector<address> {
  let mut x = @0x01;
  let y = x;
  x = @0x02;
  let z =  &mut x;
  let w = *z;
  *z = @0x03;
  vector[y, *z, w]
}

entry fun freeze_ref(y: address): address {
    let mut x = @0x01;
    let x_mut_ref: &mut address = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &address = freeze(x_mut_ref); 
    *x_frozen_ref
}
