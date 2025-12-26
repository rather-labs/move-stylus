use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "enums_control_flow",
    "tests/enums/move_sources/enums_control_flow.move"
);

sol! {
    enum Number {
        One,
        Two,
        Three,
        Four,
        Five,
    }

    enum Color {
        R,
        G,
        B,
    }

    enum Boolean {
        True,
        False,
    }

    function simpleMatch(Number n) external returns (uint32);
    function simpleMatchSingleCase(Number n) external returns (uint32);
    function nestedMatch(Number n, Color c, Boolean b) external returns (uint32);
    function matchWithConditional(Number n, bool a, bool b) external returns (uint32);
    function nestedMatchWithConditional(Number n, Color c, bool a, bool b) external returns (uint32);
    function matchWithManyAborts(Number n, Color c) external returns (uint32);
    function matchWithSingleYieldingBranch(Number n, Color c) external returns (uint32);
    function miscControlFlow(Number n, Color c, Boolean b) external returns (uint32, uint32);
    function miscControlFlow2(Number n, Color c, Boolean b) external returns (uint32);
    function miscControlFlow3(Color c) external returns (uint64);
    function miscControlFlow4(Number n, Boolean b) external returns (uint64);
    function miscControlFlow5(Number n) external returns (uint64);
}

#[rstest]
#[case(simpleMatchCall::new((Number::One,)), (1,))]
#[case(simpleMatchCall::new((Number::Two,)), (2,))]
#[case(simpleMatchCall::new((Number::Three,)), (3,))]
#[case(simpleMatchCall::new((Number::Four,)), (4,))]
#[case(simpleMatchCall::new((Number::Five,)), (5,))]
#[case(simpleMatchSingleCaseCall::new((Number::One,)), (42,))]
#[should_panic]
#[case(simpleMatchSingleCaseCall::new((Number::Two,)), (0,))]
#[should_panic]
#[case(simpleMatchSingleCaseCall::new((Number::Three,)), (0,))]
#[should_panic]
#[case(simpleMatchSingleCaseCall::new((Number::Four,)), (0,))]
#[case(nestedMatchCall::new((Number::One, Color::R, Boolean::True)), (1,))]
#[case(nestedMatchCall::new((Number::Two, Color::R, Boolean::True)), (2,))]
#[case(nestedMatchCall::new((Number::Two, Color::G, Boolean::False)), (3,))]
#[case(nestedMatchCall::new((Number::Two, Color::B, Boolean::True)), (4,))]
#[case(nestedMatchCall::new((Number::Three, Color::R, Boolean::True)), (5,))]
#[case(nestedMatchCall::new((Number::Four, Color::B, Boolean::False)), (6,))]
#[case(nestedMatchCall::new((Number::Five, Color::G, Boolean::False)), (6,))]
#[case(matchWithConditionalCall::new((Number::One, true, false)), (1,))]
#[case(matchWithConditionalCall::new((Number::One, false, true)), (6,))]
#[case(matchWithConditionalCall::new((Number::Two, true, true)), (2,))]
#[case(matchWithConditionalCall::new((Number::Two, false, false)), (6,))]
#[case(matchWithConditionalCall::new((Number::Three, true, true)), (2,))]
#[case(matchWithConditionalCall::new((Number::Three, false, false)), (6,))]
#[case(matchWithConditionalCall::new((Number::Four, true, false)), (2,))]
#[case(matchWithConditionalCall::new((Number::Four, false, true)), (4,))]
#[case(matchWithConditionalCall::new((Number::Four, false, false)), (5,))]
#[case(matchWithConditionalCall::new((Number::Five, true, false)), (2,))]
#[case(matchWithConditionalCall::new((Number::Five, false, true)), (3,))]
#[case(matchWithConditionalCall::new((Number::Five, false, false)), (3,))]
#[case(nestedMatchWithConditionalCall::new((Number::One, Color::R, true, false)), (1,))]
#[case(nestedMatchWithConditionalCall::new((Number::Two, Color::R, true, false)), (2,))]
#[case(nestedMatchWithConditionalCall::new((Number::Three, Color::R, true, false)), (2,))]
#[case(nestedMatchWithConditionalCall::new((Number::Four, Color::R, true, false)), (2,))]
#[case(nestedMatchWithConditionalCall::new((Number::Five, Color::R, true, false)), (2,))]
#[case(nestedMatchWithConditionalCall::new((Number::Five, Color::R, false, true)), (3,))]
#[case(nestedMatchWithConditionalCall::new((Number::Four, Color::R, false, true)), (4,))]
#[case(nestedMatchWithConditionalCall::new((Number::Four, Color::R, false, false)), (6,))]
#[case(nestedMatchWithConditionalCall::new((Number::Four, Color::B, false, false)), (7,))]
#[case(nestedMatchWithConditionalCall::new((Number::One, Color::B, false, true)), (8,))]
#[case(nestedMatchWithConditionalCall::new((Number::Two, Color::G, true, false)), (2,))]
#[case(nestedMatchWithConditionalCall::new((Number::Three, Color::G, true, false)), (2,))]
#[case(nestedMatchWithConditionalCall::new((Number::Four, Color::G, false, true)), (5,))]
#[case(nestedMatchWithConditionalCall::new((Number::Five, Color::G, false, true)), (3,))]
#[case(nestedMatchWithConditionalCall::new((Number::One, Color::G, false, false)), (8,))]
#[should_panic]
#[case(matchWithManyAbortsCall::new((Number::One, Color::R)), (1,))]
#[should_panic]
#[case(matchWithManyAbortsCall::new((Number::Two, Color::R)), (2,))]
#[case(matchWithManyAbortsCall::new((Number::Two, Color::G)), (1,))]
#[should_panic]
#[case(matchWithManyAbortsCall::new((Number::Two, Color::B)), (2,))]
#[should_panic]
#[case(matchWithManyAbortsCall::new((Number::Three, Color::R)), (1,))]
#[should_panic]
#[case(matchWithManyAbortsCall::new((Number::Three, Color::B)), (1,))]
#[case(matchWithManyAbortsCall::new((Number::Four, Color::R)), (2,))]
#[should_panic]
#[case(matchWithManyAbortsCall::new((Number::Five, Color::G)), (2,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::One, Color::R)), (42,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::Two, Color::R)), (42,))]
#[case(matchWithSingleYieldingBranchCall::new((Number::Two, Color::G)), (1,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::Two, Color::B)), (42,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::Three, Color::B)), (42,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::Four, Color::R)), (42,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::Four, Color::G)), (42,))]
#[should_panic]
#[case(matchWithSingleYieldingBranchCall::new((Number::Five, Color::G)), (42,))]
#[should_panic]
#[case(miscControlFlowCall::new((Number::One, Color::R, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlowCall::new((Number::Two, Color::R, Boolean::True)), (42,))]
#[case(miscControlFlowCall::new((Number::Two, Color::G, Boolean::False)), (5,))]
#[case(miscControlFlowCall::new((Number::Two, Color::G, Boolean::True)), (4,))]
#[should_panic]
#[case(miscControlFlowCall::new((Number::Two, Color::B, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlowCall::new((Number::Three, Color::B, Boolean::True)), (42,))]
#[case(miscControlFlowCall::new((Number::Four, Color::R, Boolean::False)), (6,))]
#[case(miscControlFlowCall::new((Number::Four, Color::G, Boolean::False)), (6,))]
#[case(miscControlFlowCall::new((Number::Four, Color::G, Boolean::True)), (5,))]
#[should_panic]
#[case(miscControlFlowCall::new((Number::Five, Color::G, Boolean::True)), (42,))]
#[case(miscControlFlow2Call::new((Number::Two, Color::G, Boolean::False)), (3,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::One, Color::R, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Two, Color::R, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Two, Color::G, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Two, Color::B, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Three, Color::B, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Four, Color::R, Boolean::False)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Four, Color::G, Boolean::False)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Five, Color::G, Boolean::True)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Two, Color::R, Boolean::False)), (42,))]
#[should_panic]
#[case(miscControlFlow2Call::new((Number::Two, Color::B, Boolean::False)), (42,))]
#[case(miscControlFlow3Call::new((Color::R,)), (5,))]
#[case(miscControlFlow3Call::new((Color::G,)), (7,))]
#[case(miscControlFlow3Call::new((Color::B,)), (11,))]
#[case(miscControlFlow4Call::new((Number::One, Boolean::True)), (2,))]
#[case(miscControlFlow4Call::new((Number::Two, Boolean::True)), (4,))]
#[case(miscControlFlow4Call::new((Number::Three, Boolean::True)), (30,))]
#[case(miscControlFlow4Call::new((Number::Four, Boolean::True)), (30,))]
#[case(miscControlFlow4Call::new((Number::Five, Boolean::True)), (30,))]
#[case(miscControlFlow4Call::new((Number::One, Boolean::False)), (3,))]
#[case(miscControlFlow4Call::new((Number::Two, Boolean::False)), (6,))]
#[case(miscControlFlow5Call::new((Number::One,)), (1,))]
#[case(miscControlFlow5Call::new((Number::Two,)), (2,))]
#[case(miscControlFlow5Call::new((Number::Three,)), (3,))]
#[case(miscControlFlow5Call::new((Number::Four,)), (4,))]
#[case(miscControlFlow5Call::new((Number::Five,)), (5,))]
fn test_match_with_many_aborts<T: SolCall, V: SolValue>(
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
