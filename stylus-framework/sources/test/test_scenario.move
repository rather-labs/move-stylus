module stylus::test_scenario;
use stylus::tx_context::TxContext;

/// Creates a new mock TxContext
public native fun new_tx_context(): TxContext;

/// This is used to drop storage objects.
///
/// This is needed when creating structs with the `key` ability. Once we are done using them, if
/// they are not dropped, Move will throw a compilation error telling us that we should do
/// something with the object.
public native fun drop_storage_object<T: key>(storage_object: T);

/// Sets the signer address (tx_origin).
public native fun set_signer_address(addr: address);

/// Sets the msg.sender address.
public native fun set_sender_address(addr: address);

/// Sets the block base fee.
public native fun set_block_basefee(base_fee: u256);

/// Sets the gas price.
public native fun set_gas_price(gas_price: u256);

/// Sets the block number.
public native fun set_block_number(block_number: u64);

/// Sets the gas limit.
public native fun set_gas_limit(gas_limit: u64);

/// Sets the block timestamp.
public native fun set_block_timestamp(block_timestamp: u64);

/// Sets the chain id.
public native fun set_chain_id(chain_id: u64);

/// Provides the default transaction signer (`tx_origin`) used in the test environment.
public fun default_signer(): address {
    @0xbeef
}

/// Provides the default transaction sender (`msg.sender`) used in the test environment.
public fun default_sender(): address {
    @0xcafe
}

/// Provides the default transaction base fee used in the test environment.
public fun default_base_fee(): u256 {
    12345678
}

/// Provides the default transaction gas price used in the test environment.
public fun default_gas_price(): u256 {
    55555555555555
}

/// Provides the default transaction block number used in the test environment.
public fun default_block_number(): u64 {
    3141592
}

/// Provides the default transaction gas limit used in the test environment.
public fun default_gas_limit(): u64 {
    30_000_000
}

/// Provides the default transaction block timestmap used in the test environment.
public fun default_block_timestamp(): u64 {
    1438338373
}

/// Provides the default transaction chain id used in the test environment.
public fun default_chain_id(): u64 {
    42331
}
