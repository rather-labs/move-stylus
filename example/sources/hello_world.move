module 0x01::hello_world;

public fun test_match_in_loop(): u8 {
    let mut i = 0;
    let mut acc = 0;
    while (i < 10) {
        match (i) {
            1 => acc = acc + 1,
            2 => acc = acc + 2,
            _ => acc = acc + 3,
        };
        i = i + 1;
    };

    acc
}