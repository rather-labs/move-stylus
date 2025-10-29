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
            expected_result.abi_encode(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    fn test_enum_abi_unpacking_out_of_bounds() {
        // Calldata with non-extistant enum member 4
        let call_data = [packUnpackCall::SELECTOR.to_vec(), (4,).abi_encode()].concat();
        let runtime = runtime();
        runtime.call_entrypoint(call_data).unwrap();
    }
}

mod enum_with_fields {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "enum_with_fields";
        const SOURCE_PATH: &str = "tests/enums/enum_with_fields.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    use alloy_primitives::U256;
    use alloy_sol_types::SolValue; // for .abi_encode()
    use alloy_sol_types::sol; // runtime bytes

    sol! {
        function packUnpackPlanet(uint8 index) external returns (uint64, uint64);
        function packUnpackStackInts(uint8 x, uint16 y, uint32 z, uint64 w) external returns (uint8, uint16, uint32, uint64);
        function packUnpackHeapInts(uint128 x, uint256 y) external returns (uint128, uint256);
        function packUnpackPositionalVector(uint8 a, uint16 b, uint32 c, uint64 d) external returns (uint8[], uint16[], uint32[], uint64[]);
        function packUnpackNamedVectors(uint128 x, uint256 y) external returns (uint128[], uint256[]);
        function packUnpackPositionalNestedVectors(uint32 x, uint64 y) external returns (uint32[][], uint64[][]);
        function packUnpackAlpha(uint8 a, uint16 b, uint32 c, uint64 d) external returns (uint8, uint16, uint32, uint64);
        function packUnpackBeta(uint128 e, uint256 f) external returns (uint128, uint256);
        function packUnpackGamma(uint32[] a, bool[] b, uint128 c, uint256 d) external returns (uint32[], bool[], uint128, uint256);
        function getGammaVecSum(uint32[] a, bool[] b, uint128 c, uint256 d) external returns (uint32);
    }

    #[rstest]
    #[case(packUnpackPlanetCall::new((0,)), (6371, 5972))]
    #[case(packUnpackPlanetCall::new((1,)), (69911, 1898000))]
    #[case(packUnpackPlanetCall::new((2,)), (3389, 641))]
    #[case(packUnpackPlanetCall::new((3,)), (6051, 4868))]
    #[case(packUnpackPlanetCall::new((4,)), (58232, 56800))]
    fn test_pack_unpack_planet<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(packUnpackStackIntsCall::new((0, 0u16, 0u32, 0u64)), (0, 0u16, 0u32, 0u64))]
    #[case(packUnpackStackIntsCall::new((1, 2u16, 3u32, 4u64)), (1, 2u16, 3u32, 4u64))]
    #[case(packUnpackStackIntsCall::new((255, u16::MAX, u32::MAX, u64::MAX)), (255, u16::MAX, u32::MAX, u64::MAX))]
    #[case(packUnpackHeapIntsCall::new((0u128, U256::from(0u128))), (0u128, U256::from(0u128)))]
    #[case(packUnpackHeapIntsCall::new((1u128, U256::from(2u128))), (1u128, U256::from(2u128)))]
    #[case(packUnpackHeapIntsCall::new((u128::MAX, U256::from(u128::MAX))), (u128::MAX, U256::from(u128::MAX)))]
    fn test_pack_unpack_ints<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(
        packUnpackPositionalVectorCall::new((87u8, 42u16, 55u32, 71u64)),
        (
            vec![87, 88, 89],
            vec![42u16, 43u16, 44u16],
            vec![55u32, 56u32, 57u32],
            vec![71u64, 72u64, 73u64],
        )
    )]
    #[case(packUnpackNamedVectorsCall::new((0u128, U256::from(0u128))), (vec![0u128, 1u128, 2u128], vec![U256::from(0u128), U256::from(1u128), U256::from(2u128)]))]
    #[case(packUnpackPositionalNestedVectorsCall::new((0u32, 0u64)), (vec![vec![0u32, 1u32, 2u32], vec![3u32, 4u32, 5u32]], vec![vec![0u64, 1u64, 2u64], vec![3u64, 4u64, 5u64]]))]
    fn test_pack_unpack_enums_with_vectores<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode_sequence(),
        )
        .unwrap();
    }

    #[rstest]
    #[case(packUnpackAlphaCall::new((0, 0u16, 0u32, 0u64)), (0, 0u16, 0u32, 0u64))]
    #[case(packUnpackAlphaCall::new((1, 2u16, 3u32, 4u64)), (1, 2u16, 3u32, 4u64))]
    #[case(packUnpackAlphaCall::new((255, u16::MAX, u32::MAX, u64::MAX)), (255, u16::MAX, u32::MAX, u64::MAX))]
    #[case(packUnpackBetaCall::new((0u128, U256::from(0u128))), (0u128, U256::from(0u128)))]
    #[case(packUnpackBetaCall::new((1u128, U256::from(2u128))), (1u128, U256::from(2u128)))]
    #[case(packUnpackBetaCall::new((u128::MAX, U256::from(u128::MAX))), (u128::MAX, U256::from(u128::MAX)))]
    #[case(packUnpackGammaCall::new((vec![0, 1, 2], vec![false, true, false], 33u128, U256::from(42))), (vec![0, 1, 2], vec![false, true, false], 33u128, U256::from(42)))]
    #[case(packUnpackGammaCall::new((vec![42u32, 43u32, 44u32], vec![true, false, true], 123u128, U256::from(321))), (vec![42u32, 43u32, 44u32], vec![true, false, true], 123u128, U256::from(321)))]
    #[case(getGammaVecSumCall::new((vec![42u32, 43u32, 44u32], vec![true, false, true], 123u128, U256::from(321))), (129u32,))]
    fn test_pack_unpack_struct_enums<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode_sequence(),
        )
        .unwrap();
    }
}

