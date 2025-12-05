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

/// Provides the default transaction block timestmap used in the test environment.
public fun default_chain_id(): u64 {
    42331
}


//
// Unit tests
//
// These tests, besides testin this module, tests the test runner
//
#[test]
fun test_default_signer() {
    let addr = default_signer();
    assert!(addr == @0xbeef);
}

#[test]
fun test_default_sender() {
    let addr = default_sender();
    assert!(addr == @0xcafe);
}

#[test]
fun test_default_base_fee() {
    let fee = default_base_fee();
    assert!(fee == 12345678);
}

#[test]
fun test_default_gas_price() {
    let price = default_gas_price();
    assert!(price == 55555555555555);
}

#[test]
fun test_default_block_number() {
    let num = default_block_number();
    assert!(num == 3141592);
}

#[test]
fun test_default_gas_limit() {
    let limit = default_gas_limit();
    assert!(limit == 30_000_000);
}

#[test]
fun test_default_block_timestamp() {
    let ts = default_block_timestamp();
    assert!(ts == 1438338373);
}

#[test]
fun test_default_chain_id() {
    let id = default_chain_id();
    assert!(id == 42331);
}

#[test]
fun test_basefee_changed() {
    let ctx = new_tx_context();
    let new_fee: u256 = 999999;

    set_block_basefee(new_fee);

    assert!(default_base_fee() != ctx.block_basefee());
    assert!(new_fee == ctx.block_basefee());
}

#[test]
fun test_sender_changed() {
    let ctx = new_tx_context();
    let new_addr = @0x1111;

    set_sender_address(new_addr);

    assert!(default_signer() != ctx.sender());
    assert!(new_addr == ctx.sender());
}

#[test]
fun test_gas_price_changed() {
    let ctx = new_tx_context();
    let new_price: u256 = 123456789;

    set_gas_price(new_price);

    assert!(default_gas_price() != ctx.gas_price());
    assert!(new_price == ctx.gas_price());
}

#[test]
fun test_block_number_changed() {
    let ctx = new_tx_context();
    let new_number: u64 = 42;

    set_block_number(new_number);

    assert!(default_block_number() != ctx.block_number());
    assert!(new_number == ctx.block_number());
}

#[test]
fun test_gas_limit_changed() {
    let ctx = new_tx_context();
    let new_limit: u64 = 1000000;

    set_gas_limit(new_limit);

    assert!(default_gas_limit() != ctx.block_gas_limit());
    assert!(new_limit == ctx.block_gas_limit());
}

#[test]
fun test_block_timestamp_changed() {
    let ctx = new_tx_context();
    let new_ts: u64 = 987654321;

    set_block_timestamp(new_ts);

    assert!(default_block_timestamp() != ctx.block_timestamp());
    assert!(new_ts == ctx.block_timestamp());
}

#[test]
fun test_chain_id_changed() {
    let ctx = new_tx_context();
    let new_id: u64 = 99;

    set_chain_id(new_id);

    assert!(default_chain_id() != ctx.chain_id());
    assert!(new_id == ctx.chain_id());
}

#[test, expected_failure]
fun test_chain_id_expect_failure() {
    let ctx = new_tx_context();
    let new_id: u64 = 99;

    set_chain_id(new_id);

    assert!(default_chain_id() == ctx.chain_id());
}

#[test, skip]
fun test_skip() {
    assert!(true);
}
