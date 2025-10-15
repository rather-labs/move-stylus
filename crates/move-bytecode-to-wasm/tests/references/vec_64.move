module 0x01::vec_64;

entry fun deref(x: vector<u64>): vector<u64> {
  let y = &x;
  *y
}

entry fun deref_arg(y: &vector<u64>): vector<u64> {
  *y
}

entry fun identity_vec_ref(x: &vector<u64>): &vector<u64> {
    x
}

entry fun call_deref_arg(x: vector<u64>): vector<u64> {
  deref_arg(&x)
}

entry fun dummy(_v: &vector<u64>) {
    // Does nothing, but forces a borrow
}

entry fun call_dummy(v: vector<u64>) {
    dummy(&v);
}

entry fun vec_from_element(index: u64): vector<u64> {
    let v = vector[10u64, 20u64];
    let x = v[index];  
    vector[x]
}

entry fun get_element_vector(index: u64): vector<u64> {
    let v = vector[vector[10u64, 20u64], vector[30u64, 40u64]];
    let x = v[index];  
    x
}

entry fun deref_mut_arg(x: &mut vector<u64> ): vector<u64> {
 *x
}

entry fun write_mut_ref(x: &mut vector<u64> ): vector<u64> {
 *x = vector<u64>[1, 2, 3];
 *x
}

entry fun miscellaneous_0(): vector<u64> {
 let mut x = vector<u64>[1, 2, 3];
 let y = &mut x;
 *y = vector<u64>[4, 5, 6];
 vector[y[0], y[1], x[0]]
}

entry fun miscellaneous_1(): vector<u64> {
    let v = vector[vector[10u64, 20u64], vector[30u64, 40u64]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}

entry fun miscellaneous_2(): vector<u64> {
 let mut x = vector<u64>[1, 2, 3];
 let y =  x;
 x = vector<u64>[4, 5, 6];
 let w = x;
 let z = &mut x;
 *z = vector<u64>[7, 8, 9];
 let v = *z;
 vector[y[0], w[0], v[0]]
}

entry fun freeze_ref(y: vector<u64>): vector<u64> {
    let mut x = vector<u64>[1, 2, 3];
    let x_mut_ref: &mut vector<u64> = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &vector<u64> = freeze(x_mut_ref); 
    *x_frozen_ref
}

entry fun miscellaneous_3(x: vector<u64>): vector<u64> {
  let mut y = x;
  let a = &mut y[0];
  let b = *a;
  *a = 99;
  *vector::borrow_mut(&mut y, 1) = b;
  y
}


entry fun miscellaneous_4(): vector<u64> {
  let mut x = vector[vector[1u64, 2u64], vector[3u64, 4u64]]; // x = [ [1, 2], [3, 4] ]
  let a = &mut x[0]; // a = vector[1, 2]
  *vector::borrow_mut(a, 1) = 12; // a = vector[1, 12]
  let b = *a; // b = vector[1, 12]
  let mut c = b; // c = vector[1, 12]
  *vector::borrow_mut(a, 0) = 11; // a = vector[11, 12]
  *vector::borrow_mut(a, 1) = 112; // a = vector[11, 112]
  *vector::borrow_mut(&mut c, 0) = 111;  // c = vector[111, 12]
  vector[b[0], b[1], c[0], c[1], a[0], a[1]]
}