use walrus::{FunctionId, ImportId, Module, ValType};

pub fn add_pay_for_memory_grow(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "pay_for_memory_grow", &[ValType::I32], &[])
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
    get_or_insert_import(module, "read_args", &[ValType::I32], &[])
}

/// Host function to write the result to memory
///
/// Writes the final return data. If not called before the program exists, the return data will
/// be 0 bytes long. Note that this hostio does not cause the program to exit, which happens
/// naturally when `user_entrypoint` returns.
///
/// Receives a pointer to the memory and the length of the result
pub fn write_result(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "write_result", &[ValType::I32, ValType::I32], &[])
}

/// Reads a 32-byte value from permanent storage. Stylus's storage format is identical to
/// that of the EVM. This means that, under the hood, this hostio is accessing the 32-byte
/// value stored in the EVM state trie at offset `key`, which will be `0` when not previously
/// set. The semantics, then, are equivalent to that of the EVM's [`SLOAD`] opcode.
///
/// Note: the Stylus VM implements storage caching. This means that repeated calls to the same key
/// will cost less than in the EVM.
/// params: key: *const u8, dest: *mut u8
pub fn storage_load_bytes32(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        "storage_load_bytes32",
        &[ValType::I32, ValType::I32],
        &[],
    )
}

/// Writes a 32-byte value to the permanent storage cache. Stylus's storage format is identical to that
/// of the EVM. This means that, under the hood, this hostio represents storing a 32-byte value into
/// the EVM state trie at offset `key`. Refunds are tabulated exactly as in the EVM. The semantics, then,
/// are equivalent to that of the EVM's [`SSTORE`] opcode.
///
/// Note: because the value is cached, one must call `storage_flush_cache` to persist it.
/// params: key: *const u8, value: *const u8
pub fn storage_cache_bytes32(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        "storage_cache_bytes32",
        &[ValType::I32, ValType::I32],
        &[],
    )
}

/// Persists any dirty values in the storage cache to the EVM state trie, dropping the cache entirely if requested.
/// Analogous to repeated invocations of [`SSTORE`].
///
/// [`SSTORE`]: https://www.evm.codes/#55
///
/// param: clear: bool -> clear the cache if true
pub fn storage_flush_cache(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "storage_flush_cache", &[ValType::I32], &[])
}

/// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
/// EVM's [`ORIGIN`] opcode.
///
/// [`ORIGIN`]: https://www.evm.codes/#32
pub fn tx_origin(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "tx_origin", &[ValType::I32], &[])
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
    get_or_insert_import(
        module,
        "emit_log",
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    )
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
    get_or_insert_import(module, "msg_sender", &[ValType::I32], &[])
}

/// Get the ETH value in wei sent to the program. The semantics are equivalent to that of the
/// EVM's [`CALLVALUE`] opcode.
///
/// [`CALLVALUE`]: https://www.evm.codes/#34
pub fn msg_value(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "msg_value", &[ValType::I32], &[])
}

/// Gets a bounded estimate of the L1 block number at which the Sequencer sequenced the
/// transaction. See [`Block Numbers and Time`] for more information on how this value is
/// determined.
///
/// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
pub fn block_number(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "block_number", &[], &[ValType::I64])
}

/// Gets the basefee of the current block. The semantics are equivalent to that of the EVM's
/// [`BASEFEE`] opcode.
///
/// [`BASEFEE`]: https://www.evm.codes/#48
pub fn block_basefee(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "block_basefee", &[ValType::I32], &[])
}

/// Gets the gas limit of the current block. The semantics are equivalent to that of the EVM's
/// [`GAS_LIMIT`] opcode. Note that as of the time of this writing, `evm.codes` incorrectly
/// implies that the opcode returns the gas limit of the current transaction.  When in doubt,
/// consult [`The Ethereum Yellow Paper`].
///
/// [`GAS_LIMIT`]: https://www.evm.codes/#45
/// [`The Ethereum Yellow Paper`]: https://ethereum.github.io/yellowpaper/paper.pdf
pub fn block_gas_limit(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "block_gas_limit", &[], &[ValType::I64])
}

/// Gets a bounded estimate of the Unix timestamp at which the Sequencer sequenced the
/// transaction. See [`Block Numbers and Time`] for more information on how this value is
/// determined.
///
/// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
pub fn block_timestamp(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "block_timestamp", &[], &[ValType::I64])
}

