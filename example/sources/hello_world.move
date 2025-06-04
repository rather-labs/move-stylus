module 0x01::hello_world;

public fun freeze_ref(y: u8): u8 {
    let mut x = 1;
    let x_mut_ref: &mut u8 = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &u8 = freeze(x_mut_ref); 
    *x_frozen_ref
}

