module 0x01::hello_world;

public fun test_vec_pop_back(): u8 {
    let mut vec = vector[1, 2, 3];
    vec.pop_back()
}


