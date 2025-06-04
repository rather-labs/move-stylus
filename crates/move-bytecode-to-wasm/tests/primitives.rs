use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
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

mod bool_type {
    use super::*;

    const MODULE_NAME: &str = "bool_type";
    const SOURCE_PATH: &str = "tests/primitives/bool.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (bool);
        function getLocal(bool _z) external returns (bool);
        function getCopiedLocal() external returns (bool, bool);
        function echo(bool x) external returns (bool);
        function echo2(bool x, bool y) external returns (bool);
        function notTrue() external returns (bool);
        function not(bool x) external returns (bool);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (true,))]
    #[case(getLocalCall::new((true,)), (false,))]
    #[case(getCopiedLocalCall::new(()), (true, false))]
    #[case(echoCall::new((true,)), (true,))]
    #[case(echoCall::new((false,)), (false,))]
    #[case(echo2Call::new((true, false)), (false,))]
    #[case(notTrueCall::new(()), (false,))]
    #[case(notCall::new((false,)), (true,))]
    #[case(notCall::new((true,)), (false,))]
    fn test_bool<T: SolCall, V: SolValue>(
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
}

mod address_type {
    use super::*;
    use alloy_primitives::address;

    const MODULE_NAME: &str = "address_type";
    const SOURCE_PATH: &str = "tests/primitives/address.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (address);
        function getLocal(address _z) external returns (address);
        function getCopiedLocal() external returns (address, address);
        function echo(address x) external returns (address);
        function echo2(address x, address y) external returns (address);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (address!("0x0000000000000000000000000000000000000001"),))]
    #[case(
        getLocalCall::new((address!("0x0000000000000000000000000000000000000022"),)),
        (address!("0x0000000000000000000000000000000000000011"),)
    )]
    #[case(
        getCopiedLocalCall::new(()),
        (
            address!("0x0000000000000000000000000000000000000001"),
            address!("0x0000000000000000000000000000000000000022")
        )
    )]
    #[case(
        echoCall::new((address!("0x0000000000000000000000000000000000000033"),)),
        (address!("0x0000000000000000000000000000000000000033"),)
    )]
    #[case(
        echoCall::new((address!("0x0000000000000000000000000000000000000044"),)),
        (address!("0x0000000000000000000000000000000000000044"),)
    )]
    #[case(
        echo2Call::new((
            address!("0x0000000000000000000000000000000000000055"),
            address!("0x0000000000000000000000000000000000000066"),
        )),
        ( address!("0x0000000000000000000000000000000000000066"),)
    )]
    fn test_address<T: SolCall, V: SolValue>(
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
}

mod signer_type {
    use alloy_primitives::address;

    use super::*;

    sol!(
        #[allow(missing_docs)]
        function echo() external returns (address);
        function echoIdentity() external returns (address);
        function echoWithInt(uint8 y) external returns (uint8, address);
    );

    const MODULE_NAME: &str = "signer_type";
    const SOURCE_PATH: &str = "tests/primitives/signer.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(echoCall::new(()), (address!("0x0000000000000000000000000000000007030507"),))]
    #[case(
        echoIdentityCall::new(()),
        (address!("0x0000000000000000000000000000000007030507"),)
    )]
    #[case(
        echoWithIntCall::new((42,)),
        (42, address!("0x0000000000000000000000000000000007030507"))
    )]
    fn test_signer<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[should_panic(expected = "only one \"signer\" argument at the beginning is admitted")]
    #[case("tests/primitives/signer_invalid_dup_signer.move")]
    #[should_panic(expected = "complex types can't contain the type \"signer\"")]
    #[case("tests/primitives/signer_invalid_nested_signer.move")]
    fn test_signer_invalid(#[case] path: &str) {
        translate_test_package(path, MODULE_NAME);
    }
}

mod uint_8 {
    use super::*;

