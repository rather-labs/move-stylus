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
    
//     if (i < 11) {
//         42
//     } else {
//         55
//     }
// }



public struct Foo has drop, copy {
    q: address,
    r: vector<u32>,
    s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    bar: Bar,
    baz: Baz,
}

// Static abi sub-struct
public struct Bar has drop, copy {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
public struct Baz has drop, copy {
    a: u16,
    b: vector<u256>,
}

public fun deref_struct(x: Foo): Foo {
  let y = &x;
  *y
}
