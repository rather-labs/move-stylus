module 0x01::control_flow;

public fun simple_loop(x: u8): u8 {
    let mut i = 0;
    while (i < x) {
        i = i + 1;
    };
    i
}

public fun misc_1(x: u8): u8 {
    let mut i = 0;
    while (i < x) {
        i = i + 1;
    };
    
    if (i < 11) {
        42
    } else {
        55
    }
}