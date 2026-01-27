# Account

The `stylus::account` module allows you to inspect account data, such as the amount of ETH held or whether an address contains smart contract code. This functions are direct wrappers to the account [stylus host functions](https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/stylus-sdk/src/hostio.rs).

```move
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
```