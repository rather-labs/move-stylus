use alloy_primitives::U256;
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
        "return data mismatch: {return_data:?} != {expected_result:?}"
    );

    Ok(())
}

mod uint_8 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function or(uint8 x, uint8 y) external returns (uint8);
        function xor(uint8 x, uint8 y) external returns (uint8);
        function and(uint8 x, uint8 y) external returns (uint8);
        function shiftLeft(uint8 x, uint8 slots) external returns (uint8);
        function shiftRight(uint8 x, uint8 slots) external returns (uint8);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_8";
        const SOURCE_PATH: &str = "tests/operations-bitwise/uint_8.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(orCall::new((250, 250)), 250)]
    #[case(orCall::new((250, 50)), 250)]
    #[case(orCall::new((250, 0)), 250)]
    #[case(orCall::new((15, 240)), 255)]
    #[case(orCall::new((240, 15)), 255)]
    #[case(orCall::new((0, 0)), 0)]
    #[case(orCall::new((u8::MAX, u8::MAX)), u8::MAX)]
    fn test_uint_8_or<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(xorCall::new((250, 250)), 0)]
    #[case(xorCall::new((250, 50)), 200)]
    #[case(xorCall::new((250, 0)), 250)]
    #[case(xorCall::new((15, 240)), 255)]
    #[case(xorCall::new((240, 15)), 255)]
    #[case(xorCall::new((u8::MAX, u8::MAX)), 0)]
    #[case(xorCall::new((0, 0)), 0)]
    #[case(xorCall::new((u8::MAX, 0)), u8::MAX)]
    fn test_uint_8_xor<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(andCall::new((250, 250)), 250)]
    #[case(andCall::new((250, 50)), 50)]
    #[case(andCall::new((250, 0)), 0)]
    #[case(andCall::new((15, 240)), 0)]
    #[case(andCall::new((240, 15)), 0)]
    #[case(andCall::new((u8::MAX, u8::MAX)), u8::MAX)]
    #[case(andCall::new((0, 0)), 0)]
    #[case(andCall::new((u8::MAX, 0)), 0)]
    fn test_uint_8_and<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(shiftLeftCall::new((255, 7)), 255 << 7)]
    #[case(shiftLeftCall::new((255, 1)), 255 << 1)]
    #[case(shiftLeftCall::new((254, 7)), 254 << 7)]
    #[case(shiftLeftCall::new((250, 0)), 250)]
    #[case(shiftLeftCall::new((250, 4)), 250 << 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((240, 8)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((240, 10)), 0)]
    #[case(shiftRightCall::new((255, 7)), 255 >> 7)]
    #[case(shiftRightCall::new((255, 1)), 255 >> 1)]
    #[case(shiftRightCall::new((254, 7)), 254 >> 7)]
    #[case(shiftRightCall::new((250, 0)), 250)]
    #[case(shiftRightCall::new((250, 4)), 250 >> 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((240, 8)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((240, 10)), 0)]
    fn test_uint_8_shift<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod uint_16 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function or(uint16 x, uint16 y) external returns (uint16);
        function xor(uint16 x, uint16 y) external returns (uint16);
        function and(uint16 x, uint16 y) external returns (uint16);
        function shiftLeft(uint16 x, uint8 slots) external returns (uint16);
        function shiftRight(uint16 x, uint8 slots) external returns (uint16);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_16";
        const SOURCE_PATH: &str = "tests/operations-bitwise/uint_16.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(orCall::new((1616, 1616)), 1616)]
    #[case(orCall::new((1616, u8::MAX as u16 + 1)), u8::MAX as u16 + 1 + 1616)]
    #[case(orCall::new((1616, 0)), 1616)]
    #[case(orCall::new((u8::MAX as u16, u16::MAX - (u8::MAX as u16))), u16::MAX)]
    #[case(orCall::new((u16::MAX - (u8::MAX as u16), u8::MAX as u16)), u16::MAX)]
    #[case(orCall::new((0, 0)), 0)]
    #[case(orCall::new((u16::MAX, u16::MAX)), u16::MAX)]
    fn test_uint_16_or<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(xorCall::new((1616, 1616)), 0)]
    #[case(xorCall::new((1616, u8::MAX as u16 + 1)), u8::MAX as u16 + 1 + 1616)]
    #[case(xorCall::new((1616, 0)), 1616)]
    #[case(xorCall::new((u8::MAX as u16, u16::MAX - (u8::MAX as u16))), u16::MAX)]
    #[case(xorCall::new((u16::MAX - (u8::MAX as u16), u8::MAX as u16)), u16::MAX)]
    #[case(xorCall::new((0, 0)), 0)]
    #[case(xorCall::new((u16::MAX, u16::MAX)), 0)]
    fn test_uint_16_xor<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(andCall::new((1616, 1616)), 1616)]
    #[case(andCall::new((1616, u8::MAX as u16 + 1)), 0)]
    #[case(andCall::new((1616, 0)), 0)]
    #[case(andCall::new((u8::MAX as u16, u16::MAX - (u8::MAX as u16))), 0)]
    #[case(andCall::new((u16::MAX - (u8::MAX as u16), u8::MAX as u16)), 0)]
    #[case(andCall::new((0, 0)), 0)]
    #[case(andCall::new((u16::MAX, u16::MAX)), u16::MAX)]
    fn test_uint_16_and<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(shiftLeftCall::new((1616, 7)), 1616 << 7)]
    #[case(shiftLeftCall::new((1616, 1)), 1616 << 1)]
    #[case(shiftLeftCall::new((1615, 7)), 1615 << 7)]
    #[case(shiftLeftCall::new((1610, 0)), 1610)]
    #[case(shiftLeftCall::new((1610, 4)), 1610 << 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((1600, 16)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((1600, 30)), 0)]
    #[case(shiftRightCall::new((1616, 7)), 1616 >> 7)]
    #[case(shiftRightCall::new((1616, 1)), 1616 >> 1)]
    #[case(shiftRightCall::new((1615, 7)), 1615 >> 7)]
    #[case(shiftRightCall::new((1610, 0)), 1610)]
    #[case(shiftRightCall::new((1610, 4)), 1610 >> 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((1600, 16)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((1600, 30)), 0)]
    fn test_uint_16_shift<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod uint_32 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function or(uint32 x, uint32 y) external returns (uint32);
        function xor(uint32 x, uint32 y) external returns (uint32);
        function and(uint32 x, uint32 y) external returns (uint32);
        function shiftLeft(uint32 x, uint8 slots) external returns (uint32);
        function shiftRight(uint32 x, uint8 slots) external returns (uint32);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_32";
        const SOURCE_PATH: &str = "tests/operations-bitwise/uint_32.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(orCall::new((3232, 3232)), 3232)]
    #[case(orCall::new((3232, u16::MAX as u32 + 1)), u16::MAX as u32 + 1 + 3232)]
    #[case(orCall::new((3232, 0)), 3232)]
    #[case(orCall::new((u16::MAX as u32, u32::MAX - (u16::MAX as u32))), u32::MAX)]
    #[case(orCall::new((u32::MAX - (u16::MAX as u32), u16::MAX as u32)), u32::MAX)]
    #[case(orCall::new((0, 0)), 0)]
    #[case(orCall::new((u32::MAX, u32::MAX)), u32::MAX)]
    fn test_uint_32_or<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        println!("expected_result: {expected_result}");
        let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(xorCall::new((3232, 3232)), 0)]
    #[case(xorCall::new((3232, u16::MAX as u32 + 1)), u16::MAX as u32 + 1 + 3232)]
    #[case(xorCall::new((3232, 0)), 3232)]
    #[case(xorCall::new((u16::MAX as u32, u32::MAX - (u16::MAX as u32))), u32::MAX)]
    #[case(xorCall::new((u32::MAX - (u16::MAX as u32), u16::MAX as u32)), u32::MAX)]
    #[case(xorCall::new((0, 0)), 0)]
    #[case(xorCall::new((u32::MAX, u32::MAX)), 0)]
    fn test_uint_32_xor<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(andCall::new((3232, 3232)), 3232)]
    #[case(andCall::new((3232, u16::MAX as u32 + 1)), 0)]
    #[case(andCall::new((3232, 0)), 0)]
    #[case(andCall::new((u16::MAX as u32, u32::MAX - (u16::MAX as u32))), 0)]
    #[case(andCall::new((u32::MAX - (u16::MAX as u32), u16::MAX as u32)), 0)]
    #[case(andCall::new((0, 0)), 0)]
    #[case(andCall::new((u32::MAX, u32::MAX)), u32::MAX)]
    fn test_uint_32_and<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(shiftLeftCall::new((3232, 7)), 3232 << 7)]
    #[case(shiftLeftCall::new((3232, 1)), 3232 << 1)]
    #[case(shiftLeftCall::new((3231, 7)), 3231 << 7)]
    #[case(shiftLeftCall::new((3226, 0)), 3226)]
    #[case(shiftLeftCall::new((3226, 4)), 3226 << 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((3200, 32)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((3200, 50)), 0)]
    #[case(shiftRightCall::new((3232, 7)), 3232 >> 7)]
    #[case(shiftRightCall::new((3232, 1)), 3232 >> 1)]
    #[case(shiftRightCall::new((3231, 7)), 3231 >> 7)]
    #[case(shiftRightCall::new((3226, 0)), 3226)]
    #[case(shiftRightCall::new((3226, 4)), 3226 >> 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((3200, 32)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((3200, 50)), 0)]
    fn test_uint_32_shift<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod uint_64 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function or(uint64 x, uint64 y) external returns (uint64);
        function xor(uint64 x, uint64 y) external returns (uint64);
        function and(uint64 x, uint64 y) external returns (uint64);
        function shiftLeft(uint64 x, uint8 slots) external returns (uint64);
        function shiftRight(uint64 x, uint8 slots) external returns (uint64);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_64";
        const SOURCE_PATH: &str = "tests/operations-bitwise/uint_64.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(orCall::new((6464, 6464)), 6464)]
    #[case(orCall::new((6464, u32::MAX as u64 + 1)), u32::MAX as u64 + 1 + 6464)]
    #[case(orCall::new((6464, 0)), 6464)]
    #[case(orCall::new((u32::MAX as u64, u64::MAX - (u32::MAX as u64))), u64::MAX)]
    #[case(orCall::new((u64::MAX - (u32::MAX as u64), u32::MAX as u64)), u64::MAX)]
    #[case(orCall::new((0, 0)), 0)]
    #[case(orCall::new((u64::MAX, u64::MAX)), u64::MAX)]
    fn test_uint_64_or<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(xorCall::new((6464, 6464)), 0)]
    #[case(xorCall::new((6464, u32::MAX as u64 + 1)), u32::MAX as u64 + 1 + 6464)]
    #[case(xorCall::new((6464, 0)), 6464)]
    #[case(xorCall::new((u32::MAX as u64, u64::MAX - (u32::MAX as u64))), u64::MAX)]
    #[case(xorCall::new((u64::MAX - (u32::MAX as u64), u32::MAX as u64)), u64::MAX)]
    #[case(xorCall::new((0, 0)), 0)]
    #[case(xorCall::new((u64::MAX, u64::MAX)), 0)]
    fn test_uint_64_xor<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(andCall::new((6464, 6464)), 6464)]
    #[case(andCall::new((6464, u32::MAX as u64 + 1)), 0)]
    #[case(andCall::new((6464, 0)), 0)]
    #[case(andCall::new((u32::MAX as u64, u64::MAX - (u32::MAX as u64))), 0)]
    #[case(andCall::new((u64::MAX - (u32::MAX as u64), u32::MAX as u64)), 0)]
    #[case(andCall::new((0, 0)), 0)]
    #[case(andCall::new((u64::MAX, u64::MAX)), u64::MAX)]
    fn test_uint_64_and<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(shiftLeftCall::new((6464, 7)), 6464 << 7)]
    #[case(shiftLeftCall::new((6464, 1)), 6464 << 1)]
    #[case(shiftLeftCall::new((6463, 7)), 6463 << 7)]
    #[case(shiftLeftCall::new((6458, 0)), 6458)]
    #[case(shiftLeftCall::new((6458, 4)), 6458 << 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((6400, 64)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((6400, 100)), 0)]
    #[case(shiftRightCall::new((6464, 7)), 6464 >> 7)]
    #[case(shiftRightCall::new((6464, 1)), 6464 >> 1)]
    #[case(shiftRightCall::new((6463, 7)), 6463 >> 7)]
    #[case(shiftRightCall::new((6458, 0)), 6458)]
    #[case(shiftRightCall::new((6458, 4)), 6458 >> 4)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((6400, 64)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((6400, 100)), 0)]
    fn test_uint_64_shift<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod uint_128 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function or(uint128 x, uint128 y) external returns (uint128);
        function xor(uint128 x, uint128 y) external returns (uint128);
        function and(uint128 x, uint128 y) external returns (uint128);
        function shiftLeft(uint128 x, uint8 slots) external returns (uint128);
        function shiftRight(uint128 x, uint8 slots) external returns (uint128);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_128";
        const SOURCE_PATH: &str = "tests/operations-bitwise/uint_128.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(orCall::new((128128, 128128)), 128128)]
    #[case(orCall::new((128128, u64::MAX as u128 + 1)), u64::MAX as u128 + 1 + 128128)]
    #[case(orCall::new((128128, 0)), 128128)]
    #[case(orCall::new((u64::MAX as u128, u128::MAX - (u64::MAX as u128))), u128::MAX)]
    #[case(orCall::new((u128::MAX - (u64::MAX as u128), u64::MAX as u128)), u128::MAX)]
    #[case(orCall::new((0, 0)), 0)]
    #[case(orCall::new((u128::MAX, u128::MAX)), u128::MAX)]
    fn test_uint_128_or<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(xorCall::new((128128, 128128)), 0)]
    #[case(xorCall::new((128128, u64::MAX as u128 + 1)), u64::MAX as u128 + 1 + 128128)]
    #[case(xorCall::new((128128, 0)), 128128)]
    #[case(xorCall::new((u64::MAX as u128, u128::MAX - (u64::MAX as u128))), u128::MAX)]
    #[case(xorCall::new((u128::MAX - (u64::MAX as u128), u64::MAX as u128)), u128::MAX)]
    #[case(xorCall::new((0, 0)), 0)]
    #[case(xorCall::new((u128::MAX, u128::MAX)), 0)]
    fn test_uint_128_xor<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(andCall::new((128128, 128128)), 128128)]
    #[case(andCall::new((128128, u64::MAX as u128 + 1)), 0)]
    #[case(andCall::new((128128, 0)), 0)]
    #[case(andCall::new((u64::MAX as u128, u128::MAX - (u64::MAX as u128))), 0)]
    #[case(andCall::new((u128::MAX - (u64::MAX as u128), u64::MAX as u128)), 0)]
    #[case(andCall::new((0, 0)), 0)]
    #[case(andCall::new((u128::MAX, u128::MAX)), u128::MAX)]
    fn test_uint_128_and<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(shiftLeftCall::new((128128, 7)), 128128 << 7)]
    #[case(shiftLeftCall::new((128128, 35)), 128128 << 35)]
    #[case(shiftLeftCall::new((128127, 68)), 128127 << 68)]
    #[case(shiftLeftCall::new((128122, 0)), 128122)]
    #[case(shiftLeftCall::new((128122, 100)), 128122 << 100)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((128000, 128)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftLeftCall::new((128000, 250)), 0)]
    #[case(shiftRightCall::new((128128, 7)), 128128 >> 7)]
    #[case(shiftRightCall::new((128128, 35)), 128128 >> 35)]
    #[case(shiftRightCall::new((128127, 68)), 128127 >> 68)]
    #[case(shiftRightCall::new((128122, 0)), 128122)]
    #[case(shiftRightCall::new((128122, 100)), 128122 >> 100)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((128000, 128)), 0)]
    #[should_panic(expected = "wasm `unreachable` instruction executed")]
    #[case(shiftRightCall::new((128000, 240)), 0)]
    fn test_uint_128_shift<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod uint_256 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function or(uint256 x, uint256 y) external returns (uint256);
        function xor(uint256 x, uint256 y) external returns (uint256);
        function and(uint256 x, uint256 y) external returns (uint256);
        function shiftLeft(uint256 x, uint8 slots) external returns (uint256);
        function shiftRight(uint256 x, uint8 slots) external returns (uint256);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_256";
        const SOURCE_PATH: &str = "tests/operations-bitwise/uint_256.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(orCall::new((U256::from(256256), U256::from(256256))), U256::from(256256))]
    #[case(orCall::new((U256::from(256256), U256::from(u128::MAX) + U256::from(1))), U256::from(u128::MAX) + U256::from(1) + U256::from(256256))]
    #[case(orCall::new((U256::from(256256), U256::from(0))), U256::from(256256))]
    #[case(orCall::new((U256::from(u128::MAX), U256::MAX - (U256::from(u128::MAX)))), U256::MAX)]
    #[case(orCall::new((U256::MAX - (U256::from(u128::MAX)), U256::from(u128::MAX))), U256::MAX)]
    #[case(orCall::new((U256::from(0), U256::from(0))), U256::from(0))]
    #[case(orCall::new((U256::MAX, U256::MAX)), U256::MAX)]
    fn test_uint_256_or<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(xorCall::new((U256::from(256256), U256::from(256256))), U256::from(0))]
    #[case(xorCall::new((U256::from(256256), U256::from(u128::MAX) + U256::from(1))), U256::from(u128::MAX) + U256::from(1) + U256::from(256256))]
    #[case(xorCall::new((U256::from(256256), U256::from(0))), U256::from(256256))]
    #[case(xorCall::new((U256::from(u128::MAX), U256::MAX - (U256::from(u128::MAX)))), U256::MAX)]
    #[case(xorCall::new((U256::MAX - (U256::from(u128::MAX)), U256::from(u128::MAX))), U256::MAX)]
    #[case(xorCall::new((U256::from(0), U256::from(0))), U256::from(0))]
    #[case(xorCall::new((U256::MAX, U256::MAX)), U256::from(0))]
    fn test_uint_256_xor<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(andCall::new((U256::from(256256), U256::from(256256))), U256::from(256256))]
    #[case(andCall::new((U256::from(256256), U256::from(u128::MAX) + U256::from(1))), U256::from(0))]
    #[case(andCall::new((U256::from(256256), U256::from(0))), U256::from(0))]
    #[case(andCall::new((U256::from(u128::MAX), U256::MAX - (U256::from(u128::MAX)))), U256::from(0))]
    #[case(andCall::new((U256::MAX - (U256::from(u128::MAX)), U256::from(u128::MAX))), U256::from(0))]
    #[case(andCall::new((U256::from(0), U256::from(0))), U256::from(0))]
    #[case(andCall::new((U256::MAX, U256::MAX)), U256::MAX)]
    fn test_uint_256_and<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(shiftLeftCall::new((U256::from(256256), 0)), U256::from(256256))]
    #[case(shiftLeftCall::new((U256::from(256256), 7)), U256::from(256256) << 7)]
    #[case(shiftLeftCall::new((U256::from(u128::MAX), 35)), U256::from(u128::MAX) << 35)]
    #[case(shiftLeftCall::new((U256::from(u128::MAX), 68)), U256::from(u128::MAX) << 68)]
    #[case(shiftLeftCall::new((U256::from(u128::MAX), 100)), U256::from(u128::MAX) << 100)]
    #[case(shiftLeftCall::new((U256::MAX, 150)), U256::MAX << 150)]
    #[case(shiftLeftCall::new((U256::MAX, 210)), U256::MAX << 210)]
    #[case(shiftRightCall::new((U256::from(256256), 0)), U256::from(256256))]
    #[case(shiftRightCall::new((U256::from(256256), 7)), U256::from(256256) >> 7)]
    #[case(shiftRightCall::new((U256::from(256256), 35)), U256::from(256256) >> 35)]
    #[case(shiftRightCall::new((U256::from(u128::MAX), 68)), U256::from(u128::MAX) >> 68)]
    #[case(shiftRightCall::new((U256::from(u128::MAX), 100)), U256::from(u128::MAX) >> 100)]
    #[case(shiftRightCall::new((U256::MAX, 150)), U256::MAX >> 150)]
    #[case(shiftRightCall::new((U256::MAX, 210)), U256::MAX >> 210)]
    fn test_uint_256_shift<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
