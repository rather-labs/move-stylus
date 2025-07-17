module 0x01::control_flow;

public fun simple_loop(x: u8): u8 {
    let mut i = 0;
    while (i < x) {
        i = i + 1;
    };
    i
}

public fun nested_loop(x: u8): u8 {
    let mut i = 0;
    let mut acc = 0;
    while (i < x) {
        let mut j = 0;
        while (j < i) {
            j = j + 1;
            acc = acc + j;
        };
        i = i + 1;
    };
    acc
}

public fun loop_with_break(x: u8): u8 {
    let mut i = 0;
    let mut acc = 0;
    while (true) {
        if (i > x) {
            break
        };
        i = i + 1;
        acc = acc + i;
    };
    acc
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

public fun early_return(x: u8): u8 {
    if (x > 100) {
        return 255
    };
    x + 1
}

public fun crazy_loop(mut i: u8): u8 {
    let mut acc = 0;
    while ( true ) {
        i = i + 1;
        if (i > 10) {
            break
        };
        acc = acc + i;
    };

    let mut j = 0;

    while (j < 10) {
        j = j + i;
        acc = acc + j;
    };
    acc
}

public fun test_match(x: u8): u8 {
    match (x) {
        1 => 44,
        2 => 55,
        3 => 66,
        _ => 0
    }

}
