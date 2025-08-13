module hello_world::features_2;

use hello_world::other_mod::{generic_identity, generic_identity_two_types};

// Usage of generic functions
public fun echo_with_generic_function_u16(x: u16): u16 {
    generic_identity(x)
}

public fun echo_with_generic_function_vec32(x: vector<u32>): vector<u32> {
    generic_identity(x)
}

public fun echo_with_generic_function_u16_vec32(x: u16, y: vector<u32>): (u16, vector<u32>) {
    generic_identity_two_types(x, y)
}

public fun echo_with_generic_function_address_vec128(x: address, y: vector<u128>): (address, vector<u128>) {
    generic_identity_two_types(x, y)
}
