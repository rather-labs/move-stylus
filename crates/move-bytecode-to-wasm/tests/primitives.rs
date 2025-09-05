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
            expected_result.abi_encode(),
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
            expected_result.abi_encode(),
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
    #[should_panic]
    #[case(echoCall::new(()), (address!("0x0000000000000000000000000000000007030507"),))]
    #[should_panic]
    #[case(echoIdentityCall::new(()), (address!("0x0000000000000000000000000000000007030507"),))]
    #[should_panic]
    #[case(echoWithIntCall::new((42,)), (42, address!("0x0000000000000000000000000000000007030507")))]
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
            expected_result.abi_encode(),
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
            expected_result.abi_encode(),
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
            <(&i32,)>::abi_encode(&(&expected_result,)),
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
            <(&i32,)>::abi_encode(&(&expected_result,)),
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
            <(&i32,)>::abi_encode(&(&expected_result,)),
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
            expected_result.abi_encode(),
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
            <(&u16,)>::abi_encode(&(&expected_result,)),
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
            <(&u16,)>::abi_encode(&(&expected_result,)),
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
            <(&u16,)>::abi_encode(&(&expected_result,)),
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
    #[case(sumCall::new((0, 1)), (1,))]
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
            expected_result.abi_encode(),
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
            <(&u32,)>::abi_encode(&(&expected_result,)),
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
            <(&u32,)>::abi_encode(&(&expected_result,)),
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
            <(&u32,)>::abi_encode(&(&expected_result,)),
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
            expected_result.abi_encode(),
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
            <(&u64,)>::abi_encode(&(&expected_result,)),
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
            <(&u64,)>::abi_encode(&(&expected_result,)),
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
            <(&u64,)>::abi_encode(&(&expected_result,)),
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
            expected_result.abi_encode(),
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
            expected_result.abi_encode(),
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
            expected_result.abi_encode(),
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
            <(&u128,)>::abi_encode(&(&expected_result,)),
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
            <(&u128,)>::abi_encode(&(&expected_result,)),
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
            <(&u128,)>::abi_encode(&(&expected_result,)),
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
            expected_result.abi_encode(),
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
            expected_result.abi_encode(),
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
            expected_result.abi_encode(),
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
            <(&U256,)>::abi_encode(&(&expected_result,)),
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
            <(&U256,)>::abi_encode(&(&expected_result,)),
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
            <(&U256,)>::abi_encode(&(&expected_result,)),
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
    ))>::abi_encode_sequence(&(
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
    ))>::abi_encode_sequence(&(
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
    ))>::abi_encode_sequence(&(
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

mod vec_32 {
    use super::*;

    const MODULE_NAME: &str = "vec_32";
    const SOURCE_PATH: &str = "tests/primitives/vec_32.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

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
        function vecPopBack(uint32[] x) external returns (uint32[]);
        function vecSwap(uint32[] x, uint64 id1, uint64 id2) external returns (uint32[]);
        function vecPushBack(uint32[] x, uint32 y) external returns (uint32[]);
        function vecPushAndPopBack(uint32[] x, uint32 y) external returns (uint32[]);
        function vecUnpack(uint32[] x) external returns (uint32[]);
    );

    #[rstest]
    #[case(getConstantCall::new(()), vec![1, 2, 3])]
    #[case(getConstantLocalCall::new(()), vec![1, 2, 3])]
    #[case(getLiteralCall::new(()), vec![1, 2, 3])]
    #[case(getCopiedLocalCall::new(()), vec![1, 2, 3])]
    #[case(echoCall::new((vec![1u32, 2u32, 3u32],)), vec![1, 2, 3])]
    #[case(vecFromIntCall::new((1u32, 2u32)), vec![1, 2, 1])]
    #[case(vecFromVecCall::new((vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32])), vec![vec![1, 2, 3], vec![4, 5, 6]])]
    #[case(vecFromVecAndIntCall::new((vec![1u32, 2u32, 3u32], 4u32)), vec![vec![1, 2, 3], vec![4, 4]])]
    #[case(vecLenCall::new((vec![1u32, 2u32, 3u32],)), (3u64,))]
    #[case(vecPopBackCall::new((vec![1u32, 2u32, 3u32],)), vec![1])]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((vec![1u32, 2u32, 3u32], 0u64, 3u64)), ((),))]
    #[case(vecSwapCall::new((vec![1u32, 2u32, 3u32], 0u64, 1u64)), vec![2, 1, 3])]
    #[case(vecSwapCall::new((vec![1u32, 2u32, 3u32], 0u64, 2u64)), vec![3, 2, 1])]
    #[case(vecPushBackCall::new((vec![1u32, 2u32, 3u32], 4u32)), vec![1, 2, 3, 4])]
    #[case(vecPushAndPopBackCall::new((vec![1u32, 2u32, 3u32], 4u32)), vec![1, 2, 3])]
    #[case(vecUnpackCall::new((vec![1u32, 5u32, 9u32],)), vec![3, 1, 4, 1, 5, 9])]
    fn test_vec_32<T: SolCall, V: SolValue>(
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

mod vec_64 {
    use super::*;

    const MODULE_NAME: &str = "vec_64";
    const SOURCE_PATH: &str = "tests/primitives/vec_64.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
          #[allow(missing_docs)]
          function getConstant() external returns (uint64[]);
          function getConstantLocal() external returns (uint64[]);
          function getLiteral() external returns (uint64[]);
          function getCopiedLocal() external returns (uint64[]);
          function echo(uint64[] x) external returns (uint64[]);
          function vecFromInt(uint64 x, uint64 y) external returns (uint64[]);
          function vecFromVec(uint64[] x, uint64[] y) external returns (uint64[][]);
          function vecFromVecAndInt(uint64[] x, uint64 y) external returns (uint64[][]);
          function vecLen(uint64[] x) external returns (uint64);
          function vecPopBack(uint64[] x) external returns (uint64[]);
          function vecSwap(uint64[] x, uint64 id1, uint64 id2) external returns (uint64[]);
          function vecPushBack(uint64[] x, uint64 y) external returns (uint64[]);
          function vecPushAndPopBack(uint64[] x, uint64 y) external returns (uint64[]);
          function vecUnpack(uint64[] x) external returns (uint64[]);
    );

    #[rstest]
    #[case(getConstantCall::new(()), vec![1u64, 2u64, 3u64])]
    #[case(getConstantLocalCall::new(()), vec![1u64, 2u64, 3u64])]
    #[case(getLiteralCall::new(()), vec![1u64, 2u64, 3u64])]
    #[case(getCopiedLocalCall::new(()), vec![1u64, 2u64, 3u64])]
    #[case(echoCall::new((vec![1u64, 2u64, 3u64],)), vec![1u64, 2u64, 3u64])]
    #[case(vecFromIntCall::new((1u64, 2u64)), vec![1u64, 2u64, 1u64 ])]
    #[case(vecFromVecCall::new((vec![1u64, 2u64, 3u64], vec![4u64, 5u64, 6u64])), vec![vec![1u64, 2u64, 3u64], vec![4u64, 5u64, 6u64]])]
    #[case(vecFromVecAndIntCall::new((vec![1u64, 2u64, 3u64], 4u64)), vec![vec![1, 2, 3], vec![4, 4]])]
    #[case(vecLenCall::new((vec![1u64, 2u64, 3u64],)), (3u64,))]
    #[case(vecPopBackCall::new((vec![1u64, 2u64, 3u64],)), vec![1])]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((vec![1u64, 2u64, 3u64], 0u64, 3u64)), ((),))]
    #[case(vecSwapCall::new((vec![1u64, 2u64, 3u64], 0u64, 1u64)), vec![2u64, 1u64, 3u64])]
    #[case(vecSwapCall::new((vec![1u64, 2u64, 3u64], 0u64, 2u64)), vec![3u64, 2u64, 1u64])]
    #[case(vecPushBackCall::new((vec![1u64, 2u64, 3u64], 4u64)), vec![1u64, 2u64, 3u64, 4u64, 4u64])]
    #[case(vecPushAndPopBackCall::new((vec![1u64, 2u64, 3u64], 4u64)), vec![1u64, 2u64, 3u64])]
    #[case(vecUnpackCall::new((vec![1u64, 5u64, 9u64],)), vec![3, 1, 4, 1, 5, 9])]
    fn test_vec_64<T: SolCall, V: SolValue>(
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

mod vec_128 {
    use super::*;

    const MODULE_NAME: &str = "vec_128";
    const SOURCE_PATH: &str = "tests/primitives/vec_128.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

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
        function vecLen(uint128[] x) external returns (uint64);
        function vecPopBack(uint128[] x) external returns (uint128[]);
        function vecSwap(uint128[] x, uint64 id1, uint64 id2) external returns (uint128[]);
        function vecPushBack(uint128[] x, uint128 y) external returns (uint128[]);
        function vecPushAndPopBack(uint128[] x, uint128 y) external returns (uint128[]);
        function vecUnpack(uint128[] x) external returns (uint128[]);
    );

    #[rstest]
    #[case(getConstantCall::new(()), vec![1u128, 2u128, 3u128])]
    #[case(getConstantLocalCall::new(()), vec![1u128, 2u128, 3u128])]
    #[case(getLiteralCall::new(()), vec![1u128, 2u128, 3u128])]
    #[case(getCopiedLocalCall::new(()), vec![1u128, 2u128, 3u128])]
    #[case(echoCall::new((vec![1u128, 2u128, 3u128],)), vec![1u128, 2u128, 3u128])]
    #[case(vecFromIntCall::new((1u128, 2u128)), vec![1u128, 2u128, 1u128])]
    #[case(vecFromVecCall::new((vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128])), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128]])]
    #[case(vecFromVecAndIntCall::new((vec![1u128, 2u128, 3u128], 4u128)), vec![vec![1u128, 2u128, 3u128], vec![4u128, 4u128]])]
    #[case(vecLenCall::new((vec![1u128, 2u128, 3u128],)), (3u64,))]
    #[case(vecPopBackCall::new((vec![1u128, 2u128, 3u128],)), vec![1u128])]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((vec![1u128, 2u128, 3u128], 0u64, 3u64)), ((),))]
    #[case(vecSwapCall::new((vec![1u128, 2u128, 3u128], 0u64, 1u64)), vec![2u128, 1u128, 3u128])]
    #[case(vecSwapCall::new((vec![1u128, 2u128, 3u128], 0u64, 2u64)), vec![3u128, 2u128, 1u128])]
    #[case(vecPushBackCall::new((vec![1u128, 2u128, 3u128], 4u128)), vec![1u128, 2u128, 3u128, 4u128, 4u128])]
    #[case(vecPushAndPopBackCall::new((vec![1u128, 2u128, 3u128], 4u128)), vec![1u128, 2u128, 3u128])]
    #[case(vecUnpackCall::new((vec![1u128, 5u128, 9u128],)), vec![3, 1, 4, 1, 5, 9])]
    fn test_vec_128<T: SolCall, V: SolValue>(
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

mod vec_vec_32 {
    use super::*;

    const MODULE_NAME: &str = "vec_vec_32";
    const SOURCE_PATH: &str = "tests/primitives/vec_vec_32.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint32[][]);
        function getConstantLocal() external returns (uint32[][]);
        function getLiteral() external returns (uint32[][]);
        function getCopiedLocal() external returns (uint32[][]);
        function echo(uint32[][] x) external returns (uint32[][]);
        function vecLen(uint32[][] x) external returns (uint64);
        function vecPopBack(uint32[][] x) external returns (uint32[][]);
        function vecSwap(uint32[][] x, uint64 id1, uint64 id2) external returns (uint32[][]);
        function vecPushBack(uint32[][] x, uint32[] y) external returns (uint32[][]);
        function vecPushBackToElement(uint32[][] x, uint32 y) external returns (uint32[][]);
        function vecPushAndPopBack(uint32[][] x, uint32[] y) external returns (uint32[][]);
        function misc0(uint32[][] x, uint32 y) external returns (uint32[][]);
        function vecUnpack(uint32[][] x) external returns (uint32[][]);
    );

    #[rstest]
    #[case(getConstantCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
    #[case(getConstantLocalCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
    #[case(getLiteralCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
    #[case(getCopiedLocalCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
    #[case(echoCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]],)), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
    #[case(vecLenCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]],)), (3u64,))]
    #[case(vecPopBackCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]],)), vec![vec![1u32, 2u32, 3u32],])]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]], 0u64, 3u64)), ((),))]
    #[case(vecSwapCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]], 0u64, 1u64)), vec![vec![4u32, 5u32, 6u32], vec![1u32, 2u32, 3u32], vec![7u32, 8u32, 9u32]])]
    #[case(vecSwapCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]], 0u64, 2u64)), vec![vec![7u32, 8u32, 9u32], vec![4u32, 5u32, 6u32], vec![1u32, 2u32, 3u32]])]
    #[case(vecPushBackCall::new((vec![vec![1u32, 2u32], vec![3u32, 4u32]], vec![5u32, 6u32])), vec![vec![1u32, 2u32], vec![3u32, 4u32], vec![5u32, 6u32], vec![5u32, 6u32]])]
    #[case(vecPushAndPopBackCall::new((vec![vec![1u32, 2u32], vec![3u32, 4u32]], vec![5u32, 6u32])), vec![vec![1u32, 2u32], vec![3u32, 4u32]])]
    #[case(misc0Call::new((vec![vec![1u32, 2u32], vec![3u32, 4u32]], 99u32)), vec![vec![1u32, 2u32, 99u32], vec![4u32, 99u32]])]
    #[case(vecUnpackCall::new((vec![vec![1u32], vec![5u32], vec![9u32]],)), vec![vec![3], vec![1], vec![4], vec![1], vec![5], vec![9]])]
    fn test_vec_vec_32<T: SolCall, V: SolValue>(
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

mod vec_vec_128 {
    use super::*;

    const MODULE_NAME: &str = "vec_vec_128";
    const SOURCE_PATH: &str = "tests/primitives/vec_vec_128.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128[][]);
        function getConstantLocal() external returns (uint128[][]);
        function getLiteral() external returns (uint128[][]);
        function getCopiedLocal() external returns (uint128[][]);
        function echo(uint128[][] x) external returns (uint128[][]);
        function vecLen(uint128[][] x) external returns (uint64);
        function vecPopBack(uint128[][] x) external returns (uint128[][]);
        function vecSwap(uint128[][] x, uint64 id1, uint64 id2) external returns (uint128[][]);
        function vecPushBack(uint128[][] x, uint128[] y) external returns (uint128[][]);
        function vecPushBackToElement(uint128[][] x, uint128 y) external returns (uint128[][]);
        function vecPushAndPopBack(uint128[][] x, uint128[] y) external returns (uint128[][]);
        function misc0(uint128[][] x, uint128 y) external returns (uint128[][]);
        function vecUnpack(uint128[][] x) external returns (uint128[][]);
    );

    #[rstest]
    #[case(getConstantCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
    #[case(getConstantLocalCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
    #[case(getLiteralCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
    #[case(getCopiedLocalCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
    #[case(echoCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]],)), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
    #[case(vecLenCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]],)), (3u64,))]
    #[case(vecPopBackCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]],)), vec![vec![1u128, 2u128, 3u128],])]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]], 0u64, 3u64)), ((),))]
    #[case(vecSwapCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]], 0u64, 1u64)), vec![vec![4u128, 5u128, 6u128], vec![1u128, 2u128, 3u128], vec![7u128, 8u128, 9u128]])]
    #[case(vecSwapCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]], 0u64, 2u64)), vec![vec![7u128, 8u128, 9u128], vec![4u128, 5u128, 6u128], vec![1u128, 2u128, 3u128]])]
    #[case(vecPushBackCall::new((vec![vec![1u128, 2u128], vec![3u128, 4u128]], vec![5u128, 6u128])), vec![vec![1u128, 2u128], vec![3u128, 4u128], vec![5u128, 6u128], vec![5u128, 6u128]])]
    #[case(vecPushAndPopBackCall::new((vec![vec![1u128, 2u128], vec![3u128, 4u128]], vec![5u128, 6u128])), vec![vec![1u128, 2u128], vec![3u128, 4u128]])]
    #[case(misc0Call::new((vec![vec![1u128, 2u128], vec![3u128, 4u128]], 99u128)), vec![vec![1u128, 2u128, 99u128], vec![4u128, 99u128]])]
    #[case(vecUnpackCall::new((vec![vec![1u128], vec![5u128], vec![9u128]],)), vec![vec![3], vec![1], vec![4], vec![1], vec![5], vec![9]])]
    fn test_vec_vec_128<T: SolCall, V: SolValue>(
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

mod vec_struct {
    use super::*;

    const MODULE_NAME: &str = "vec_struct";
    const SOURCE_PATH: &str = "tests/primitives/vec_struct.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]

        struct Foo {
            address q;
            uint32[] r;
            uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Bar bar;
            Baz baz;
        }

        struct Bar {
            uint16 a;
            uint128 b;
        }

        struct Baz {
            uint16 a;
            uint256[] b;
        }

        function getLiteral() external returns (Foo[]);
        function getCopiedLocal() external returns (Foo[]);
        function echo(Foo[] x) external returns (Foo[]);
        function vecFromStruct(Foo x, Foo y) external returns (Foo[]);
        function vecFromVec(Foo[] x, Foo[] y) external returns (Foo[][]);
        function vecFromVecAndStruct(Foo[] x, Foo y) external returns (Foo [][]);
        function vecLen(Foo[] x) external returns (uint64);
        function vecPopBack(Foo[] x) external returns (Foo[]);
        function vecSwap(Foo[] x, uint64 id1, uint64 id2) external returns (Foo[]);
        function vecPushBack(Foo[] x, Foo y) external returns (Foo[]);
        function vecPushAndPopBack(Foo[] x, Foo y) external returns (Foo[]);
        function vecEq(Foo[] x, Foo[] y) external returns (bool);
        function vecNeq(Foo[] x, Foo[] y) external returns (bool);
        function vecBorrow(Foo[] x) external returns (Foo);
        function vecMutBorrow(Foo[] x) external returns (Foo);
        function vecUnpack(Foo[] x) external returns (Foo[]);
    );

    fn get_foo_vector() -> Vec<Foo> {
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
        ]
    }

    fn get_new_fooo() -> Foo {
        Foo {
            q: address!("0x00000000000000000000000000000004deadbeef"),
            r: vec![4, 3, 0, 3, 4, 5, 6],
            s: vec![4, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 44,
            v: 44242,
            w: 4424242,
            x: 442424242,
            y: 44242424242,
            z: U256::from(4424242424242_u128),
            bar: Bar { a: 442, b: 44242 },
            baz: Baz {
                a: 44242,
                b: vec![U256::from(4)],
            },
        }
    }

    #[rstest]
    #[case(getLiteralCall::new(()), get_foo_vector())]
    #[case(getCopiedLocalCall::new(()), get_foo_vector())]
    #[case(echoCall::new((get_foo_vector(),)), get_foo_vector())]
    #[case(
        vecFromStructCall::new((
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            }
        )),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
    #[case(vecFromVecCall::new((get_foo_vector(), get_foo_vector())), vec![get_foo_vector(), get_foo_vector()])]
    #[case(
        vecFromVecAndStructCall::new((
            get_foo_vector(),
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        )),
        vec![
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar { a: 142, b: 14242 },
                    baz: Baz {
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                },
                Foo {
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar { a: 142, b: 14242 },
                    baz: Baz {
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                }
            ]
        ]
    )]
    #[case(vecLenCall::new((get_foo_vector(),)), (3u64,))]
    #[case(
        vecPopBackCall::new((get_foo_vector(),)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((get_foo_vector(), 0u64, 3u64)), ((),))]
    #[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 1u64)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            }
        ]
    )]
    #[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 2u64)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
        ])]
    #[case(
        vecPushBackCall::new((
            get_foo_vector(),
            Foo {
                q: address!("0x00000000000000000000000000000004deadbeef"),
                r: vec![4, 3, 0, 3, 4, 5, 6],
                s: vec![4, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 44,
                v: 44242,
                w: 4424242,
                x: 442424242,
                y: 44242424242,
                z: U256::from(4424242424242_u128),
                bar: Bar { a: 442, b: 44242 },
                baz: Baz {
                    a: 44242,
                    b: vec![U256::from(4)],
                },
            }
        )),
        [get_foo_vector(), vec![get_new_fooo(), get_new_fooo()]].concat()
    )]
    #[case(vecPushAndPopBackCall::new((get_foo_vector(), get_new_fooo())), get_foo_vector())]
    #[case(vecEqCall::new((get_foo_vector(), get_foo_vector())), (true,))]
    #[case(
        vecEqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar { a: 442, b: 44242 },
                    baz: Baz {
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (false,)
    )]
    #[case(vecNeqCall::new((get_foo_vector(), get_foo_vector())), (false,))]
    #[case(
        vecNeqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar { a: 442, b: 44242 },
                    baz: Baz {
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (true,)
    )]
    #[case(vecBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
    #[case(vecMutBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
    #[case(vecUnpackCall::new((get_foo_vector(),)), [get_foo_vector(), get_foo_vector()].concat())]
    fn test_vec_struct<T: SolCall, V: SolValue>(
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

mod vec_external_struct {
    use crate::common::translate_test_complete_package;

    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_packages = translate_test_complete_package("tests/primitives/external");

        let translated_package = translated_packages.get_mut("vec_external_struct").unwrap();
        RuntimeSandbox::new(translated_package)
    }

    sol!(
        #[allow(missing_docs)]

        struct Foo {
            address q;
            uint32[] r;
            uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Bar bar;
            Baz baz;
        }

        struct Bar {
            uint16 a;
            uint128 b;
        }

        struct Baz {
            uint16 a;
            uint256[] b;
        }

        function getLiteral() external returns (Foo[]);
        function getCopiedLocal() external returns (Foo[]);
        function echo(Foo[] x) external returns (Foo[]);
        function vecFromStruct(Foo x, Foo y) external returns (Foo[]);
        function vecFromVec(Foo[] x, Foo[] y) external returns (Foo[][]);
        function vecFromVecAndStruct(Foo[] x, Foo y) external returns (Foo [][]);
        function vecLen(Foo[] x) external returns (uint64);
        function vecPopBack(Foo[] x) external returns (Foo[]);
        function vecSwap(Foo[] x, uint64 id1, uint64 id2) external returns (Foo[]);
        function vecPushBack(Foo[] x, Foo y) external returns (Foo[]);
        function vecPushAndPopBack(Foo[] x, Foo y) external returns (Foo[]);
        function vecEq(Foo[] x, Foo[] y) external returns (bool);
        function vecNeq(Foo[] x, Foo[] y) external returns (bool);
        function vecBorrow(Foo[] x) external returns (Foo);
        function vecMutBorrow(Foo[] x) external returns (Foo);
    );

    fn get_foo_vector() -> Vec<Foo> {
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
        ]
    }

    fn get_new_fooo() -> Foo {
        Foo {
            q: address!("0x00000000000000000000000000000004deadbeef"),
            r: vec![4, 3, 0, 3, 4, 5, 6],
            s: vec![4, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 44,
            v: 44242,
            w: 4424242,
            x: 442424242,
            y: 44242424242,
            z: U256::from(4424242424242_u128),
            bar: Bar { a: 442, b: 44242 },
            baz: Baz {
                a: 44242,
                b: vec![U256::from(4)],
            },
        }
    }

    #[rstest]
    #[case(getLiteralCall::new(()), get_foo_vector())]
    #[case(getCopiedLocalCall::new(()), get_foo_vector())]
    #[case(echoCall::new((get_foo_vector(),)), get_foo_vector())]
    #[case(
        vecFromStructCall::new((
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            }
        )),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
    #[case(vecFromVecCall::new((get_foo_vector(), get_foo_vector())), vec![get_foo_vector(), get_foo_vector()])]
    #[case(
        vecFromVecAndStructCall::new((
            get_foo_vector(),
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        )),
        vec![
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar { a: 142, b: 14242 },
                    baz: Baz {
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                },
                Foo {
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar { a: 142, b: 14242 },
                    baz: Baz {
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                }
            ]
        ]
    )]
    #[case(vecLenCall::new((get_foo_vector(),)), (3u64,))]
    #[case(
        vecPopBackCall::new((get_foo_vector(),)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((get_foo_vector(), 0u64, 3u64)), ((),))]
    #[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 1u64)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            }
        ]
    )]
    #[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 2u64)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
        ])]
    #[case(
        vecPushBackCall::new((
            get_foo_vector(),
            Foo {
                q: address!("0x00000000000000000000000000000004deadbeef"),
                r: vec![4, 3, 0, 3, 4, 5, 6],
                s: vec![4, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 44,
                v: 44242,
                w: 4424242,
                x: 442424242,
                y: 44242424242,
                z: U256::from(4424242424242_u128),
                bar: Bar { a: 442, b: 44242 },
                baz: Baz {
                    a: 44242,
                    b: vec![U256::from(4)],
                },
            }
        )),
        [get_foo_vector(), vec![get_new_fooo(), get_new_fooo()]].concat()
    )]
    #[case(vecPushAndPopBackCall::new((get_foo_vector(), get_new_fooo())), get_foo_vector())]
    #[case(vecEqCall::new((get_foo_vector(), get_foo_vector())), (true,))]
    #[case(
        vecEqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar { a: 442, b: 44242 },
                    baz: Baz {
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (false,)
    )]
    #[case(vecNeqCall::new((get_foo_vector(), get_foo_vector())), (false,))]
    #[case(
        vecNeqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar { a: 442, b: 44242 },
                    baz: Baz {
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (true,)
    )]
    #[case(vecBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
    #[case(vecMutBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
    fn test_vec_external_struct<T: SolCall, V: SolValue>(
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

mod vec_external_generic_struct {
    use crate::common::translate_test_complete_package;

    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_packages = translate_test_complete_package("tests/primitives/external");

        let translated_package = translated_packages
            .get_mut("vec_external_generic_struct")
            .unwrap();
        RuntimeSandbox::new(translated_package)
    }

    sol!(
        #[allow(missing_docs)]

        struct Foo {
            uint32 g;
            address q;
            uint32[] r;
            uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Bar bar;
            Baz baz;
        }

        struct Bar {
            uint32 g;
            uint16 a;
            uint128 b;
        }

        struct Baz {
            uint32 g;
            uint16 a;
            uint256[] b;
        }

        function getLiteral() external returns (Foo[]);
        function getCopiedLocal() external returns (Foo[]);
        function echo(Foo[] x) external returns (Foo[]);
        function vecFromStruct(Foo x, Foo y) external returns (Foo[]);
        function vecFromVec(Foo[] x, Foo[] y) external returns (Foo[][]);
        function vecFromVecAndStruct(Foo[] x, Foo y) external returns (Foo [][]);
        function vecLen(Foo[] x) external returns (uint64);
        function vecPopBack(Foo[] x) external returns (Foo[]);
        function vecSwap(Foo[] x, uint64 id1, uint64 id2) external returns (Foo[]);
        function vecPushBack(Foo[] x, Foo y) external returns (Foo[]);
        function vecPushAndPopBack(Foo[] x, Foo y) external returns (Foo[]);
        function vecEq(Foo[] x, Foo[] y) external returns (bool);
        function vecNeq(Foo[] x, Foo[] y) external returns (bool);
        function vecBorrow(Foo[] x) external returns (Foo);
        function vecMutBorrow(Foo[] x) external returns (Foo);
    );

    fn get_foo_vector() -> Vec<Foo> {
        vec![
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242,
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                g: 2,
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar {
                    g: 2,
                    a: 242,
                    b: 24242,
                },
                baz: Baz {
                    g: 2,
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                g: 3,
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar {
                    g: 3,
                    a: 342,
                    b: 34242,
                },
                baz: Baz {
                    g: 3,
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
        ]
    }

    fn get_new_fooo() -> Foo {
        Foo {
            g: 4,
            q: address!("0x00000000000000000000000000000004deadbeef"),
            r: vec![4, 3, 0, 3, 4, 5, 6],
            s: vec![4, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 44,
            v: 44242,
            w: 4424242,
            x: 442424242,
            y: 44242424242,
            z: U256::from(4424242424242_u128),
            bar: Bar {
                g: 4,
                a: 442,
                b: 44242,
            },
            baz: Baz {
                g: 4,
                a: 44242,
                b: vec![U256::from(4)],
            },
        }
    }

    #[rstest]
    #[case(getLiteralCall::new(()), get_foo_vector())]
    #[case(getCopiedLocalCall::new(()), get_foo_vector())]
    #[case(echoCall::new((get_foo_vector(),)), get_foo_vector())]
    #[case(
        vecFromStructCall::new((
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                g: 2,
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar {
                    g: 2,
                    a: 242,
                    b: 24242
                },
                baz: Baz {
                    g: 2,
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            }
        )),
        vec![
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                g: 2,
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar {
                    g: 2,
                    a: 242,
                    b: 24242
                },
                baz: Baz {
                    g: 2,
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
    #[case(vecFromVecCall::new((get_foo_vector(), get_foo_vector())), vec![get_foo_vector(), get_foo_vector()])]
    #[case(
        vecFromVecAndStructCall::new((
            get_foo_vector(),
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        )),
        vec![
            get_foo_vector(),
            vec![
                Foo {
                    g: 1,
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar {
                        g: 1,
                        a: 142,
                        b: 14242
                    },
                    baz: Baz {
                        g: 1,
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                },
                Foo {
                    g: 1,
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar {
                        g: 1,
                        a: 142,
                        b: 14242
                    },
                    baz: Baz {
                        g: 1,
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                }
            ]
        ]
    )]
    #[case(vecLenCall::new((get_foo_vector(),)), (3u64,))]
    #[case(
        vecPopBackCall::new((get_foo_vector(),)),
        vec![
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
    #[should_panic(expected = "wasm trap: wasm `unreachable` instruction executed")]
    #[case(vecPopBackCall::new((vec![],)), ((),))]
    #[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
    #[case(vecSwapCall::new((get_foo_vector(), 0u64, 3u64)), ((),))]
    #[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 1u64)),
        vec![
            Foo {
                g: 2,
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar {
                    g: 2,
                    a: 242,
                    b: 24242
                },
                baz: Baz {
                    g: 2,
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                g: 3,
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar {
                    g: 3,
                    a: 342,
                    b: 34242
                },
                baz: Baz {
                    g: 3,
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            }
        ]
    )]
    #[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 2u64)),
        vec![
            Foo {
                g: 3,
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar {
                    g: 3,
                    a: 342,
                    b: 34242
                },
                baz: Baz {
                    g: 3,
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
            Foo {
                g: 2,
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar {
                    g: 2,
                    a: 242,
                    b: 24242
                },
                baz: Baz {
                    g: 2,
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                g: 1,
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar {
                    g: 1,
                    a: 142,
                    b: 14242
                },
                baz: Baz {
                    g: 1,
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
        ])]
    #[case(
        vecPushBackCall::new((
            get_foo_vector(),
            Foo {
                g: 4,
                q: address!("0x00000000000000000000000000000004deadbeef"),
                r: vec![4, 3, 0, 3, 4, 5, 6],
                s: vec![4, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 44,
                v: 44242,
                w: 4424242,
                x: 442424242,
                y: 44242424242,
                z: U256::from(4424242424242_u128),
                bar: Bar {
                    g: 4,
                    a: 442,
                    b: 44242
                },
                baz: Baz {
                    g: 4,
                    a: 44242,
                    b: vec![U256::from(4)],
                },
            }
        )),
        [get_foo_vector(), vec![get_new_fooo(), get_new_fooo()]].concat()
    )]
    #[case(vecPushAndPopBackCall::new((get_foo_vector(), get_new_fooo())), get_foo_vector())]
    #[case(vecEqCall::new((get_foo_vector(), get_foo_vector())), (true,))]
    #[case(
        vecEqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    g: 4,
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar {
                        g: 4,
                        a: 442,
                        b: 44242
                    },
                    baz: Baz {
                        g: 4,
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (false,)
    )]
    #[case(vecNeqCall::new((get_foo_vector(), get_foo_vector())), (false,))]
    #[case(
        vecNeqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    g: 4,
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar {
                        g: 4,
                        a: 442,
                        b: 44242
                    },
                    baz: Baz {
                        g: 4,
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (true,)
    )]
    #[case(vecBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
    #[case(vecMutBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
    fn test_vec_external_struct<T: SolCall, V: SolValue>(
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