mod control_flow {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "enums_control_flow";
        const SOURCE_PATH: &str = "tests/enums/enums_control_flow.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

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
}

mod geometry {
    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::{SolCall, SolValue}; // for .abi_encode() // runtime bytes

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "enums_geometry";
        const SOURCE_PATH: &str = "tests/enums/geometry.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        function testSquare(uint64 side) external returns (uint64, uint64);
        function testMutateSquare(uint64 side) external returns (uint64, uint64, uint64, uint64);
        function testTriangle(uint64 base, uint64 height) external returns (uint64, uint64, uint64);
        function testMutateTriangle(uint64 base, uint64 height) external returns (uint64, uint64, uint64, uint64, uint64, uint64);
        function testVectorOfShapes1(uint64 a, uint64 b) external returns (uint64, uint64, uint64);
        function testVectorOfShapes2(uint64 a, uint64 b) external returns (uint64, uint64, uint64);
    }

    #[rstest]
    #[case(testSquareCall::new((4u64,)), (4u64, 16u64))]
    #[case(testSquareCall::new((5u64,)), (5u64, 25u64))]
    #[case(testMutateSquareCall::new((4u64,)), (4u64, 16u64, 5u64, 25u64))]
    #[case(testMutateSquareCall::new((5u64,)), (5u64, 25u64, 6u64, 36u64))]
    #[case(testTriangleCall::new((4u64, 5u64)), (4u64, 5u64, 10u64))]
    #[case(testTriangleCall::new((5u64, 6u64)), (5u64, 6u64, 15u64))]
    #[case(testMutateTriangleCall::new((4u64, 5u64)), (4u64, 5u64, 10u64, 5u64, 6u64, 15u64))]
    #[case(testMutateTriangleCall::new((5u64, 6u64)), (5u64, 6u64, 15u64, 6u64, 7u64, 21u64))]
    #[case(testVectorOfShapes1Call::new((2u64, 3u64)), (2u64, 4u64, 3u64))]
    #[case(testVectorOfShapes2Call::new((2u64, 3u64)), (2u64, 3u64, 4u64))]
    fn test_geometry<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode_sequence(),
        )
        .unwrap();
    }
}

mod magical_creatures {
    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::{SolCall, SolValue};

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "enums_magical_creatures";
        const SOURCE_PATH: &str = "tests/enums/magical_creatures.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        function testBeast(uint32 level, uint64 ferocity) external returns (uint32, uint64, uint32, uint64);
        function testGolem(uint32 level, uint128 density, uint64[] shards) external returns (uint32, uint64, uint32, uint64);
        function testSpirit(uint32 level, uint8[][] chants, uint64 age) external returns (uint32, uint64, uint32, uint64);
    }

    #[rstest]
    #[case(testBeastCall::new((1u32, 2u64)), (1u32, 2u64, 2u32, 4u64))]
    #[case(testBeastCall::new((3u32, 5u64)), (3u32, 15u64, 4u32, 20u64))]
    #[case(testGolemCall::new((1u32, 10u128, vec![5u64, 7u64])), (1u32, 23u64, 2u32, 39u64))]
    #[case(testGolemCall::new((2u32, 20u128, vec![3u64, 4u64, 6u64])), (2u32, 35u64, 3u32, 51u64))]
    #[case(testSpiritCall::new((1u32, vec![vec![2u8, 3u8, 4u8]], 4u64)), (1u32, 8u64, 2u32, 15u64))]
    #[case(testSpiritCall::new((2u32, vec![vec![1u8, 2u8], vec![3u8, 4u8, 5u8]], 6u64)), (2u32, 13u64, 3u32, 20u64))]
    fn test_magical_creatures<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode_sequence(),
        )
        .unwrap();
    }
}