    const MODULE_NAME: &str = "uint_8";
    const SOURCE_PATH: &str = "tests/primitives/uint_8.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint8);
        function getLocal(uint8 _z) external returns (uint8);
        function getCopiedLocal() external returns (uint8, uint8);
        function echo(uint8 x) external returns (uint8);
        function echo2(uint8 x, uint8 y) external returns (uint8);
        function sum(uint8 x, uint8 y) external returns (uint8);
        function sub(uint8 x, uint8 y) external returns (uint8);
        function div(uint8 x, uint8 y) external returns (uint8);
        function mul(uint8 x, uint8 y) external returns (uint8);
        function mod(uint8 x, uint8 y) external returns (uint8);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (88,))]
    #[case(getLocalCall::new((111,)), (50,))]
    #[case(getCopiedLocalCall::new(()), (100, 111))]
    #[case(echoCall::new((222,)), (222,))]
    #[case(echoCall::new((255,)), (255,))]
    #[case(echo2Call::new((111, 222)), (222,))]
    #[case(sumCall::new((42, 42)), (84,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(sumCall::new((255, 1)), ((),))]
    #[case(subCall::new((84, 42)), (42,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((42, 84)), ((),))]
    fn test_uint_8<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(100, 10, 10)]
    #[case(0, 5, 0)]
    #[case(42, 42, 1)]
    #[case(3, 7, 0)]
    #[case(u8::MAX, 1, u8::MAX as i32)]
    #[case(u8::MAX, u8::MAX, 1)]
    #[case(u8::MAX, 2, (u8::MAX / 2) as i32)]
    #[case(128, 64, 2)]
    #[case(127, 3, 42)]
    #[case(1, u8::MAX, 0)]
    #[case(0, u8::MAX, 0)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_8_div(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u8,
        #[case] divisor: u8,
        #[case] expected_result: i32,
    ) {
        run_test(
            runtime,
            divCall::new((dividend, divisor)).abi_encode(),
            <(&i32,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, 5, 0)]
    #[case(5, 10, 5)]
    #[case(10, 3, 1)]
    #[case(u8::MAX, 1, 0)]
    #[case(u8::MAX, 2, 1)]
    #[case(u8::MAX, u8::MAX, 0)]
    #[case(u8::MAX, u8::MAX - 1, 1)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_8_mod(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u8,
        #[case] divisor: u8,
        #[case] expected_result: i32,
    ) {
        run_test(
            runtime,
            modCall::new((dividend, divisor)).abi_encode(),
            <(&i32,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, u8::MAX, 0)]
    #[case(u8::MAX, 0, 0)]
    #[case(1, u8::MAX, u8::MAX as i32)]
    #[case(u8::MAX, 1, u8::MAX as i32)]
    #[case(127, 2, 254)]
    #[case(21, 4, 84)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u8::MAX, 2, -1)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(16, 16, -1)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(17, 17, -1)]
    fn test_uint_8_mul(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] n1: u8,
        #[case] n2: u8,
        #[case] expected_result: i32,
    ) {
        run_test(
            runtime,
            mulCall::new((n1, n2)).abi_encode(),
            <(&i32,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }
}

mod uint_16 {
    use super::*;

    const MODULE_NAME: &str = "uint_16";
    const SOURCE_PATH: &str = "tests/primitives/uint_16.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint16);
        function getLocal(uint16 _z) external returns (uint16);
        function getCopiedLocal() external returns (uint16, uint16);
        function echo(uint16 x) external returns (uint16);
        function echo2(uint16 x, uint16 y) external returns (uint16);
        function sum(uint16 x, uint16 y) external returns (uint16);
        function sub(uint16 x, uint16 y) external returns (uint16);
        function div(uint16 x, uint16 y) external returns (uint16);
        function mul(uint16 x, uint16 y) external returns (uint16);
        function mod(uint16 x, uint16 y) external returns (uint16);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (1616,))]
    #[case(getLocalCall::new((111,)), (50,))]
    #[case(getCopiedLocalCall::new(()), (100, 111))]
    #[case(echoCall::new((222,)), (222,))]
    #[case(echoCall::new((u16::MAX,)), (u16::MAX,))]
    #[case(echo2Call::new((111, 222)), (222,))]
    #[case(sumCall::new((255, 255)), (510,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(sumCall::new((u16::MAX, 1)), ((),))]
    #[case(subCall::new((510, 255)), (255,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((255, 510)), ((),))]
    fn test_uint_16<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(100, 10, 10)]
    #[case(0, 5, 0)]
    #[case(42, 42, 1)]
    #[case(3, 7, 0)]
    #[case(u16::MAX, 1, u16::MAX)]
    #[case(u16::MAX, u16::MAX, 1)]
    #[case(u16::MAX, 2, u16::MAX / 2)]
    #[case(128, 64, 2)]
    #[case(127, 3, 42)]
    #[case(1, u16::MAX, 0)]
    #[case(0, u16::MAX, 0)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_16_div(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u16,
        #[case] divisor: u16,
        #[case] expected_result: u16,
    ) {
        run_test(
            runtime,
            divCall::new((dividend, divisor)).abi_encode(),
            <(&u16,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, 5, 0)]
    #[case(5, 10, 5)]
    #[case(10, 3, 1)]
    #[case(u16::MAX, 1, 0)]
    #[case(u16::MAX, u8::MAX as u16 + 1, u8::MAX as u16)]
    #[case(u16::MAX, u16::MAX - 1, 1)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_16_mod(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u16,
        #[case] divisor: u16,
        #[case] expected_result: u16,
    ) {
        run_test(
            runtime,
            modCall::new((dividend, divisor)).abi_encode(),
            <(&u16,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, u16::MAX, 0)]
    #[case(u16::MAX, 0, 0)]
    #[case(1, u16::MAX, u16::MAX)]
    #[case(u16::MAX, 1, u16::MAX)]
    #[case(32767, 2, 65534)]
    #[case(21, 4, 84)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u16::MAX, 2, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(256, 256, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(256, 257, 0)]
    fn test_uint_16_mul(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] n1: u16,
        #[case] n2: u16,
        #[case] expected_result: u16,
    ) {
        run_test(
            runtime,
            mulCall::new((n1, n2)).abi_encode(),
            <(&u16,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }
}

mod uint_32 {
    use super::*;

    const MODULE_NAME: &str = "uint_32";
    const SOURCE_PATH: &str = "tests/primitives/uint_32.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint32);
        function getLocal(uint32 _z) external returns (uint32);
        function getCopiedLocal() external returns (uint32, uint32);
        function echo(uint32 x) external returns (uint32);
        function echo2(uint32 x, uint32 y) external returns (uint32);
        function sum(uint32 x, uint32 y) external returns (uint32);
        function sub(uint32 x, uint32 y) external returns (uint32);
        function div(uint32 x, uint32 y) external returns (uint32);
        function mul(uint32 x, uint32 y) external returns (uint32);
        function mod(uint32 x, uint32 y) external returns (uint32);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (3232,))]
    #[case(getLocalCall::new((111,)), (50,))]
    #[case(getCopiedLocalCall::new(()), (100, 111))]
    #[case(echoCall::new((222,)), (222,))]
    #[case(echoCall::new((u32::MAX,)), (u32::MAX,))]
    #[case(echo2Call::new((111, 222)), (222,))]
    #[case(sumCall::new((65535, 65535)), (131070,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(sumCall::new((u32::MAX, 1)), ((),))]
    #[case(subCall::new((131070, 65535)), (65535,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((65535, 131070)), ((),))]
    fn test_uint_32<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(100, 10, 10)]
    #[case(0, 5, 0)]
    #[case(42, 42, 1)]
    #[case(3, 7, 0)]
    #[case(u32::MAX, 1, u32::MAX)]
    #[case(u32::MAX, u32::MAX, 1)]
    #[case(u32::MAX, 2, u32::MAX / 2)]
    #[case(128, 64, 2)]
    #[case(127, 3, 42)]
    #[case(1, u32::MAX, 0)]
    #[case(0, u32::MAX, 0)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_32_div(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u32,
        #[case] divisor: u32,
        #[case] expected_result: u32,
    ) {
        run_test(
            runtime,
            divCall::new((dividend, divisor)).abi_encode(),
            <(&u32,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, 5, 0)]
    #[case(5, 10, 5)]
    #[case(10, 3, 1)]
    #[case(u32::MAX, 1, 0)]
    #[case(u32::MAX, u16::MAX as u32  + 1, u16::MAX as u32)]
    #[case(u32::MAX, u32::MAX - 1, 1)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_32_mod(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u32,
        #[case] divisor: u32,
        #[case] expected_result: u32,
    ) {
        run_test(
            runtime,
            modCall::new((dividend, divisor)).abi_encode(),
            <(&u32,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, u32::MAX, 0)]
    #[case(u32::MAX, 0, 0)]
    #[case(1, u32::MAX, u32::MAX)]
    #[case(u32::MAX, 1, u32::MAX)]
    #[case(u32::MAX / 2, 2, u32::MAX - 1)]
    #[case(21, 4, 84)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u32::MAX, 2, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u16::MAX as u32 + 1, u16::MAX as u32 + 1, 0)]
    fn test_uint_32_mul(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] n1: u32,
        #[case] n2: u32,
        #[case] expected_result: u32,
    ) {
        run_test(
            runtime,
            mulCall::new((n1, n2)).abi_encode(),
            <(&u32,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }
}

mod uint_64 {
    use super::*;

    const MODULE_NAME: &str = "uint_64";
    const SOURCE_PATH: &str = "tests/primitives/uint_64.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint64);
        function getLocal(uint64 _z) external returns (uint64);
        function getCopiedLocal() external returns (uint64, uint64);
        function echo(uint64 x) external returns (uint64);
        function echo2(uint64 x, uint64 y) external returns (uint64);
        function sum(uint64 x, uint64 y) external returns (uint64);
        function sub(uint64 x, uint64 y) external returns (uint64);
        function div(uint64 x, uint64 y) external returns (uint64);
        function mul(uint64 x, uint64 y) external returns (uint64);
        function mod(uint64 x, uint64 y) external returns (uint64);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (6464,))]
    #[case(getLocalCall::new((111,)), (50,))]
    #[case(getCopiedLocalCall::new(()), (100, 111))]
    #[case(echoCall::new((222,)), (222,))]
    #[case(echoCall::new((u64::MAX,)), (u64::MAX,))]
    #[case(echo2Call::new((111, 222)), (222,))]
    #[case(sumCall::new((4294967295, 4294967295)), (8589934590_u64,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(sumCall::new((u64::MAX, 1)), ())]
    #[case(subCall::new((8589934590, 4294967295)), (4294967295_u64,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((4294967295, 8589934590)), ())]
    fn test_uint_64<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(100, 10, 10)]
    #[case(0, 5, 0)]
    #[case(42, 42, 1)]
    #[case(3, 7, 0)]
    #[case(u64::MAX, 1, u64::MAX)]
    #[case(u64::MAX, u64::MAX, 1)]
    #[case(u64::MAX, 2, u64::MAX / 2)]
    #[case(128, 64, 2)]
    #[case(127, 3, 42)]
    #[case(1, u64::MAX, 0)]
    #[case(0, u64::MAX, 0)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_64_div(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u64,
        #[case] divisor: u64,
        #[case] expected_result: u64,
    ) {
        run_test(
            runtime,
            divCall::new((dividend, divisor)).abi_encode(),
            <(&u64,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, 5, 0)]
    #[case(5, 10, 5)]
    #[case(10, 3, 1)]
    #[case(u64::MAX, 1, 0)]
    #[case(u64::MAX, u32::MAX as u64 + 1, u32::MAX as u64)]
    #[case(u64::MAX, u64::MAX - 1, 1)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_32_mod(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u64,
        #[case] divisor: u64,
        #[case] expected_result: u64,
    ) {
        run_test(
            runtime,
            modCall::new((dividend, divisor)).abi_encode(),
            <(&u64,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(0, u64::MAX, 0)]
    #[case(u64::MAX, 0, 0)]
    #[case(1, u64::MAX, u64::MAX)]
    #[case(u64::MAX, 1, u64::MAX)]
    #[case(u64::MAX / 2, 2, u64::MAX - 1)]
    #[case(21, 4, 84)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u64::MAX, 2, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u32::MAX as u64 + 1, u32::MAX as u64 + 1, 0)]
    fn test_uint_64_mul(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] n1: u64,
        #[case] n2: u64,
        #[case] expected_result: u64,
    ) {
        run_test(
            runtime,
            mulCall::new((n1, n2)).abi_encode(),
            <(&u64,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }
}

mod uint_128 {
    use super::*;

    const MODULE_NAME: &str = "uint_128";
    const SOURCE_PATH: &str = "tests/primitives/uint_128.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128);
        function getLocal(uint128 _z) external returns (uint128);
        function getCopiedLocal() external returns (uint128, uint128);
        function echo(uint128 x) external returns (uint128);
        function echo2(uint128 x, uint128 y) external returns (uint128);
        function sum(uint128 x, uint128 y) external returns (uint128);
        function sub(uint128 x, uint128 y) external returns (uint128);
        function mul(uint128 x, uint128 y) external returns (uint128);
        function div(uint128 x, uint128 y) external returns (uint128);
        function mod(uint128 x, uint128 y) external returns (uint128);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (128128,))]
    #[case(getLocalCall::new((111,)), (50,))]
    #[case(getCopiedLocalCall::new(()), (100, 111))]
    #[case(echoCall::new((222,)), (222,))]
    #[case(echoCall::new((u128::MAX,)), (u128::MAX,))]
    #[case(echo2Call::new((111, 222)), (222,))]
    fn test_uint_128<T: SolCall, V: SolValue>(
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

    // The following tests test two situations:
    // 1. What happens when there is carry: we process the sum in chunks of 32 bits, so we use
    //    numbers in the form 2^(n*32) where n=1,2,3,4.
    //    If we add two numbers 2^(n*64) - 1, wthe first 64 bits will overflow and we will have to
    //    take care of the carry.
    //
    //    For example
    //    2^64 - 1 = [0, ..., 0, 0, 255, 255, 255, 255]
    //
    // 2. What happens if there is not carry :
    //    If we add two numbers 2^(n*64), the first 64 bits will of both numbers will be zero, so,
    //    when we add them there will be no carry at the beginning.
    //
    //    For example
    //    2^64     = [0, ..., 0, 0, 1, 0, 0, 0, 0]
    //
    // This tests are repeated for all the 32 bits chunks in the 128bits so we test a big number
    // that does not overflows
    #[rstest]
    #[case(sumCall::new((1,1)), (2,))]
    #[case(sumCall::new((4294967295,4294967295)), (8589934590_u128,))]
    #[case(sumCall::new((4294967296,4294967296)), (8589934592_u128,))]
    #[case(sumCall::new((18446744073709551615,18446744073709551615)), (36893488147419103230_u128,))]
    #[case(sumCall::new((18446744073709551616,18446744073709551616)), (36893488147419103232_u128,))]
    #[case(sumCall::new((79228162514264337593543950335,79228162514264337593543950335)), (158456325028528675187087900670_u128,))]
    #[case(sumCall::new((79228162514264337593543950336,79228162514264337593543950336)), (158456325028528675187087900672_u128,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(sumCall::new((u128::MAX, 42)), ((),))]
    fn test_uint_128_sum<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(subCall::new((2,1)), (1,))]
    #[case(subCall::new((8589934590, 4294967295)), (4294967295_u128,))]
    #[case(subCall::new((8589934592, 4294967296)), (4294967296_u128,))]
    #[case(subCall::new((36893488147419103232, 18446744073709551616)), (18446744073709551616_u128,))]
    #[case(subCall::new((158456325028528675187087900670, 79228162514264337593543950335)), (79228162514264337593543950335_u128,))]
    #[case(subCall::new((158456325028528675187087900672, 79228162514264337593543950336)), (79228162514264337593543950336_u128,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((1, 2)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((4294967296, 8589934592)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((18446744073709551616, 36893488147419103232)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((79228162514264337593543950336, 158456325028528675187087900672)), ((),))]
    #[case(subCall::new((36893488147419103230, 18446744073709551615)), (18446744073709551615_u128,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((1, u128::MAX)), ((),))]
    fn test_uint_128_sub<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(2, 2, 4)]
    #[case(0, 2, 0)]
    #[case(2, 0, 0)]
    #[case(1, 1, 1)]
    #[case(5, 5, 25)]
    #[case(u64::MAX as u128, 2, u64::MAX as u128 * 2)]
    #[case(2, u64::MAX as u128, u64::MAX as u128 * 2)]
    #[case(2, u64::MAX as u128 + 1, (u64::MAX as u128 + 1) * 2)]
    #[case(u64::MAX as u128, u64::MAX as u128, u64::MAX as u128 * u64::MAX as u128)]
    #[case::t_2pow63_x_2pow63(
        9_223_372_036_854_775_808,
        9_223_372_036_854_775_808,
        85_070_591_730_234_615_865_843_651_857_942_052_864
    )]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u128::MAX, 2, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u128::MAX, 5, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u128::MAX, u64::MAX as u128, 0)]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(u64::MAX as u128 * 2, u64::MAX as u128 * 2, 0)]
    fn test_uint_128_mul(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] n1: u128,
        #[case] n2: u128,
        #[case] expected_result: u128,
    ) {
        run_test(
            runtime,
            mulCall::new((n1, n2)).abi_encode(),
            <(&u128,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(350, 13, 26)]
    #[case(5, 2, 2)]
    #[case(123456, 1, 123456)]
    #[case(987654321, 123456789, 8)]
    #[case(0, 2, 0)]
    // 2^96 / 2^32 = [q = 2^64, r = 0]
    #[case(79228162514264337593543950336, 4294967296, 18446744073709551616)]
    //#[should_panic(expected = "wasm trap: integer divide by zero")]
    //#[case(10, 0, 0)]
    fn test_uint_128_div(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u128,
        #[case] divisor: u128,
        #[case] expected_result: u128,
    ) {
        run_test(
            runtime,
            divCall::new((dividend, divisor)).abi_encode(),
            <(&u128,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(350, 13, 12)]
    #[case(5, 2, 1)]
    #[case(123456, 1, 0)]
    #[case(987654321, 123456789, 9)]
    #[case(0, 2, 0)]
    // 2^96 / 2^32 = [q = 2^64, r = 0]
    #[case(79228162514264337593543950336, 4294967296, 0)]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(10, 0, 0)]
    fn test_uint_128_mod(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: u128,
        #[case] divisor: u128,
        #[case] expected_result: u128,
    ) {
        run_test(
            runtime,
            modCall::new((dividend, divisor)).abi_encode(),
            <(&u128,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }
}

mod uint_256 {
    use super::*;

    const MODULE_NAME: &str = "uint_256";
    const SOURCE_PATH: &str = "tests/primitives/uint_256.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint256);
        function getLocal(uint256 _z) external returns (uint256);
        function getCopiedLocal() external returns (uint256, uint256);
        function echo(uint256 x) external returns (uint256);
        function echo2(uint256 x, uint256 y) external returns (uint256);
        function sum(uint256 x, uint256 y) external returns (uint256);
        function sub(uint256 x, uint256 y) external returns (uint256);
        function mul(uint256 x, uint256 y) external returns (uint256);
        function div(uint256 x, uint256 y) external returns (uint256);
        function mod(uint256 x, uint256 y) external returns (uint256);
    );

    #[rstest]
    #[case(getConstantCall::new(()), (256256,))]
    #[case(getLocalCall::new((U256::from(111),)), (U256::from(50),))]
    #[case(getCopiedLocalCall::new(()), (U256::from(100), U256::from(111)))]
    #[case(echoCall::new((U256::from(222),)), (U256::from(222),))]
    #[case(echoCall::new((U256::MAX,)), (U256::MAX,))]
    #[case(echo2Call::new((U256::from(111),U256::from(222))), (U256::from(222),))]
    fn test_uint_256<T: SolCall, V: SolValue>(
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

    // The following tests test two situations:
    // 1. What happens when there is carry: we process the sum in chunks of 32 bits, so we use
    //    numbers in the form 2^(n*32) where n=1,2,3,4,5,6,7,8.
    //    If we add two numbers 2^(n*64) - 1, wthe first 64 bits will overflow and we will have to
    //    take care of the carry.
    //
    //    For example
    //    2^64 - 1 = [0, ..., 0, 0, 255, 255, 255, 255]
    //
    // 2. What happens if there is not carry :
    //    If we add two numbers 2^(n*64), the first 64 bits will of both numbers will be zero, so,
    //    when we add them there will be no carry at the beginning.
    //
    //    For example
    //    2^64     = [0, ..., 0, 0, 1, 0, 0, 0, 0]
    //
    // This tests are repeated for all the 32 bits chunks in the 256bits so we test a big number
    // that does not overflows
    #[rstest]
    #[case(sumCall::new((U256::from(1), U256::from(1))), (U256::from(2),))]
    #[case(
        sumCall::new((
            U256::from(4294967295_u128),
            U256::from(4294967295_u128)
        )),
        (U256::from(8589934590_u128),))
    ]
    #[case(
        sumCall::new((
            U256::from(4294967296_u128),
            U256::from(4294967296_u128)
        )),
        (U256::from(8589934592_u128),))
    ]
    #[case(
        sumCall::new((
            U256::from(18446744073709551615_u128),
            U256::from(18446744073709551615_u128)
        )),
        (U256::from(36893488147419103230_u128),))
    ]
    #[case(
        sumCall::new((
            U256::from(18446744073709551616_u128),
            U256::from(18446744073709551616_u128)
        )),
        (U256::from(36893488147419103232_u128),))
    ]
    #[case(
        sumCall::new(
            (U256::from(79228162514264337593543950335_u128),
            U256::from(79228162514264337593543950335_u128)
        )),
        (U256::from(158456325028528675187087900670_u128),))
    ]
    #[case(
        sumCall::new((
            U256::from(79228162514264337593543950336_u128),
            U256::from(79228162514264337593543950336_u128)
        )),
        (U256::from(158456325028528675187087900672_u128),))
    ]
    #[case(
        sumCall::new((
           U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
           U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
        )),
        (U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),)
    )]
    #[case(
        sumCall::new((
           U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
           U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
        )),
        (U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),)
    )]
    #[case(
        sumCall::new((
           U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
           U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
        )),
        (U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(sumCall::new((U256::MAX, U256::from(42))), ((),))]
    fn test_uint_256_sum<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(subCall::new((U256::from(2), U256::from(1))), (1,))]
    #[case(subCall::new((U256::from(8589934590_u128), U256::from(4294967295_u128))), (4294967295_u128,))]
    #[case(subCall::new((U256::from(8589934592_u128), U256::from(4294967296_u128))), (4294967296_u128,))]
    #[case(subCall::new((U256::from(36893488147419103230_u128), U256::from(18446744073709551615_u128))), (18446744073709551615_u128,))]
    #[case(subCall::new((U256::from(36893488147419103232_u128), U256::from(18446744073709551616_u128))), (18446744073709551616_u128,))]
    #[case(subCall::new((U256::from(158456325028528675187087900670_u128), U256::from(79228162514264337593543950335_u128))), (79228162514264337593543950335_u128,))]
    #[case(subCall::new((U256::from(158456325028528675187087900672_u128), U256::from(79228162514264337593543950336_u128))), (79228162514264337593543950336_u128,))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((U256::from(1), U256::from(2))), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((U256::from(4294967296_u128), U256::from(8589934592_u128))), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((U256::from(18446744073709551616_u128), U256::from(36893488147419103232_u128))), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((U256::from(79228162514264337593543950336_u128), U256::from(158456325028528675187087900672_u128))), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        subCall::new((
           U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
           U256::from_str_radix("680564733841876926926749214863536422912", 10).unwrap(),
        )),
        ((),)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        subCall::new((
           U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
           U256::from_str_radix("680564733841876926926749214863536422910", 10).unwrap(),
        )),
        ((),)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(
        subCall::new((
           U256::from_str_radix("6277101735386680763835789423207666416102355444464034512895", 10).unwrap(),
           U256::from_str_radix("12554203470773361527671578846415332832204710888928069025790", 10).unwrap(),
        )),
        ((),)
    )]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((U256::from(1), U256::from(u128::MAX))), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(subCall::new((U256::from(1), U256::MAX)), ((),))]
    fn test_uint_256_sub<T: SolCall, V: SolValue>(
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

    #[rstest]
    #[case(U256::from(2), U256::from(2), U256::from(4))]
    #[case(U256::from(0), U256::from(2), U256::from(0))]
    #[case(U256::from(2), U256::from(0), U256::from(0))]
    #[case(U256::from(1), U256::from(1), U256::from(1))]
    #[case(U256::from(5), U256::from(5), U256::from(25))]
    #[case(U256::from(u64::MAX), U256::from(2), U256::from(u64::MAX as u128 * 2))]
    #[case(U256::from(2), U256::from(u64::MAX), U256::from(u64::MAX as u128 * 2))]
    #[case(
        U256::from(2),
        U256::from(u64::MAX as u128 + 1),
        U256::from((u64::MAX as u128 + 1) * 2)
    )]
    #[case(
        U256::from(u64::MAX),
        U256::from(u64::MAX),
        U256::from(u64::MAX as u128 * u64::MAX as u128)
    )]
    #[case::t_2pow63_x_2pow63(
        U256::from(9_223_372_036_854_775_808_u128),
        U256::from(9_223_372_036_854_775_808_u128),
        U256::from(85_070_591_730_234_615_865_843_651_857_942_052_864_u128)
    )]
    #[case(
        U256::from(u128::MAX),
        U256::from(2),
        U256::from(u128::MAX) * U256::from(2)
    )]
    #[case(
        U256::from(u128::MAX),
        U256::from(5),
        U256::from(u128::MAX) * U256::from(5)
    )]
    #[case(
        U256::from(u128::MAX),
        U256::from(u128::MAX),
        U256::from(u128::MAX) * U256::from(u128::MAX)
    )]
    #[case(
        U256::from(u64::MAX as u128 * 2),
        U256::from(u64::MAX as u128 * 2),
        U256::from(u64::MAX as u128 * 2) * U256::from(u64::MAX as u128 * 2),
    )]
    #[case(
        U256::from(2),
        U256::from(u128::MAX) + U256::from(1),
        (U256::from(u128::MAX) + U256::from(1)) * U256::from(2)
    )]
    // asd
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(U256::MAX, U256::from(2), U256::from(0))]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(U256::MAX, U256::from(5), U256::from(0))]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(U256::MAX, U256::MAX, U256::from(0))]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(
        U256::from(u128::MAX) * U256::from(2),
        U256::from(u128::MAX) * U256::from(2),
        U256::from(0),
    )]
    fn test_uint_256_mul(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] n1: U256,
        #[case] n2: U256,
        #[case] expected_result: U256,
    ) {
        run_test(
            runtime,
            mulCall::new((n1, n2)).abi_encode(),
            <(&U256,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(U256::from(350), U256::from(13), U256::from(26))]
    #[case(U256::from(5), U256::from(2), U256::from(2))]
    #[case(U256::from(123456), U256::from(1), U256::from(123456))]
    #[case(U256::from(987654321), U256::from(123456789), U256::from(8))]
    #[case(U256::from(0), U256::from(2), U256::from(0))]
    // 2^96 / 2^32 = [q = 2^64, r = 0]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(4294967296_u128),
        U256::from(18446744073709551616_u128)
    )]
    // 2^192 / 2^64 = [q = 2^128, r = 0]
    #[case(
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551616_u128),
        U256::from(u128::MAX) + U256::from(1),
    )]
    //#[should_panic(expected = "wasm trap: integer divide by zero")]
    //#[case(10, 0, 0)]
    fn test_uint_256_div(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: U256,
        #[case] divisor: U256,
        #[case] expected_result: U256,
    ) {
        run_test(
            runtime,
            divCall::new((dividend, divisor)).abi_encode(),
            <(&U256,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }

    #[rstest]
    #[case(U256::from(350), U256::from(13), U256::from(12))]
    #[case(U256::from(5), U256::from(2), U256::from(1))]
    #[case(U256::from(123456), U256::from(1), U256::from(0))]
    #[case(U256::from(987654321), U256::from(123456789), U256::from(9))]
    #[case(U256::from(0), U256::from(2), U256::from(0))]
    // 2^96 / 2^32 = [q = 2^64, r = 0]
    #[case(
        U256::from(79228162514264337593543950336_u128),
        U256::from(4294967296_u128),
        U256::from(0)
    )]
    // 2^192 / 2^64 = [q = 2^128, r = 0]
    #[case(
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512896", 10
        ).unwrap(),
        U256::from(18446744073709551616_u128),
        U256::from(0)
    )]
    #[should_panic(expected = "wasm trap: integer divide by zero")]
    #[case(U256::from(10), U256::from(0), U256::from(0))]
    fn test_uint_256_mod(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] dividend: U256,
        #[case] divisor: U256,
        #[case] expected_result: U256,
    ) {
        run_test(
            runtime,
            modCall::new((dividend, divisor)).abi_encode(),
            <(&U256,)>::abi_encode_params(&(&expected_result,)),
        )
        .unwrap();
    }
}

