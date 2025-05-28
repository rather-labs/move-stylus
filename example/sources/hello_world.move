module 0x01::hello_world;

public fun write_mut_ref(x: &mut vector<u8> ) {
 *x = vector<u8>[1, 2, 3];
}
