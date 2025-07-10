module 0x01::hello_world;

public fun test_branch(x: u8): u8 {
    /*
    match (x) {
        1 => 42,
        2 => 55,
        4 => 67,
        _ => 20
    }*/
    if (x == 0) {
        42
    } else {
        55
    }
}
