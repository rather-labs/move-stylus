module 0x01::hello_world;

public fun write_mut_ref( ): u8 {
 let mut x = 1;
 let y = &mut x;
 *y = 2;
 *y
}
