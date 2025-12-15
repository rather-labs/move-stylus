module stylus::account;

public fun get_account_code_size(account_address: address): u32 {
    account_code_size(account_address)
}

public fun get_account_balance(account_address: address): u256 {
    account_balance(account_address)
}

/// Gets the size of the code in bytes at the given address.
/// The semantics are equivalent to that of the EVM's [`EXT_CODESIZE`].
///
/// [`EXT_CODESIZE`]: https://www.evm.codes/#3B
native fun account_code_size(account_address: address): u32;

/// Gets the ETH balance in wei of the account at the given address.
/// The semantics are equivalent to that of the EVM's [`BALANCE`] opcode.
///
/// [`BALANCE`]: https://www.evm.codes/#31
native fun account_balance(account_address: address): u256;