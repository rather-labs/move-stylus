module 0x01::hello_world;

public fun dummy(_v: &vector<u8>) {
    // Does nothing, but forces a borrow
}

public fun miscellaneous_0(): vector<u8> {
    let v = vector[vector[10u8, 20u8], vector[30u8, 40u8]];
    dummy(&v[0]);
    let x = v[0]; 
    let y = x[1];
    vector[y, v[1][1]]
}