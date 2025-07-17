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


// public fun test_loop(x: u8): u8 {
//     let mut i = 0;
//     while (i < x) {
//         i = i + 1;
//     };
//     i
// }

public fun test(): u8 {
    let mut i = 0;
    while ( true ) {
        i = i + 1;
        if (i > 10) {
            break
        }
    };

    i
}

// public fun test_crazy_loop_(): u8 {
//     let mut i = 0u8;
//     let mut j = 0u8;

//     while ( true ) {
//         i = i + 1;
//         if (i > 10) {
//             break
//         };
//     };

//     while (j < 10) {
//         j = j + i;
//     };
//     j
// }


// public fun nested_loop(x: u8): u8 {
//     let mut i = 0;
//     let mut acc = 0;
//     while (i < x) {
//         let mut j = 0;
//         while (j < i) {
//             j = j + 1;
//             acc = acc + j;
//         };
//         i = i + 1;
//     };
//     acc
// }

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
