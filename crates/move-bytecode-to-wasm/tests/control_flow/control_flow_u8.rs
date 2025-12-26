use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "control_flow_u8",
    "tests/control_flow/move_sources/control_flow_u8.move"
);

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
    function misc2(bool c1, bool c2) external returns (uint8);
    function misc3(bool c1, bool c2) external returns (uint8);
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
#[case(misc2Call::new((true, true)), 1u8)]
#[case(misc2Call::new((true, false)), 0u8)]
#[case(misc2Call::new((false, true)), 0u8)]
#[case(misc2Call::new((false, false)), 0u8)]
#[case(misc3Call::new((true, true)), 1u8)]
#[case(misc3Call::new((true, false)), 0u8)]
#[case(misc3Call::new((false, true)), 0u8)]
#[case(misc3Call::new((false, false)), 0u8)]
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