mod stars {
    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::{SolCall, SolValue};

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "stars";
        const SOURCE_PATH: &str = "tests/enums/stars.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        enum Core {
            Hydrogen,
            Helium,
            Carbon,
            Nitrogen,
            Oxygen,
        }

        enum StarType {
            RedDwarf,
            YellowDwarf,
            RedGiant,
            BlueGiant,
        }

        struct Star {
            string name;
            StarType class;
            Core core;
            uint32 size;
        }

        function createStar(string name, StarType class, Core core, uint32 size) external returns (Star);
        function evolveStar(Star star) external returns (Star);
        function getCoreProperties(Star star) external returns (uint8, uint8);
        function getMilkyWayMass() external returns (uint64);
    }

    #[rstest]
    #[case(createStarCall::new((String::from("Sun"), StarType::YellowDwarf, Core::Hydrogen, 55)), Star { name: String::from("Sun"), class: StarType::YellowDwarf, core: Core::Hydrogen, size: 55 })]
    #[case(createStarCall::new((String::from("Proxima Centauri"), StarType::RedDwarf, Core::Helium, 1)), Star { name: String::from("Proxima Centauri"), class: StarType::RedDwarf, core: Core::Helium, size: 1 })]
    #[case(createStarCall::new((String::from("Betelgeuse"), StarType::RedGiant, Core::Carbon, 764)), Star { name: String::from("Betelgeuse"), class: StarType::RedGiant, core: Core::Carbon, size: 764 })]
    #[case(createStarCall::new((String::from("Vega"), StarType::BlueGiant, Core::Nitrogen, 2)), Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 2 })]
    #[case(createStarCall::new((String::from("Polaris"), StarType::YellowDwarf, Core::Oxygen, 37)), Star { name: String::from("Polaris"), class: StarType::YellowDwarf, core: Core::Oxygen, size: 37 })]
    #[case(evolveStarCall::new((Star { name: String::from("Sun"), class: StarType::YellowDwarf, core: Core::Hydrogen, size: 55 },)), Star { name: String::from("Sun"), class: StarType::RedGiant, core: Core::Helium, size: 5500 })]
    #[case(evolveStarCall::new((Star { name: String::from("Proxima Centauri"), class: StarType::RedDwarf, core: Core::Helium, size: 1 },)), Star { name: String::from("Proxima Centauri"), class: StarType::RedGiant, core: Core::Carbon, size: 2 })]
    #[case(evolveStarCall::new((Star { name: String::from("Betelgeuse"), class: StarType::RedGiant, core: Core::Carbon, size: 764 },)), Star { name: String::from("Betelgeuse"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 3820 })]
    #[case(evolveStarCall::new((Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 2 },)), Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Oxygen, size: 6 })]
    #[should_panic]
    #[case(evolveStarCall::new((Star { name: String::from("Polaris"), class: StarType::YellowDwarf, core: Core::Oxygen, size: 37 },)), Star { name: String::from("Polaris"), class: StarType::BlueGiant, core: Core::Oxygen, size: 111 })]
    #[case(getCorePropertiesCall::new((Star { name: String::from("Sun"), class: StarType::YellowDwarf, core: Core::Hydrogen, size: 55 },)), (1, 1))]
    #[case(getCorePropertiesCall::new((Star { name: String::from("Proxima Centauri"), class: StarType::RedDwarf, core: Core::Helium, size: 1 },)), (2, 18))]
    #[case(getCorePropertiesCall::new((Star { name: String::from("Betelgeuse"), class: StarType::RedGiant, core: Core::Carbon, size: 764 },)), (6, 14))]
    #[case(getCorePropertiesCall::new((Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 2 },)), (7, 15))]
    #[case(getCorePropertiesCall::new((Star { name: String::from("Polaris"), class: StarType::YellowDwarf, core: Core::Oxygen, size: 37 },)), (8, 16))]
    #[case(getMilkyWayMassCall::new(()), 11185u64)]
    fn test_star<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }
}

