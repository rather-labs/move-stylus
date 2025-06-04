module 0x01::hello_world;

public fun test_mut_ref_chain(): u8 {
    let mut x = 1;                    // x = value1
    let y = &mut x;                    // y = &mut x
    *y = 2;                           // *y = value2
    assert!(*y == 2, 100);             // validate *y == value2
    change(y);                         // call function to mutate
    assert!(*y == 3, 102);             // validate *y == value3
    x
}

public fun change(y: &mut u8) {
    *y = 3;                           
}
