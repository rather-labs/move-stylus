module stylus::tx_context;

/// Information about the transaction currently being executed.
/// This cannot be constructed by a transaction--it is a privileged object created by
/// the VM and passed in to the entrypoint of the transaction as `&mut TxContext`.
public struct TxContext has drop {}

/// Return the address of the user that signed the current
/// transaction
public fun sender(_self: &TxContext): address {
    native_sender()
}
native fun native_sender(): address;

/// Return the number of wei sent with the message
public fun value(_self: &TxContext): u256 {
    native_msg_value()
}
native fun native_msg_value(): u256;

/// Return the current block's number.
public fun block_number(_self: &TxContext): u64 {
    native_block_number()
}
native fun native_block_number(): u64;

/// Return the current block's base fee (EIP-3198 and EIP-1559)
public fun block_basefee(_self: &TxContext): u256 {
    native_block_basefee()
}
native fun native_block_basefee(): u256;

/// Return the current block's gas limit.
public fun block_gas_limit(_self: &TxContext): u64 {
    native_block_gas_limit()
}
native fun native_block_gas_limit(): u64;

/// Return the current block's timestamp as seconds since unix epoch
public fun block_timestamp(_self: &TxContext): u64 {
    native_block_timestamp()
}
native fun native_block_timestamp(): u64;

/// Return the chain ID of the current transaction.
public fun chain_id(_self: &TxContext): u64 {
    native_chain_id()
}
native fun native_chain_id(): u64;

/// Return the gas price of the transaction
public fun gas_price(_self: &TxContext): u256 {
    native_gas_price()
}
native fun native_gas_price(): u256;

/// Create an `address` that has not been used. As it is an object address, it will never
/// occur as the address for a user.
/// In other words, the generated address is a globally unique object ID.
public fun fresh_object_address(_ctx: &mut TxContext): address {
    fresh_id()
}
native fun fresh_id(): address;

/// Return the calldata of the current transaction as a vector<u8>.
public fun data(_self: &TxContext): vector<u8> {
    native_data()
}
native fun native_data(): vector<u8>;
