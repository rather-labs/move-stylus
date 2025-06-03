module 0x01::hello_world;

public fun write_mut_ref(x: &mut address ): address {
 *x = @0x7890abcdef1234567890abcdef1234567890abcd;
 *x
}
