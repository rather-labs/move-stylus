module stylus::object;

use stylus::tx_context::TxContext;

public struct ID has copy, drop, store {
    bytes: address,
}

public struct UID has store {
    id: ID,
}

/// Create a new object. Returns the `UID` that must be stored in a Sui object.
/// This is the only way to create `UID`s.
public fun new(ctx: &mut TxContext): UID {
    UID {
        id: ID { bytes: ctx.fresh_object_address() },
    }
}


public native fun compute_id(data: vector<u8>): UID;