/// Gets the unique chain identifier of the Arbitrum chain. The semantics are equivalent to
/// that of the EVM's [`CHAIN_ID`] opcode.
///
/// [`CHAIN_ID`]: https://www.evm.codes/#46
pub fn chain_id(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "chainid", &[], &[ValType::I64])
}

/// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee. The
/// semantics are equivalent to that of the EVM's [`GAS_PRICE`] opcode.
///
/// [`GAS_PRICE`]: https://www.evm.codes/#3A
pub fn tx_gas_price(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(module, "tx_gas_price", &[ValType::I32], &[])
}

/// Efficiently computes the [`keccak256`] hash of the given preimage.
/// The semantics are equivalent to that of the EVM's [`SHA3`] opcode.
///
/// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
/// [`SHA3`]: https://www.evm.codes/#20
#[allow(unused)]
pub fn native_keccak256(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        "native_keccak256",
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    )
}

/// Calls the contract at the given address with options for passing value and to limit the
/// amount of gas supplied. The return status indicates whether the call succeeded, and is
/// nonzero on failure.
///
/// In both cases `return_data_len` will store the length of the result, the bytes of which can
/// be read via the `read_return_data` hostio. The bytes are not returned directly so that the
/// programmer can potentially save gas by choosing which subset of the return result they'd
/// like to copy.
///
/// The semantics are equivalent to that of the EVM's [`CALL`] opcode, including callvalue
/// stipends and the 63/64 gas rule. This means that supplying the `u64::MAX` gas can be used
/// to send as much as possible.
///
/// [`CALL`]: https://www.evm.codes/#f1
pub fn call_contract(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        "call_contract",
        &[
            // Contract address
            ValType::I32,
            // Calldata
            ValType::I32,
            // Calldata len
            ValType::I32,
            // Value
            ValType::I32,
            // Gas
            ValType::I64,
            // Return data len
            ValType::I32,
        ],
        // Result. Non-zero on failure
        &[ValType::I32],
    )
}

/// Static calls the contract at the given address, with the option to limit the amount of gas
/// supplied. The return status indicates whether the call succeeded, and is nonzero on
/// failure.
///
/// In both cases `return_data_len` will store the length of the result, the bytes of which can
/// be read via the `read_return_data` hostio. The bytes are not returned directly so that the
/// programmer can potentially save gas by choosing which subset of the return result they'd
/// like to copy.
///
/// The semantics are equivalent to that of the EVM's [`STATIC_CALL`] opcode, including the
/// 63/64 gas rule. This means that supplying `u64::MAX` gas can be used to send as much as
/// possible.
///
/// [`STATIC_CALL`]: https://www.evm.codes/#FA
pub fn static_call_contract(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        "static_call_contract",
        &[
            // Contract address
            ValType::I32,
            // Calldata
            ValType::I32,
            // Calldata len
            ValType::I32,
            // Gas
            ValType::I64,
            // Return data len
            ValType::I32,
        ],
        // Result. Non-zero on failure
        &[ValType::I32],
    )
}

/// Copies the bytes of the last EVM call or deployment return result. Does not revert if out of
/// bounds, but rather copies the overlapping portion. The semantics are otherwise equivalent
/// to that of the EVM's [`RETURN_DATA_COPY`] opcode.
///
/// Returns the number of bytes written.
///
/// [`RETURN_DATA_COPY`]: https://www.evm.codes/#3e
pub fn read_return_data(module: &mut Module) -> (FunctionId, ImportId) {
    get_or_insert_import(
        module,
        "read_return_data",
        &[
            // Dest
            ValType::I32,
            // Offset
            ValType::I32,
            // Size
            ValType::I32,
        ],
        // Number of bytes written
        &[ValType::I32],
    )
}

#[inline]
fn get_or_insert_import(
    module: &mut walrus::Module,
    name: &str,
    params: &[walrus::ValType],
    results: &[walrus::ValType],
) -> (walrus::FunctionId, walrus::ImportId) {
    if let Ok(function_id) = module.imports.get_func("vm_hooks", name) {
        for import in module.imports.iter() {
            if let walrus::ImportKind::Function(func_id) = import.kind {
                if func_id == function_id {
                    return (function_id, import.id());
                }
            }
        }
    }

    let ty = module.types.add(params, results);
    module.add_import_func("vm_hooks", name, ty)
}
