module stylus::contract_calls;

public struct ContractCall<phantom CONCTRACT> has drop {
    contract_address: address,
    delegate: bool
}

public struct ContractCallResult<RESULT> has drop {
    code: u8,
    result: RESULT,
}

public fun succeded<T>(self: &ContractCallResult<T>): bool {
    self.code == 0
}

public fun get_result<T>(self: ContractCallResult<T>): T {
    let ContractCallResult { code, result } = self;
    assert!(code == 0, code as u64);
    result
}

public fun new_contract_call<T>(contract_address: address, delegate: bool): ContractCall<T> {
    ContractCall {
        contract_address,
        delegate,
    }
}
