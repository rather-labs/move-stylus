module 0x01::vec_8;

public fun deref(x: vector<u8>): vector<u8> {
  let y = &x;
  *y
}

public fun deref_arg(y: &vector<u8>): vector<u8> {
  *y
}

public fun call_deref_arg(x: vector<u8>): vector<u8> {
  deref_arg(&x)
}

public fun dummy(_v: &vector<u8>) {
    // Does nothing, but forces a borrow
}

public fun call_dummy(v: vector<u8>) {
    dummy(&v); 
}

public fun vec_from_element(index: u64): vector<u8> {
    let v = vector[10u8, 20u8];
    let x = v[index];  
    vector[x]
}

public fun get_element_vector(index: u64): vector<u8> {
    let v = vector[vector[10u8, 20u8], vector[30u8, 40u8]];
    let x = v[index];  
    x
}

public fun deref_mut_arg(x: &mut vector<u8> ): vector<u8> {
 *x
}

public fun write_mut_ref(x: &mut vector<u8> ): vector<u8> {
 *x = vector<u8>[1, 2, 3];
 *x 
}


public fun miscellaneous_0(): vector<u8> {
 let mut x = vector<u8>[1, 2, 3];
 let y = &mut x;
 *y = vector<u8>[4, 5, 6];
 vector[y[0], y[1], x[0]]
}

public fun miscellaneous_1(): vector<u8> {
    let v = vector[vector[10u8, 20u8], vector[30u8, 40u8]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}

public fun miscellaneous_2(): vector<u8> {
 let mut x = vector<u8>[1, 2, 3];
 let y =  x;
 x = vector<u8>[4, 5, 6];
 let w = x;
 let z = &mut x;
 *z = vector<u8>[7, 8, 9];
 let v = *z;
 vector[y[0], w[0], v[0]]
}

public fun freeze_ref(y: vector<u8>): vector<u8> {
    let mut x = vector<u8>[1, 2, 3];
    let x_mut_ref: &mut vector<u8> = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &vector<u8> = freeze(x_mut_ref); 
    *x_frozen_ref
}

public fun vec_mut_borrow(x: vector<u8>): vector<u8> {
  let mut y = x;
  let a = &mut y[0];
  let b = *a;
  *a = 0;
  *vector::borrow_mut(&mut y, 1) = b;
  y
}