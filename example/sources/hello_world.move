module 0x01::hello_world;

// public fun mut_ref() {
//     let mut x = 1;
//     let y = &mut x;
//     *y = 2;
// }

public fun deref_mut_arg(x: &mut u8 ): u8 {
 *x
}

public fun write_mut_ref(x: &mut u128 ): u128 {
 *x = 1;
 *x
}


// public fun update_first_element(v: &mut vector<u64>) {
//     let new_val = 42;
//     let elem_ref = &mut v[0];
//     *elem_ref = new_val;
// }