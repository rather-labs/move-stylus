use alloy_sol_types::SolValue;
use alloy_sol_types::abi::TokenSeq;
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

mod enum_abi_packing_unpacking {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "enum_abi_packing_unpacking";
        const SOURCE_PATH: &str = "tests/enums/enum_abi_packing_unpacking.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

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
            expected_result.abi_encode_params(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    fn test_enum_abi_unpacking_out_of_bounds() {
        // Calldata with non-extistant enum member 4
        let call_data = [packUnpackCall::SELECTOR.to_vec(), (4,).abi_encode_params()].concat();
        let runtime = runtime();
        runtime.call_entrypoint(call_data).unwrap();
    }
}
