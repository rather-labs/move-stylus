use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "enum_abi_packing_unpacking",
    "tests/enums/move_sources/enum_abi_packing_unpacking.move"
);

sol! {
    enum SimpleEnum {
        One,
        Two,
        Three,
    }

    function pack1() external returns (SimpleEnum);
    function pack2() external returns (SimpleEnum);
    function pack3() external returns (SimpleEnum);
    function packUnpack(SimpleEnum s) external returns (SimpleEnum);
}

#[rstest]
#[case(pack1Call::new(()), (SimpleEnum::One,))]
#[case(pack2Call::new(()), (SimpleEnum::Two,))]
#[case(pack3Call::new(()), (SimpleEnum::Three,))]
#[case(packUnpackCall::new((SimpleEnum::One,)), (SimpleEnum::One,))]
#[case(packUnpackCall::new((SimpleEnum::Two,)), (SimpleEnum::Two,))]
#[case(packUnpackCall::new((SimpleEnum::Three,)), (SimpleEnum::Three,))]
fn test_enum_abi_packing_unpacking<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}

#[test]
fn test_enum_abi_unpacking_out_of_bounds() {
    // Calldata with non-extistant enum member 4
    let call_data = [packUnpackCall::SELECTOR.to_vec(), (4,).abi_encode()].concat();
    let runtime = runtime();

    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(result, 1);

    let expected = RuntimeError::OutOfBounds.encode_abi();
    assert_eq!(result_data, expected);
}
