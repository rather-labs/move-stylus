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

mod control_flow {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "control_flow";
        const SOURCE_PATH: &str = "tests/control-flow/control_flow.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function simpleLoop(uint8 x) external returns (uint8);
        function misc1(uint8 x) external returns (uint8);
        function nestedLoop(uint8 x) external returns (uint8);
        function loopWithBreak(uint8 x) external returns (uint8);
        function earlyReturn(uint8 x) external returns (uint8);
        function testMatch(uint8 x) external returns (uint8);
        function crazyLoop(uint8 i) external returns (uint8);
        function testMatchInLoop() external returns (uint8);
    );

    #[rstest]
    #[case(simpleLoopCall::new((55u8,)), 55u8)]
    #[case(simpleLoopCall::new((1u8,)), 1u8)]
    #[case(misc1Call::new((100u8,)), 55u8)]
    #[case(misc1Call::new((1u8,)), 42u8)]
    #[case(nestedLoopCall::new((5u8,)), 20u8)]
    #[case(loopWithBreakCall::new((5u8,)), 21u8)]
    #[case(loopWithBreakCall::new((10u8,)), 66u8)]
    #[case(earlyReturnCall::new((5u8,)), 6u8)]
    #[case(earlyReturnCall::new((150u8,)), 255u8)]
    #[case(testMatchCall::new((1u8,)), 44u8)]
    #[case(testMatchCall::new((2u8,)), 55u8)]
    #[case(testMatchCall::new((3u8,)), 66u8)]
    #[case(testMatchCall::new((4u8,)), 0u8)]
    #[case(crazyLoopCall::new((1u8,)), 65u8)]
    #[case(crazyLoopCall::new((2u8,)), 63u8)]
    #[case(crazyLoopCall::new((4u8,)), 56u8)]
    #[case(testMatchInLoopCall::new(()), 38u8)]
    fn test_control_flow<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((uint8,))>::abi_encode_params(&(expected_result,)),
        )
        .unwrap();
    }
}
