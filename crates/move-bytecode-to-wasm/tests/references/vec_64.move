module 0x01::vec_64;

public fun deref(x: vector<u64>): vector<u64> {
  let y = &x;
  *y
}

public fun deref_arg(y: &vector<u64>): vector<u64> {
  *y
}

public fun call_deref_arg(x: vector<u64>): vector<u64> {
  deref_arg(&x)
}

public fun dummy(_v: &vector<u64>) {
    // Does nothing, but forces a borrow
}

public fun call_dummy(v: vector<u64>) {
    dummy(&v);
}

public fun vec_from_element(index: u64): vector<u64> {
    let v = vector[10u64, 20u64];
    let x = v[index];  
    vector[x]
}

public fun get_element_vector(index: u64): vector<u64> {
    let v = vector[vector[10u64, 20u64], vector[30u64, 40u64]];
    let x = v[index];  
    x
}

public fun miscellaneous(): vector<u64> {
    let v = vector[vector[10u64, 20u64], vector[30u64, 40u64]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}