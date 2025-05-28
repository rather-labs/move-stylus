module 0x01::hello_world;

public fun write_mut_ref(x: &mut vector<u8> ): vector<u8> {
 *x = vector<u8>[1, 2, 3];
 *x
}
// public fun write_mut_ref_2( ) {
//  let mut x = vector<u8>[1, 2, 3];
//  let y = &mut x;
//  *y = vector<u8>[4, 5, 6, 7];
// }