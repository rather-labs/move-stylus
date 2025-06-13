module 0x01::vec_256;

public fun deref(x: vector<u256>): vector<u256> {
  let y = &x;
  *y
}

public fun deref_arg(y: &vector<u256>): vector<u256> {
  *y
}

public fun call_deref_arg(x: vector<u256>): vector<u256> {
  deref_arg(&x)
}

public fun dummy(_v: &vector<u256>) {
    // Does nothing, but forces a borrow
}

public fun call_dummy(v: vector<u256>) {
    dummy(&v);
}

public fun vec_from_element(index: u64): vector<u256> {
    let v = vector[10u256, 20u256];
    let x = v[index];  
    vector[x]
}

public fun get_element_vector(index: u64): vector<u256> {
    let v = vector[vector[10u256, 20u256], vector[30u256, 40u256]];
    let x = v[index];  
    x
}

public fun deref_mut_arg(x: &mut vector<u256> ): vector<u256> {
 *x
}

public fun write_mut_ref(x: &mut vector<u256> ): vector<u256> {
 *x = vector<u256>[1, 2, 3];
 *x
}


public fun miscellaneous_0(): vector<u256> {
 let mut x = vector<u256>[1, 2, 3];
 let y = &mut x;
 *y = vector<u256>[4, 5, 6];
 vector[y[0], y[1], x[0]]
}

public fun miscellaneous_1(): vector<u256> {
    let v = vector[vector[10u256, 20u256], vector[30u256, 40u256]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}

public fun miscellaneous_2(): vector<u256> {
 let mut x = vector<u256>[1, 2, 3];
 let y =  x;
 x = vector<u256>[4, 5, 6];
 let w = x;
 let z = &mut x;
 *z = vector<u256>[7, 8, 9];
 let v = *z;
 vector[y[0], w[0], v[0]]
}

public fun freeze_ref(y: vector<u256>): vector<u256> {
    let mut x = vector<u256>[1, 2, 3];
    let x_mut_ref: &mut vector<u256> = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &vector<u256> = freeze(x_mut_ref); 
    *x_frozen_ref 
}

public fun miscellaneous_3(x: vector<u256>): vector<u256> {
  let mut y = x;
  let a = &mut y[0];
  let b = *a;
  *a = 99;
  *vector::borrow_mut(&mut y, 1) = b;
  y
}


public fun miscellaneous_4(): vector<u256> {
  let mut x = vector[vector[1u256, 2u256], vector[3u256, 4u256]]; // x = [ [1, 2], [3, 4] ]
  let a = &mut x[0]; // a = vector[1, 2]
  *vector::borrow_mut(a, 1) = 12; // a = vector[1, 12]
  let b = *a; // b = vector[1, 12]
  let mut c = b; // c = vector[1, 12]
  *vector::borrow_mut(a, 0) = 11; // a = vector[11, 12]
  *vector::borrow_mut(a, 1) = 112; // a = vector[11, 112]
  *vector::borrow_mut(&mut c, 0) = 111;  // c = vector[111, 12]
  vector[b[0], b[1], c[0], c[1], a[0], a[1]]
}