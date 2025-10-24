module stylus::contract_calls;

const ECallFailed: u64 = 101;

public use fun empty_result_succeded as ContractCallEmptyResult.succeded;

public struct CrossContractCall has drop, copy {
    contract_address: address,
    delegate: bool,
    gas: u64,
    value: u256,
}

public fun new(contract_address: address): CrossContractCall {
    CrossContractCall {
        contract_address,
        delegate: false,
        gas: 0xffffffffffffffff,
        value: 0,
    }
}

public fun gas(self: &mut CrossContractCall, gas: u64) {
    self.gas = gas;
}

public fun value(self: &mut CrossContractCall, value: u256) {
    self.value = value;
}

public fun delegate(self: &mut CrossContractCall) {
    self.delegate = true;
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
    assert!(code == 0, ECallFailed);
    result
}

public struct ContractCallEmptyResult has drop {
    code: u8,
}

public fun empty_result_succeded(self: &ContractCallEmptyResult): bool {
    self.code == 0
}
