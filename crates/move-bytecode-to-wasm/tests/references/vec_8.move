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
    dummy(&v); // this throws an error
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

public fun miscellaneous(): vector<u8> {
    let v = vector[vector[10u8, 20u8], vector[30u8, 40u8]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}