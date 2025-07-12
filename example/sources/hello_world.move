module 0x01::hello_world;

// public fun test_branch(x: u8): u8 {
//     /*
//     match (x) {
//         1 => 42,
//         2 => 55,
//         4 => 67,
//         _ => 20
//     }*/
//     if (x == 0) {
//         42
//     } else {
//         55
//     }
// }


public fun test_loop(x: u8): u8 {
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