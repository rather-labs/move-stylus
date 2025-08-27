module 0x00::generic_functions_args;

public fun test_forward(x: &u32, b: bool): (bool, &u32) {
    if (b) {
        test(x, b)
    } else {
        test_inv(b, x)
    }
}

// This ones work fine.
public fun test(x: &u32, b: bool): (bool, &u32) {
    (b, x)
}

public fun test_inv(b: bool, x: &u32): (bool, &u32) {
    (b, x)
}

public fun test_mix(x: &u32, b: bool, v: u64, w: &u64): (bool, &u32, u64, &u64) {
    (b, x, v, w)
}