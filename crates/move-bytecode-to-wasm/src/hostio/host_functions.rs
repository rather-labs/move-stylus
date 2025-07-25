use walrus::{FunctionId, ImportId, Module, ValType};

pub fn add_pay_for_memory_grow(module: &mut Module) -> (FunctionId, ImportId) {
    let pay_for_memory_grow_type = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "pay_for_memory_grow", pay_for_memory_grow_type)
}

/// Host function to read the arguments to memory
///
/// Reads the program calldata. The semantics are equivalent to that of the EVM's
/// [`CALLDATA_COPY`] opcode when requesting the entirety of the current call's calldata.
///
/// [`CALLDATA_COPY`]: https://www.evm.codes/#37
///
/// Receives a pointer to the memory, and writes the length of the arguments to it
pub fn read_args(module: &mut Module) -> (FunctionId, ImportId) {
    let read_args_type = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "read_args", read_args_type)
}

/// Host function to write the result to memory
///
/// Writes the final return data. If not called before the program exists, the return data will
/// be 0 bytes long. Note that this hostio does not cause the program to exit, which happens
/// naturally when `user_entrypoint` returns.
///
/// Receives a pointer to the memory and the length of the result
pub fn write_result(module: &mut Module) -> (FunctionId, ImportId) {
    let write_result_type = module.types.add(&[ValType::I32, ValType::I32], &[]);
    module.add_import_func("vm_hooks", "write_result", write_result_type)
}

/// Persists any dirty values in the storage cache to the EVM state trie, dropping the cache entirely if requested.
/// Analogous to repeated invocations of [`SSTORE`].
///
/// [`SSTORE`]: https://www.evm.codes/#55
///
/// param: clear: bool -> clear the cache if true
pub fn storage_flush_cache(module: &mut Module) -> (FunctionId, ImportId) {
    let storage_flush_cache_type = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "storage_flush_cache", storage_flush_cache_type)
}

/// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
/// EVM's [`ORIGIN`] opcode.
///
/// [`ORIGIN`]: https://www.evm.codes/#32
pub fn tx_origin(module: &mut Module) -> (FunctionId, ImportId) {
    let tx_origin = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "tx_origin", tx_origin)
}

/// Emits an EVM log with the given number of topics and data, the first bytes of which should
/// be the 32-byte-aligned topic data. The semantics are equivalent to that of the EVM's
/// [`LOG0`], [`LOG1`], [`LOG2`], [`LOG3`], and [`LOG4`] opcodes based on the number of topics
/// specified. Requesting more than `4` topics will induce a revert.
///
/// [`LOG0`]: https://www.evm.codes/#a0
/// [`LOG1`]: https://www.evm.codes/#a1
/// [`LOG2`]: https://www.evm.codes/#a2
/// [`LOG3`]: https://www.evm.codes/#a3
/// [`LOG4`]: https://www.evm.codes/#a4
pub fn emit_log(module: &mut Module) -> (FunctionId, ImportId) {
    let emit_log = module
        .types
        .add(&[ValType::I32, ValType::I32, ValType::I32], &[]);
    module.add_import_func("vm_hooks", "emit_log", emit_log)
}

/// Gets the address of the account that called the program. For normal L2-to-L2 transactions
/// the semantics are equivalent to that of the EVM's [`CALLER`] opcode, including in cases
/// arising from [`DELEGATE_CALL`].
///
/// For L1-to-L2 retryable ticket transactions, the top-level sender's address will be aliased.
/// See [`Retryable Ticket Address Aliasing`] for more information on how this works.
///
/// [`CALLER`]: https://www.evm.codes/#33
/// [`DELEGATE_CALL`]: https://www.evm.codes/#f4
/// [`Retryable Ticket Address Aliasing`]: https://developer.arbitrum.io/arbos/l1-to-l2-messaging#address-aliasing
pub fn msg_sender(module: &mut Module) -> (FunctionId, ImportId) {
    let msg_sender = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "msg_sender", msg_sender)
}

/// Get the ETH value in wei sent to the program. The semantics are equivalent to that of the
/// EVM's [`CALLVALUE`] opcode.
///
/// [`CALLVALUE`]: https://www.evm.codes/#34
pub fn msg_value(module: &mut Module) -> (FunctionId, ImportId) {
    let msg_value_ty = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "msg_value", msg_value_ty)
}

/// Gets a bounded estimate of the L1 block number at which the Sequencer sequenced the
/// transaction. See [`Block Numbers and Time`] for more information on how this value is
/// determined.
///
/// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
pub fn block_number(module: &mut Module) -> (FunctionId, ImportId) {
    let block_number_ty = module.types.add(&[], &[ValType::I64]);
    module.add_import_func("vm_hooks", "block_number", block_number_ty)
}

/// Gets the basefee of the current block. The semantics are equivalent to that of the EVM's
/// [`BASEFEE`] opcode.
///
/// [`BASEFEE`]: https://www.evm.codes/#48
pub fn block_basefee(module: &mut Module) -> (FunctionId, ImportId) {
    let block_basefee_ty = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "block_basefee", block_basefee_ty)
}

/// Gets the gas limit of the current block. The semantics are equivalent to that of the EVM's
/// [`GAS_LIMIT`] opcode. Note that as of the time of this writing, `evm.codes` incorrectly
/// implies that the opcode returns the gas limit of the current transaction.  When in doubt,
/// consult [`The Ethereum Yellow Paper`].
///
/// [`GAS_LIMIT`]: https://www.evm.codes/#45
/// [`The Ethereum Yellow Paper`]: https://ethereum.github.io/yellowpaper/paper.pdf
pub fn block_gas_limit(module: &mut Module) -> (FunctionId, ImportId) {
    let block_gas_limit_ty = module.types.add(&[], &[ValType::I64]);
    module.add_import_func("vm_hooks", "block_gas_limit", block_gas_limit_ty)
}

/// Gets a bounded estimate of the Unix timestamp at which the Sequencer sequenced the
/// transaction. See [`Block Numbers and Time`] for more information on how this value is
/// determined.
///
/// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
pub fn block_timestamp(module: &mut Module) -> (FunctionId, ImportId) {
    let block_timestamp_ty = module.types.add(&[], &[ValType::I64]);
    module.add_import_func("vm_hooks", "block_timestamp", block_timestamp_ty)
}

/// Gets the unique chain identifier of the Arbitrum chain. The semantics are equivalent to
/// that of the EVM's [`CHAIN_ID`] opcode.
///
/// [`CHAIN_ID`]: https://www.evm.codes/#46
pub fn chain_id(module: &mut Module) -> (FunctionId, ImportId) {
    let chain_id_ty = module.types.add(&[], &[ValType::I64]);
    module.add_import_func("vm_hooks", "chainid", chain_id_ty)
}

/// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee. The
/// semantics are equivalent to that of the EVM's [`GAS_PRICE`] opcode.
///
/// [`GAS_PRICE`]: https://www.evm.codes/#3A
pub fn tx_gas_price(module: &mut Module) -> (FunctionId, ImportId) {
    let tx_gas_price_ty = module.types.add(&[ValType::I32], &[]);
    module.add_import_func("vm_hooks", "tx_gas_price", tx_gas_price_ty)
}
