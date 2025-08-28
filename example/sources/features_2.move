module hello_world::features_2;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;

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

public fun get_fresh_object_address(ctx: &mut TxContext): address {
    ctx.fresh_object_address()
}

public fun get_unique_ids(ctx: &mut TxContext): (UID, UID, UID) {
    (
        object::new(ctx),
        object::new(ctx),
        object::new(ctx),
    )
}

public fun get_unique_id(ctx: &mut TxContext): UID {
    object::new(ctx)
}
