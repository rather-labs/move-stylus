module stylus::test_scenario;
use stylus::tx_context::TxContext;

public native fun new_tx_context(): TxContext;

public native fun drop_storage_object<T: key>(storage_object: T);

public fun default_signer_address(): address {
    @0xbeef
}

public fun default_sender_address(): address {
    @0xcafe
}
