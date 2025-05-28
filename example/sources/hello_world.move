module 0x01::hello_world;

public fun mut_ref(x: &mut u64) {
    *x = 1;
}
