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

mod enums_control_flow {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "simple_enums_control_flow";
        const SOURCE_PATH: &str = "tests/enums/simple_enums_control_flow.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        enum NumberEnum {
            One,
            Two,
            Three,
            Four,
            Five,
        }

        enum ColorEnum {
            Red,
            Green,
            Blue,
        }

        enum YinYangEnum {
            Yin,
            Yang,
        }

        function matchNumberEnum(NumberEnum x) external returns (uint32);
        function singleMatch(NumberEnum x) external returns (uint32);
        function matchNestedEnum(NumberEnum x, ColorEnum y, YinYangEnum z) external returns (uint32);
        function matchWithConditional(NumberEnum x, uint32 y) external returns (uint32);
        function nestedMatchWithConditional(NumberEnum x, ColorEnum y, uint32 z) external returns (uint32);
        function controlFlow1(NumberEnum x, ColorEnum y) external returns (uint32);
        function controlFlow1Bis(NumberEnum x, ColorEnum y) external returns (uint32);
        function controlFlow2(NumberEnum x, ColorEnum y, YinYangEnum z) external returns (uint32);
        function controlFlow2Bis(NumberEnum x, ColorEnum y, YinYangEnum z) external returns (uint32);
        function testControlFlowWithWhile(ColorEnum x) external returns (uint32);
    }

    #[rstest]
    #[case(NumberEnum::One, 11)]
    #[case(NumberEnum::Two, 22)]
    #[case(NumberEnum::Three, 33)]
    #[case(NumberEnum::Four, 44)]
    #[case(NumberEnum::Five, 44)]
    fn test_basic_enum_match(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] input: NumberEnum,
        #[case] expected: u32,
    ) {
        let call_data = matchNumberEnumCall::new((input,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(result, 0);
        assert_eq!(return_data, expected.abi_encode());
    }

    #[rstest]
    #[case(NumberEnum::One, 42)]
    #[should_panic]
    #[case(NumberEnum::Two, 0)]
    #[should_panic]
    #[case(NumberEnum::Three, 0)]
    #[should_panic]
    #[case(NumberEnum::Four, 0)]
    fn test_single_match(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] input: NumberEnum,
        #[case] expected: u32,
    ) {
        let call_data = singleMatchCall::new((input,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(result, 0);
        assert_eq!(return_data, expected.abi_encode());
    }

    #[rstest]
    #[case(NumberEnum::One, ColorEnum::Red, YinYangEnum::Yin, 11)]
    #[case(NumberEnum::Two, ColorEnum::Red, YinYangEnum::Yang, 22)]
    #[case(NumberEnum::Two, ColorEnum::Green, YinYangEnum::Yin, 33)]
    #[case(NumberEnum::Two, ColorEnum::Blue, YinYangEnum::Yang, 44)]
    #[case(NumberEnum::Three, ColorEnum::Red, YinYangEnum::Yin, 55)]
    #[case(NumberEnum::Four, ColorEnum::Blue, YinYangEnum::Yang, 66)]
    #[case(NumberEnum::Five, ColorEnum::Green, YinYangEnum::Yang, 66)]
    fn test_nested_enum_match(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] color: ColorEnum,
        #[case] yin_yang: YinYangEnum,
        #[case] expected: u32,
    ) {
        let call_data = matchNestedEnumCall::new((number, color, yin_yang)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(result, 0);
        assert_eq!(return_data, expected.abi_encode());
    }

    #[rstest]
    #[case(NumberEnum::One, 43, 1)]
    #[case(NumberEnum::Two, 44, 2)]
    #[case(NumberEnum::Three, 45, 2)]
    #[case(NumberEnum::Four, 123, 2)]
    #[case(NumberEnum::Five, 321, 2)]
    #[case(NumberEnum::Five, 10, 3)]
    #[case(NumberEnum::Four, 30, 4)]
    #[case(NumberEnum::Four, 10, 5)]
    #[case(NumberEnum::One, 0, 6)]
    #[case(NumberEnum::Two, 42, 6)]
    #[case(NumberEnum::Three, 18, 6)]
    fn test_match_with_conditional(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] y: u32,
        #[case] expected: u32,
    ) {
        let call_data = matchWithConditionalCall::new((number, y)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(result, 0);
        assert_eq!(return_data, expected.abi_encode());
    }

    #[rstest]
    #[case(NumberEnum::One, ColorEnum::Red, 43, 1)]
    #[case(NumberEnum::Two, ColorEnum::Red, 44, 2)]
    #[case(NumberEnum::Three, ColorEnum::Red, 45, 2)]
    #[case(NumberEnum::Four, ColorEnum::Red, 123, 2)]
    #[case(NumberEnum::Five, ColorEnum::Red, 321, 2)]
    #[case(NumberEnum::Five, ColorEnum::Red, 10, 3)]
    #[case(NumberEnum::Four, ColorEnum::Red, 30, 4)]
    #[case(NumberEnum::Four, ColorEnum::Blue, 30, 5)]
    #[case(NumberEnum::Four, ColorEnum::Red, 10, 6)]
    #[case(NumberEnum::Four, ColorEnum::Blue, 10, 7)]
    #[case(NumberEnum::One, ColorEnum::Blue, 10, 8)]
    fn test_nested_match_with_conditional(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] color: ColorEnum,
        #[case] z: u32,
        #[case] expected: u32,
    ) {
        let call_data = nestedMatchWithConditionalCall::new((number, color, z)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(result, 0);
        assert_eq!(return_data, expected.abi_encode());
    }

    #[rstest]
    #[case(NumberEnum::One, ColorEnum::Red, true, 11)]
    #[case(NumberEnum::Two, ColorEnum::Red, true, 44)]
    #[case(NumberEnum::Two, ColorEnum::Green, false, 33)]
    #[case(NumberEnum::Two, ColorEnum::Blue, true, 44)]
    #[case(NumberEnum::Three, ColorEnum::Blue, true, 33)]
    #[case(NumberEnum::Four, ColorEnum::Red, false, 44)]
    #[case(NumberEnum::Four, ColorEnum::Green, false, 44)]
    #[case(NumberEnum::Five, ColorEnum::Green, true, 55)]
    fn test_control_flow_1(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] color: ColorEnum,
        #[case] should_panic: bool,
        #[case] expected: u32,
    ) {
        let call_data = controlFlow1Call::new((number, color)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        if should_panic {
            assert_eq!(result, 1);
        } else {
            assert_eq!(result, 0);
            assert_eq!(return_data, expected.abi_encode());
        }
    }

    #[rstest]
    #[case(NumberEnum::One, ColorEnum::Red, true, 11)]
    #[case(NumberEnum::Two, ColorEnum::Red, true, 44)]
    #[case(NumberEnum::Two, ColorEnum::Green, false, 33)]
    #[case(NumberEnum::Two, ColorEnum::Blue, true, 44)]
    #[case(NumberEnum::Three, ColorEnum::Blue, true, 33)]
    #[case(NumberEnum::Four, ColorEnum::Red, true, 44)]
    #[case(NumberEnum::Four, ColorEnum::Green, true, 44)]
    #[case(NumberEnum::Five, ColorEnum::Green, true, 55)]
    fn test_control_flow_1_bis(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] color: ColorEnum,
        #[case] should_panic: bool,
        #[case] expected: u32,
    ) {
        let call_data = controlFlow1BisCall::new((number, color)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        if should_panic {
            assert_eq!(result, 1);
        } else {
            assert_eq!(result, 0);
            assert_eq!(return_data, expected.abi_encode());
        }
    }

    #[rstest]
    #[case(NumberEnum::One, ColorEnum::Red, YinYangEnum::Yin, 11, true)]
    #[case(NumberEnum::Two, ColorEnum::Red, YinYangEnum::Yang, 44, true)]
    #[case(NumberEnum::Two, ColorEnum::Green, YinYangEnum::Yin, 88, false)]
    #[case(NumberEnum::Two, ColorEnum::Green, YinYangEnum::Yang, 99, false)]
    #[case(NumberEnum::Two, ColorEnum::Blue, YinYangEnum::Yang, 44, true)]
    #[case(NumberEnum::Three, ColorEnum::Blue, YinYangEnum::Yang, 33, true)]
    #[case(NumberEnum::Four, ColorEnum::Red, YinYangEnum::Yin, 77, false)]
    #[case(NumberEnum::Four, ColorEnum::Green, YinYangEnum::Yin, 77, false)]
    #[case(NumberEnum::Four, ColorEnum::Green, YinYangEnum::Yang, 88, false)]
    #[case(NumberEnum::Five, ColorEnum::Green, YinYangEnum::Yang, 55, true)]
    fn test_control_flow_2(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] color: ColorEnum,
        #[case] yin_yang: YinYangEnum,
        #[case] expected: u32,
        #[case] should_panic: bool,
    ) {
        let call_data = controlFlow2Call::new((number, color, yin_yang)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        if should_panic {
            assert_eq!(result, 1);
        } else {
            assert_eq!(result, 0);
            assert_eq!(return_data, expected.abi_encode());
        }
    }

    #[rstest]
    #[case(NumberEnum::One, ColorEnum::Red, YinYangEnum::Yin, 11, true)]
    #[case(NumberEnum::Two, ColorEnum::Red, YinYangEnum::Yang, 44, true)]
    #[case(NumberEnum::Two, ColorEnum::Green, YinYangEnum::Yin, 33, true)]
    #[case(NumberEnum::Two, ColorEnum::Green, YinYangEnum::Yang, 99, false)]
    #[case(NumberEnum::Two, ColorEnum::Blue, YinYangEnum::Yang, 44, true)]
    #[case(NumberEnum::Three, ColorEnum::Blue, YinYangEnum::Yang, 33, true)]
    #[case(NumberEnum::Four, ColorEnum::Red, YinYangEnum::Yin, 44, true)]
    #[case(NumberEnum::Four, ColorEnum::Green, YinYangEnum::Yin, 44, true)]
    #[case(NumberEnum::Five, ColorEnum::Green, YinYangEnum::Yang, 55, true)]
    fn test_control_flow_2_bis(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] number: NumberEnum,
        #[case] color: ColorEnum,
        #[case] yin_yang: YinYangEnum,
        #[case] expected: u32,
        #[case] should_panic: bool,
    ) {
        let call_data = controlFlow2BisCall::new((number, color, yin_yang)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        if should_panic {
            assert_eq!(result, 1);
        } else {
            assert_eq!(result, 0);
            assert_eq!(return_data, expected.abi_encode());
        }
    }

    #[rstest]
    #[case(ColorEnum::Red, 5)]
    #[case(ColorEnum::Green, 7)]
    #[case(ColorEnum::Blue, 11)]
    fn test_control_flow_with_while(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] color: ColorEnum,
        #[case] expected: u64,
    ) {
        let call_data = testControlFlowWithWhileCall::new((color,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(result, 0);
        assert_eq!(return_data, expected.abi_encode());
    }
}

mod enums_geometry {
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

mod enums_magical_creatures {
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

mod structs_with_enums {
    use super::*;
    use alloy_sol_types::sol;
    use alloy_sol_types::{SolCall, SolValue};

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "structs_with_enums";
        const SOURCE_PATH: &str = "tests/enums/structs_with_enums.move";

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
    fn test_star<T: SolCall, V: SolValue>(
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
