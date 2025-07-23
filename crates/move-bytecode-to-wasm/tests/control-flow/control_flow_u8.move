module 0x01::control_flow_u8;

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

public fun conditional_return(x: u8): u8 {
    if (x > 100) {
        return 255
    } else {
        if (x % 17 == 0) {
            x + 200
        } else {
            x - 20
        }
    }
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

public fun test_match_in_loop(): u8 {
    let mut i = 0;
    let mut acc = 0;
    loop {
        i = i + 1;
        match (i) {
            1 => acc = acc + 1,
            2 => acc = acc + 2,
            3 => acc = acc + 3,
            4 => acc = acc + 4,
            _ => acc = acc + 4,
        };
        if (i > 1) {
            break
        };
    };

    acc
}

public fun test_labeled_loops(x: u8): u8 {
    let mut outer_count = 1;
    let mut inner_count = 1;

    'outer: loop {
        outer_count = outer_count + 1;

        'inner: while (inner_count < x) {
            inner_count = inner_count + 1;

            if (inner_count % 17 == 0) {
                break 'outer 
            };

           if (inner_count % 13 == 0) {
                outer_count = outer_count + 1;
                continue 'outer
            };
        };

        if (outer_count > 23) {
            break  
        };
    };

    outer_count + inner_count
}


public fun check_even(i: u8): u8 {
    if (i % 2 == 0) {
        42
    } else {
        55
    }
}

public fun check_even_after_loop(x: u8): u8 {
    let mut i = 0;
    while (i < x) {
        i = i + 1;
    };
    
   let j = check_even(i);
   j
}
