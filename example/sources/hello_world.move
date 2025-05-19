module 0x01::hello_world;

public fun fn_1(): u8 {
    let v = vector[10u8, 20u8];
    let x = v[0];  
    x
}

public fun fn_2(): vector<u8> {
    let v = vector[vector[10u8, 20u8], vector[30u8, 40u8]];
    let x = v[0];  
    x
}