#[test]
fn test_multi_values_return() {
    const MODULE_NAME: &str = "multi_values_return";
    const SOURCE_PATH: &str = "tests/primitives/multi_values_return.move";

    sol!(
        #[allow(missing_docs)]
        function getConstants() external returns (uint256, uint64, uint32, uint8, bool, address, uint32[], uint128[]);
        function getConstantsReversed() external returns (uint128[], uint32[], address, bool, uint8, uint32, uint64, uint256);
        function getConstantsNested() external returns (uint256, uint64, uint32, uint8, bool, address, uint32[], uint128[]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantsCall::abi_encode(&getConstantsCall::new(()));
    let expected_result = <sol!((
        uint256,
        uint64,
        uint32,
        uint8,
        bool,
        address,
        uint32[],
        uint128[]
    ))>::abi_encode_params(&(
        U256::from(256256),
        6464,
        3232,
        88,
        true,
        address!("0x0000000000000000000000000000000000000001"),
        vec![10, 20, 30],
        vec![100, 200, 300],
    ));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantsReversedCall::abi_encode(&getConstantsReversedCall::new(()));
    let expected_result = <sol!((
        uint128[],
        uint32[],
        address,
        bool,
        uint8,
        uint32,
        uint64,
        uint256
    ))>::abi_encode_params(&(
        vec![100, 200, 300],
        vec![10, 20, 30],
        address!("0x0000000000000000000000000000000000000001"),
        true,
        88,
        3232,
        6464,
        U256::from(256256),
    ));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantsNestedCall::abi_encode(&getConstantsNestedCall::new(()));
    let expected_result = <sol!((
        uint256,
        uint64,
        uint32,
        uint8,
        bool,
        address,
        uint32[],
        uint128[]
    ))>::abi_encode_params(&(
        U256::from(256256),
        6464,
        3232,
        88,
        true,
        address!("0x0000000000000000000000000000000000000001"),
        vec![10, 20, 30],
        vec![100, 200, 300],
    ));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_32() {
    const MODULE_NAME: &str = "vec_32";
    const SOURCE_PATH: &str = "tests/primitives/vec_32.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint32[]);
        function getConstantLocal() external returns (uint32[]);
        function getLiteral() external returns (uint32[]);
        function getCopiedLocal() external returns (uint32[]);
        function echo(uint32[] x) external returns (uint32[]);
        function vecFromInt(uint32 x, uint32 y) external returns (uint32[]);
        function vecFromVec(uint32[] x, uint32[] y) external returns (uint32[][]);
        function vecFromVecAndInt(uint32[] x, uint32 y) external returns (uint32[][]);
        function vecLen(uint32[] x) external returns (uint64);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // getLiteral() should return [1, 2, 3]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [1, 2, 3]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([1, 2, 3]) should return [1, 2, 3]
    let data = echoCall::abi_encode(&echoCall::new((vec![1u32, 2u32, 3u32],)));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromInt(1, 2) should return [1, 2]
    let data = vecFromIntCall::abi_encode(&vecFromIntCall::new((1u32, 2u32)));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 1u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // vec_from_vec([1, 2, 3], [4, 5, 6]) should return [[1, 2, 3], [4, 5, 6]]
    let data = vecFromVecCall::abi_encode(&vecFromVecCall::new((
        vec![1u32, 2u32, 3u32],
        vec![4u32, 5u32, 6u32],
    )));
    let expected_result = <sol!((uint32[][],))>::abi_encode_params(&(vec![
        vec![1u32, 2u32, 3u32],
        vec![4u32, 5u32, 6u32],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromVecAndInt([1, 2, 3], 4) should return [[1, 2, 3], [4, 4]]
    let data = vecFromVecAndIntCall::abi_encode(&vecFromVecAndIntCall::new((
        vec![1u32, 2u32, 3u32],
        4u32,
    )));
    let expected_result =
        <sol!((uint32[][],))>::abi_encode_params(
            &(vec![vec![1u32, 2u32, 3u32], vec![4u32, 4u32]],),
        );
    run_test(&runtime, data, expected_result).unwrap();

    let data = vecLenCall::abi_encode(&vecLenCall::new((vec![1u32, 2u32, 3u32],)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(3u64,));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_64() {
    const MODULE_NAME: &str = "vec_64";
    const SOURCE_PATH: &str = "tests/primitives/vec_64.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint64[]);
        function getConstantLocal() external returns (uint64[]);
        function getLiteral() external returns (uint64[]);
        function getCopiedLocal() external returns (uint64[]);
        function echo(uint64[] x) external returns (uint64[]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    // getLiteral() should return [1, 2, 3]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [1, 2, 3]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([1, 2, 3]) should return [1, 2, 3]
    let data = echoCall::abi_encode(&echoCall::new((vec![1u64, 2u64, 3u64],)));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_128() {
    const MODULE_NAME: &str = "vec_128";
    const SOURCE_PATH: &str = "tests/primitives/vec_128.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128[]);
        function getConstantLocal() external returns (uint128[]);
        function getLiteral() external returns (uint128[]);
        function getCopiedLocal() external returns (uint128[]);
        function echo(uint128[] x) external returns (uint128[]);
        function vecFromInt(uint128 x, uint128 y) external returns (uint128[]);
        function vecFromVec(uint128[] x, uint128[] y) external returns (uint128[][]);
        function vecFromVecAndInt(uint128[] x, uint128 y) external returns (uint128[][]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // getLiteral() should return [1, 2, 3]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [1, 2, 3]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([1, 2, 3]) should return [1, 2, 3]
    let data = echoCall::abi_encode(&echoCall::new((vec![1u128, 2u128, 3u128],)));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromInt(1, 2) should return [1, 2, 1]
    let data = vecFromIntCall::abi_encode(&vecFromIntCall::new((1u128, 2u128)));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 1u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromVec([1, 2, 3], [4, 5, 6]) should return [[1, 2, 3], [4, 5, 6]]
    let data = vecFromVecCall::abi_encode(&vecFromVecCall::new((
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
    )));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromVecAndInt([1, 2, 3], 4) should return [[1, 2, 3], [4, 4]]
    let data = vecFromVecAndIntCall::abi_encode(&vecFromVecAndIntCall::new((
        vec![1u128, 2u128, 3u128],
        4u128,
    )));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 4u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_vec_128() {
    const MODULE_NAME: &str = "vec_vec_128";
    const SOURCE_PATH: &str = "tests/primitives/vec_vec_128.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128[][]);
        function getConstantLocal() external returns (uint128[][]);
        function getLiteral() external returns (uint128[][]);
        function getCopiedLocal() external returns (uint128[][]);
        function echo(uint128[][] x) external returns (uint128[][]);
        function vecLen(uint128[][] x) external returns (uint64);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();
    // getLiteral() should return [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([[1, 2, 3], [4, 5, 6], [7, 8, 9]]) should return the same
    let data = echoCall::abi_encode(&echoCall::new((vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],)));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = vecLenCall::abi_encode(&vecLenCall::new((vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(3u64,));
    run_test(&runtime, data, expected_result).unwrap();
}