mod lab_experiment {
    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::{SolCall, SolValue};

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "elements_experiment";
        const SOURCE_PATH: &str = "tests/enums/elements_experiment.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        enum State {
            Solid,
            Liquid,
            Gas
        }
        enum Symbol  {
            H,
            He,
            C,
            N,
            O,
        }
        struct Element {
            Symbol symbol;
            uint64 boil_point;
            uint64 freezing_point;
            uint64 density;
        }

        function getElementState(Symbol symbol, uint64 temperature) external returns (State);
        function getPureSubstanceDensity(Symbol symbol) external returns (uint64);
        function getMixtureSubstanceDensity(Symbol a, Symbol b, uint8 concentration) external returns (uint64);
        function runExperiments() external returns (uint64[]);
        function getDensityOfSubstances() external returns (uint64[]);
    }

    #[rstest]
    #[case(getElementStateCall::new((Symbol::H, 10u64)), State::Solid)]
    #[case(getElementStateCall::new((Symbol::H, 30u64)), State::Gas)]
    #[case(getElementStateCall::new((Symbol::He, 0u64)), State::Solid)]
    #[case(getElementStateCall::new((Symbol::He, 2u64)), State::Liquid)]
    #[case(getElementStateCall::new((Symbol::C, 2000u64)), State::Solid)]
    #[case(getElementStateCall::new((Symbol::C, 4000u64)), State::Gas)]
    #[case(getElementStateCall::new((Symbol::N, 50u64)), State::Solid)]
    #[case(getElementStateCall::new((Symbol::N, 70u64)), State::Liquid)]
    #[case(getElementStateCall::new((Symbol::O, 40u64)), State::Solid)]
    #[case(getElementStateCall::new((Symbol::O, 70u64)), State::Liquid)]
    #[case(getElementStateCall::new((Symbol::O, 100u64)), State::Gas)]
    #[case(getPureSubstanceDensityCall::new((Symbol::H,)), 899u64)]
    #[case(getPureSubstanceDensityCall::new((Symbol::C,)), 2260000u64)]
    #[case(getPureSubstanceDensityCall::new((Symbol::O,)), 1429u64)]
    #[case(getMixtureSubstanceDensityCall::new((Symbol::H, Symbol::He, 50u8)), 538u64)]
    #[case(getMixtureSubstanceDensityCall::new((Symbol::O, Symbol::C, 10u8)), 2034142u64)]
    #[case(getMixtureSubstanceDensityCall::new((Symbol::N, Symbol::O, 70u8)), 1304u64)]
    #[case(getMixtureSubstanceDensityCall::new((Symbol::He, Symbol::N, 30u8)), 929u64)]
    #[case(getMixtureSubstanceDensityCall::new((Symbol::C, Symbol::He, 90u8)), 2034017u64)]
    #[case(runExperimentsCall::new(()), vec![5000u64, 6000u64, 4000u64])]
    #[case(getDensityOfSubstancesCall::new(()), vec![1000u64, 800u64, 2000u64, 1000u64, 1200u64])]
    fn test_elements_experiment<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }
}

mod generic_enums {
    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::{SolCall, SolValue};

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "generic_enums";
        const SOURCE_PATH: &str = "tests/enums/generic_enums.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        function packUnpackFoo(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint64, uint32);
        function packUnpackFooViaWrapper(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint64, uint32);
        function packUnpackFooViaWrapper2(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint32[]);
        function packUnpackBaz(uint8 variant_index, uint32 value32) external returns (uint32);
        function packMutateUnpackFu(uint8 variant_index, uint64 value64, uint32 value32) external returns (uint64, uint32);
    }

    #[rstest]
    #[case(packUnpackFooCall::new((0u8, 42u64, 32u32)), (42u64, 32u32))]
    #[case(packUnpackFooCall::new((1u8, u64::MAX, u32::MAX)), (u64::MAX, u32::MAX))]
    #[case(packUnpackFooCall::new((2u8, 0u64, 0u32)), (0u64, 0u32))]
    #[case(packUnpackFooViaWrapperCall::new((0u8, 42u64, 32u32)), (42u64, 32u32))]
    #[case(packUnpackFooViaWrapperCall::new((1u8, u64::MAX, u32::MAX)), (u64::MAX, u32::MAX))]
    #[case(packUnpackFooViaWrapperCall::new((2u8, 0u64, 0u32)), (0u64, 0u32))]
    #[case(packUnpackBazCall::new((0u8, 42u32)), 42u32)]
    #[case(packUnpackBazCall::new((1u8, 8u32)), 24u32)]
    #[case(packUnpackBazCall::new((2u8, 33u32)), 66u32)]
    #[case(packMutateUnpackFuCall::new((0u8, 42u64, 32u32)), (43u64, 33u32))]
    #[case(packMutateUnpackFuCall::new((1u8, u64::MAX-1, u32::MAX-1)), (u64::MAX, u32::MAX))]
    #[case(packMutateUnpackFuCall::new((2u8, 0u64, 0u32)), (1u64, 1u32))]
    fn test_generic_enums<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }

    #[rstest]
    #[case(packUnpackFooViaWrapper2Call::new((0u8, 42u64, 32u32)), vec![0u32; 0])]
    #[case(packUnpackFooViaWrapper2Call::new((1u8, u64::MAX, u32::MAX)), vec![u32::MAX, u32::MAX, u32::MAX])]
    #[case(packUnpackFooViaWrapper2Call::new((2u8, 0u64, 0u32)), vec![0u32, 0u32, 0u32])]
    fn test_generic_enums_vectors<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u32>,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }
}
