module 0x01::hello_world;

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