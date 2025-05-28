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

public fun miscellaneous(): vector<u256> {
    let v = vector[vector[10u256, 20u256], vector[30u256, 40u256]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}


public fun write_mut_ref(x: &mut vector<u256> ): vector<u256> {
 *x = vector<u256>[1, 2, 3];
 *x
}