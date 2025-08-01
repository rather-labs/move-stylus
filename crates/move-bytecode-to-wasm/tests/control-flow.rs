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

mod control_flow_u8 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "control_flow_u8";
        const SOURCE_PATH: &str = "tests/control-flow/control_flow_u8.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function simpleLoop(uint8 x) external returns (uint8);
        function misc1(uint8 x) external returns (uint8);
        function nestedLoop(uint8 x) external returns (uint8);
        function loopWithBreak(uint8 x) external returns (uint8);
        function conditionalReturn(uint8 x) external returns (uint8);
        function testMatch(uint8 x) external returns (uint8);
        function crazyLoop(uint8 i) external returns (uint8);
        function testMatchInLoop() external returns (uint8);
        function testLabeledLoops(uint8 x) external returns (uint8);
        function checkEven(uint8 x) external returns (uint8);
        function checkEvenAfterLoop(uint8 x) external returns (uint8);
    );

    #[rstest]
    #[case(simpleLoopCall::new((55u8,)), 55u8)]
    #[case(simpleLoopCall::new((1u8,)), 1u8)]
    #[case(misc1Call::new((100u8,)), 55u8)]
    #[case(misc1Call::new((1u8,)), 42u8)]
    #[case(nestedLoopCall::new((5u8,)), 20u8)]
    #[case(loopWithBreakCall::new((5u8,)), 21u8)]
    #[case(loopWithBreakCall::new((10u8,)), 66u8)]
    #[should_panic]
    #[case(conditionalReturnCall::new((5u8,)), 0u8)]
    #[should_panic]
    #[case(conditionalReturnCall::new((68u8,)), 255u8)]
    #[case(conditionalReturnCall::new((17u8,)), 217u8)]
    #[case(conditionalReturnCall::new((20u8,)), 0u8)]
    #[case(conditionalReturnCall::new((26u8,)), 6u8)]
    #[case(conditionalReturnCall::new((101u8,)), 255u8)]
    #[case(conditionalReturnCall::new((255u8,)), 255u8)]
    #[case(testMatchCall::new((1u8,)), 44u8)]
    #[case(testMatchCall::new((2u8,)), 55u8)]
    #[case(testMatchCall::new((3u8,)), 66u8)]
    #[case(testMatchCall::new((4u8,)), 0u8)]
    #[case(crazyLoopCall::new((1u8,)), 65u8)]
    #[case(crazyLoopCall::new((2u8,)), 63u8)]
    #[case(crazyLoopCall::new((4u8,)), 56u8)]
    #[case(testMatchInLoopCall::new(()), 3u8)]
    #[case(testLabeledLoopsCall::new((1u8,)), 25u8)]
    #[case(testLabeledLoopsCall::new((20u8,)), 21u8)]
    #[case(testLabeledLoopsCall::new((10u8,)), 34u8)]
    #[case(checkEvenAfterLoopCall::new((10u8,)), 42u8)]
    #[case(checkEvenAfterLoopCall::new((15u8,)), 55u8)]
    #[case(checkEvenCall::new((10u8,)), 42u8)]
    #[case(checkEvenCall::new((15u8,)), 55u8)]
    fn test_control_flow_u8<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((uint8,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}

mod control_flow_u64 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "control_flow_u64";
        const SOURCE_PATH: &str = "tests/control-flow/control_flow_u64.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function collatz(uint64 x) external returns (uint64);
        function fibonacci(uint64 n) external returns (uint64);
        function isPrime(uint64 i) external returns (bool);
        function sumSpecial(uint64 n) external returns (uint64);
    );

    #[rstest]
    #[case(collatzCall::new((1u64,)), 0u64)]
    #[case(collatzCall::new((2u64,)), 1u64)]
    #[case(collatzCall::new((3u64,)), 7u64)]
    #[case(collatzCall::new((4u64,)), 2u64)]
    #[case(collatzCall::new((5u64,)), 5u64)]
    #[case(collatzCall::new((6u64,)), 8u64)]
    #[case(collatzCall::new((7u64,)), 16u64)]
    #[case(collatzCall::new((8u64,)), 3u64)]
    #[case(collatzCall::new((9u64,)), 19u64)]
    #[case(collatzCall::new((10u64,)), 6u64)]
    #[case(fibonacciCall::new((0u64,)), 0u64)]
    #[case(fibonacciCall::new((1u64,)), 1u64)]
    #[case(fibonacciCall::new((2u64,)), 1u64)]
    #[case(fibonacciCall::new((3u64,)), 2u64)]
    #[case(fibonacciCall::new((4u64,)), 3u64)]
    #[case(fibonacciCall::new((5u64,)), 5u64)]
    #[case(isPrimeCall::new((1u64,)), 0)]
    #[case(isPrimeCall::new((2u64,)), 1)]
    #[case(isPrimeCall::new((3u64,)), 1)]
    #[case(isPrimeCall::new((4u64,)), 0)]
    #[case(isPrimeCall::new((5u64,)), 1)]
    #[case(isPrimeCall::new((7u64,)), 1)]
    #[case(isPrimeCall::new((13u64,)), 1)]
    #[case(isPrimeCall::new((53u64,)), 1)]
    #[case(isPrimeCall::new((54u64,)), 0)]
    #[case(sumSpecialCall::new((0u64,)), 0u64)]
    #[case(sumSpecialCall::new((1u64,)), 0u64)]
    #[case(sumSpecialCall::new((2u64,)), 7u64)]
    #[case(sumSpecialCall::new((3u64,)), 14u64)]
    #[case(sumSpecialCall::new((4u64,)), 14u64)]
    fn test_control_flow_u64<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((uint64,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}
