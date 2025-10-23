module stylus::contract_calls;

const ECallFailed: u64 = 101;

public use fun empty_result_succeded as ContractCallEmptyResult.succeded;

public struct ContractCallResult<RESULT> has drop {
    code: u8,
    result: RESULT,
}

public fun succeded<T>(self: &ContractCallResult<T>): bool {
    self.code == 0
}

public fun get_result<T>(self: ContractCallResult<T>): T {
    let ContractCallResult { code, result } = self;
    assert!(code == 0, ECallFailed);
    result
}

public struct ContractCallEmptyResult has drop {
    code: u8,
}

public fun empty_result_succeded<T>(self: &ContractCallEmptyResult): bool {
    self.code == 0
}
