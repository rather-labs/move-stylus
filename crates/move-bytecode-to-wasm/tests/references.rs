use alloy_primitives::U256;
use alloy_sol_types::abi::TokenSeq;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    println!("return_data: {:?}", return_data);
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

mod reference_bool {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefBool(bool x) external returns (bool);
        function derefBoolRef(bool x) external returns (bool);
        function callDerefBoolRef(bool x) external returns (bool);
        function derefNestedBool(bool x) external returns (bool);
        function derefMutArg(bool x) external returns (bool);
        function writeMutRef(bool x) external returns (bool);
        function miscellaneous0() external returns (bool[]);
        function miscellaneous1() external returns (bool[]);
        function identityBoolRef(bool x) external returns (bool);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "bool";
        const SOURCE_PATH: &str = "tests/references/bool.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefBoolCall::new((true,)), true)]
    #[case(derefBoolRefCall::new((false,)), false)]
    #[case(callDerefBoolRefCall::new((true,)), true)]
    #[case(derefNestedBoolCall::new((false,)), false)]
    #[case(derefMutArgCall::new((true,)), true)]
    #[case(writeMutRefCall::new((false,)), true)]
    #[case(identityBoolRefCall::new((true,)), true)]
    fn test_bool_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        let expected_result = <sol!((bool,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![false, true, false])]
    #[case(miscellaneous1Call::new(()), vec![true, true, false])]
    fn test_bool_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<bool>,
    ) {
        let expected_result = <sol!(bool[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_uint_8 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefU8(uint8 x) external returns (uint8);
        function derefU8Ref(uint8 x) external returns (uint8);
        function callDerefU8Ref(uint8 x) external returns (uint8);
        function derefNestedU8(uint8 x) external returns (uint8);
        function derefMutArg(uint8 x) external returns (uint8);
        function writeMutRef(uint8 x) external returns (uint8);
        function miscellaneous0() external returns (uint8[]);
        function miscellaneous1() external returns (uint8[]);
        function freezeRef(uint8 x) external returns (uint8);
        function identityU8Ref(uint8 x) external returns (uint8);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_8";
        const SOURCE_PATH: &str = "tests/references/uint_8.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefU8Call::new((250,)), 250)]
    #[case(derefU8RefCall::new((u8::MAX,)), u8::MAX)]
    #[case(callDerefU8RefCall::new((1,)), 1)]
    #[case(derefNestedU8Call::new((7,)), 7)]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    #[case(freezeRefCall::new((3,)), 3)]
    #[case(identityU8RefCall::new((4,)), 4)]
    fn test_uint_8_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![1u8, 2u8, 3u8])]
    #[case(miscellaneous1Call::new(()), vec![1u8, 2u8, 3u8])]
    fn test_uint_8_mut_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u8>,
    ) {
        let expected_result = <sol!(uint8[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_uint_16 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefU16(uint16 x) external returns (uint16);
        function derefU16Ref(uint16 x) external returns (uint16);
        function callDerefU16Ref(uint16 x) external returns (uint16);
        function derefNestedU16(uint16 x) external returns (uint16);
        function derefMutArg(uint16 x) external returns (uint16);
        function writeMutRef(uint16 x) external returns (uint16);
        function miscellaneous0() external returns (uint16[]);
        function miscellaneous1() external returns (uint16[]);
        function freezeRef(uint16 x) external returns (uint16);
        function identityU16Ref(uint16 x) external returns (uint16);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_16";
        const SOURCE_PATH: &str = "tests/references/uint_16.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefU16Call::new((250,)), 250)]
    #[case(derefU16RefCall::new((u16::MAX,)), u16::MAX)]
    #[case(callDerefU16RefCall::new((1,)), 1)]
    #[case(derefNestedU16Call::new((7,)), 7)]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    #[case(freezeRefCall::new((3,)), 3)]
    #[case(identityU16RefCall::new((4,)), 4)]
    fn test_uint_16_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![1u16, 2u16, 3u16])]
    #[case(miscellaneous1Call::new(()), vec![1u16, 2u16, 3u16])]
    fn test_uint_16_mut_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u16>,
    ) {
        let expected_result = <sol!(uint16[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_uint_32 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefU32(uint32 x) external returns (uint32);
        function derefU32Ref(uint32 x) external returns (uint32);
        function callDerefU32Ref(uint32 x) external returns (uint32);
        function derefNestedU32(uint32 x) external returns (uint32);
        function derefMutArg(uint32 x) external returns (uint32);
        function writeMutRef(uint32 x) external returns (uint32);
        function miscellaneous0() external returns (uint32[]);
        function miscellaneous1() external returns (uint32[]);
        function freezeRef(uint32 x) external returns (uint32[]);
        function identityU32Ref(uint32 x) external returns (uint32);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_32";
        const SOURCE_PATH: &str = "tests/references/uint_32.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefU32Call::new((250,)), 250)]
    #[case(derefU32RefCall::new((u32::MAX,)), u32::MAX)]
    #[case(callDerefU32RefCall::new((1,)), 1)]
    #[case(derefNestedU32Call::new((7,)), 7)]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    #[case(freezeRefCall::new((3,)), 3)]
    #[case(identityU32RefCall::new((4,)), 4)]
    fn test_uint_32_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![1u32, 2u32, 3u32])]
    #[case(miscellaneous1Call::new(()), vec![1u32, 2u32, 3u32])]
    fn test_uint_32_mut_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u32>,
    ) {
        let expected_result = <sol!(uint32[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_uint_64 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefU64(uint64 x) external returns (uint64);
        function derefU64Ref(uint64 x) external returns (uint64);
        function callDerefU64Ref(uint64 x) external returns (uint64);
        function derefNestedU64(uint64 x) external returns (uint64);
        function derefMutArg(uint64 x) external returns (uint64);
        function writeMutRef(uint64 x) external returns (uint64);
        function miscellaneous0() external returns (uint64[]);
        function miscellaneous1() external returns (uint64[]);
        function freezeRef(uint64 x) external returns (uint64[]);
        function identityU64Ref(uint64 x) external returns (uint64);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_64";
        const SOURCE_PATH: &str = "tests/references/uint_64.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefU64Call::new((250,)), 250)]
    #[case(derefU64RefCall::new((u64::MAX,)), u64::MAX)]
    #[case(callDerefU64RefCall::new((1,)), 1)]
    #[case(derefNestedU64Call::new((7,)), 7)]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    #[case(freezeRefCall::new((3,)), 3)]
    #[case(identityU64RefCall::new((4,)), 4)]
    fn test_uint_64_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![1u64, 2u64, 3u64])]
    #[case(miscellaneous1Call::new(()), vec![1u64, 2u64, 3u64])]
    fn test_uint_64_mut_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u64>,
    ) {
        let expected_result = <sol!(uint64[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_uint_128 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefU128(uint128 x) external returns (uint128);
        function derefU128Ref(uint128 x) external returns (uint128);
        function callDerefU128Ref(uint128 x) external returns (uint128);
        function derefNestedU128(uint128 x) external returns (uint128);
        function derefMutArg(uint128 x) external returns (uint128);
        function writeMutRef(uint128 x) external returns (uint128);
        function miscellaneous0() external returns (uint128[]);
        function miscellaneous1() external returns (uint128[]);
        function freezeRef(uint128 x) external returns (uint128[]);
        function identityU128Ref(uint128 x) external returns (uint128);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_128";
        const SOURCE_PATH: &str = "tests/references/uint_128.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefU128Call::new((250,)), 250)]
    #[case(derefU128RefCall::new((u128::MAX,)), u128::MAX)]
    #[case(callDerefU128RefCall::new((1,)), 1)]
    #[case(derefNestedU128Call::new((7,)), 7)]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    #[case(freezeRefCall::new((3,)), 3)]
    #[case(identityU128RefCall::new((4,)), 4)]
    fn test_uint_128_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![1u128, 2u128, 3u128])]
    #[case(miscellaneous1Call::new(()), vec![1u128, 2u128, 3u128])]
    fn test_uint_128_mut_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u128>,
    ) {
        let expected_result = <sol!(uint128[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_uint_256 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function derefU256(uint256 x) external returns (uint256);
        function derefU256Ref(uint256 x) external returns (uint256);
        function callDerefU256Ref(uint256 x) external returns (uint256);
        function derefNestedU256(uint256 x) external returns (uint256);
        function derefMutArg(uint256 x) external returns (uint256);
        function writeMutRef(uint256 x) external returns (uint256);
        function miscellaneous0() external returns (uint256[]);
        function miscellaneous1() external returns (uint256[]);
        function freezeRef(uint256 x) external returns (uint256[]);
        function identityU256Ref(uint256 x) external returns (uint256);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_256";
        const SOURCE_PATH: &str = "tests/references/uint_256.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefU256Call::new((U256::from(250),)), U256::from(250))]
    #[case(derefU256RefCall::new((U256::from(1234567890),)), U256::from(1234567890))]
    #[case(callDerefU256RefCall::new((U256::from(1),)), U256::from(1))]
    #[case(derefNestedU256Call::new((U256::from(7),)), U256::from(7))]
    #[case(derefMutArgCall::new((U256::from(1),)), U256::from(1))]
    #[case(writeMutRefCall::new((U256::from(2),)), U256::from(1))]
    #[case(freezeRefCall::new((U256::from(3),)), U256::from(3))]
    #[case(identityU256RefCall::new((U256::from(4),)), U256::from(4))]
    fn test_uint_256_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![U256::from(1), U256::from(2), U256::from(3)])]
    #[case(miscellaneous1Call::new(()), vec![U256::from(1), U256::from(2), U256::from(3)])]
    fn test_uint_256_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<U256>,
    ) {
        let expected_result = <sol!(uint256[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_address {
    use super::*;
    use alloy_primitives::{Address, address};

    sol!(
        #[allow(missing_docs)]
        function derefAddress(address x) external returns (address);
        function derefAddressRef(address x) external returns (address);
        function callDerefAddressRef(address x) external returns (address);
        function derefNestedAddress(address x) external returns (address);
        function derefMutArg(address x) external returns (address);
        function writeMutRef(address x) external returns (address);
        function miscellaneous0() external returns (address[]);
        function miscellaneous1() external returns (address[]);
        function freezeRef(address x) external returns (address);
        function identityAddressRef(address x) external returns (address);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "ref_address";
        const SOURCE_PATH: &str = "tests/references/address.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefAddressCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
    #[case(callDerefAddressRefCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
    #[case(derefNestedAddressCall::new((address!("0x7890abcdef1234567890abcdef1234567890abcd"),)), address!("0x7890abcdef1234567890abcdef1234567890abcd"))]
    #[case(derefMutArgCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
    #[case(writeMutRefCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x0000000000000000000000000000000000000001"))]
    #[case(freezeRefCall::new((address!("0x0000000000000000000000000000000000000003"),)), address!("0x0000000000000000000000000000000000000003"))]
    #[case(identityAddressRefCall::new((address!("0x0000000000000000000000000000000000000004"),)), address!("0x0000000000000000000000000000000000000004"))]
    fn test_address_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Address,
    ) {
        let expected_result = <sol!((address,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(miscellaneous0Call::new(()), vec![address!("0x0000000000000000000000000000000000000001"), address!("0x0000000000000000000000000000000000000002"), address!("0x0000000000000000000000000000000000000002")])]
    #[case(miscellaneous1Call::new(()), vec![address!("0x0000000000000000000000000000000000000001"), address!("0x0000000000000000000000000000000000000003"), address!("0x0000000000000000000000000000000000000002")])]
    fn test_address_ref_misc<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<Address>,
    ) {
        let expected_result = <sol!(address[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_signer {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function useDummy() external returns (address);  // Returns the signer
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "ref_signer";
        const SOURCE_PATH: &str = "tests/references/signer.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[should_panic]
    #[case(useDummyCall::new(()), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7])]
    fn test_signer_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: [u8; 20],
    ) {
        let expected_result = <sol!((address,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_vec_8 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function deref(uint8[] x) external returns (uint8[]);
        function derefArg(uint8[] x) external returns (uint8[]);
        function callDerefArg(uint8[] x) external returns (uint8[]);
        function vecFromElement(uint64 index) external returns (uint8[]);
        function getElementVector(uint64 index) external returns (uint8[]);
        function derefMutArg(uint8[] x) external returns (uint8[]);
        function writeMutRef(uint8[] x) external returns (uint8[]);
        function miscellaneous0() external returns (uint8[]);
        function miscellaneous1() external returns (uint8[]);
        function miscellaneous2() external returns (uint8[]);
        function miscellaneous3(uint8[] x) external returns (uint8[]);
        function miscellaneous4() external returns (uint8[]);
        function miscellaneous5() external returns (uint8[]);
        function freezeRef(uint8[] x) external returns (uint8[]);
        function identityVecRef(uint8[] x) external returns (uint8[]);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "vec_8";
        const SOURCE_PATH: &str = "tests/references/vec_8.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(derefArgCall::new((vec![4, 5, 6],)), vec![4, 5, 6])]
    #[case(callDerefArgCall::new((vec![7, 8, 9],)), vec![7, 8, 9])]
    #[case(vecFromElementCall::new((0,)), vec![10])]
    #[case(getElementVectorCall::new((0,)), vec![10, 20])]
    #[case(derefMutArgCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(writeMutRefCall::new((vec![4, 5, 6],)), vec![1, 2, 3])]
    #[case(freezeRefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(miscellaneous0Call::new(()), vec![4, 5, 4])]
    #[case(miscellaneous1Call::new(()), vec![20, 40])]
    #[case(miscellaneous2Call::new(()), vec![1, 4, 7])]
    #[case(miscellaneous3Call::new((vec![1, 2, 3],)), vec![99, 1, 3])]
    #[case(miscellaneous4Call::new(()), vec![1, 12, 111, 12, 11, 112])]
    #[case(miscellaneous5Call::new(()), vec![1, 12, 112, 11, 112, 113, 112])]
    #[case(identityVecRefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    fn test_vec_8_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u8>,
    ) {
        let expected_result = <sol!(uint8[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(getElementVectorCall::new((2,)))]
    #[case(getElementVectorCall::new((u64::MAX,)))]
    fn test_vec_8_out_of_bounds<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}
mod reference_vec_64 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function deref(uint64[] x) external returns (uint64[]);
        function derefArg(uint64[] x) external returns (uint64[]);
        function callDerefArg(uint64[] x) external returns (uint64[]);
        function vecFromElement(uint64 index) external returns (uint64[]);
        function getElementVector(uint64 index) external returns (uint64[]);
        function derefMutArg(uint64[] x) external returns (uint64[]);
        function writeMutRef(uint64[] x) external returns (uint64[]);
        function miscellaneous0() external returns (uint64[]);
        function miscellaneous1() external returns (uint64[]);
        function miscellaneous2() external returns (uint64[]);
        function miscellaneous3(uint64[] x) external returns (uint64[]);
        function miscellaneous4() external returns (uint64[]);
        function freezeRef(uint64[] x) external returns (uint64[]);
        function identityVecRef(uint64[] x) external returns (uint64[]);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "vec_64";
        const SOURCE_PATH: &str = "tests/references/vec_64.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(derefArgCall::new((vec![4, 5, 6],)), vec![4, 5, 6])]
    #[case(callDerefArgCall::new((vec![7, 8, 9],)), vec![7, 8, 9])]
    #[case(vecFromElementCall::new((0,)), vec![10])]
    #[case(getElementVectorCall::new((0,)), vec![10, 20])]
    #[case(derefMutArgCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(writeMutRefCall::new((vec![4, 5, 6],)), vec![1, 2, 3])]
    #[case(freezeRefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(miscellaneous0Call::new(()), vec![4, 5, 4])]
    #[case(miscellaneous1Call::new(()), vec![20, 40])]
    #[case(miscellaneous2Call::new(()), vec![1, 4, 7])]
    #[case(miscellaneous3Call::new((vec![1, 2, 3],)), vec![99, 1, 3])]
    #[case(miscellaneous4Call::new(()), vec![1, 12, 111, 12, 11, 112])]
    #[case(identityVecRefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    fn test_vec_64_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u64>,
    ) {
        let expected_result = <sol!(uint64[])>::abi_encode(&expected_result);
        println!("expected_result: {:?}", expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
mod reference_vec_256 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function deref(uint256[] x) external returns (uint256[]);
        function derefArg(uint256[] x) external returns (uint256[]);
        function callDerefArg(uint256[] x) external returns (uint256[]);
        function vecFromElement(uint64 index) external returns (uint256[]);
        function getElementVector(uint64 index) external returns (uint256[]);
        function derefMutArg(uint256[] x) external returns (uint256[]);
        function writeMutRef(uint256[] x) external returns (uint256[]);
        function miscellaneous0() external returns (uint256[]);
        function miscellaneous1() external returns (uint256[]);
        function miscellaneous2() external returns (uint256[]);
        function miscellaneous3(uint256[] x) external returns (uint256[]);
        function miscellaneous4() external returns (uint256[]);
        function freezeRef(uint256[] x) external returns (uint256[]);
        function identityVecRef(uint256[] x) external returns (uint256[]);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "vec_256";
        const SOURCE_PATH: &str = "tests/references/vec_256.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    #[case(derefArgCall::new((vec![U256::from(4), U256::from(5), U256::from(6)],)), vec![U256::from(4), U256::from(5), U256::from(6)])]
    #[case(callDerefArgCall::new((vec![U256::from(7), U256::from(8), U256::from(9)],)), vec![U256::from(7), U256::from(8), U256::from(9)])]
    #[case(vecFromElementCall::new((0,)), vec![U256::from(10)])]
    #[case(getElementVectorCall::new((0,)), vec![U256::from(10), U256::from(20)])]
    #[case(derefMutArgCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    #[case(writeMutRefCall::new((vec![U256::from(4), U256::from(5), U256::from(6)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    #[case(freezeRefCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    #[case(miscellaneous0Call::new(()), vec![U256::from(4), U256::from(5), U256::from(4)])]
    #[case(miscellaneous1Call::new(()), vec![U256::from(20), U256::from(40)])]
    #[case(miscellaneous2Call::new(()), vec![U256::from(1), U256::from(4), U256::from(7)])]
    #[case(miscellaneous3Call::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(99), U256::from(1), U256::from(3)])]
    #[case(miscellaneous4Call::new(()), vec![U256::from(1), U256::from(12), U256::from(111), U256::from(12), U256::from(11), U256::from(112)])]
    #[case(identityVecRefCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    fn test_vec_256_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<U256>,
    ) {
        let expected_result = <sol!(uint256[])>::abi_encode(&expected_result);
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(getElementVectorCall::new((2,)))]
    #[case(getElementVectorCall::new((u64::MAX,)))]
    fn test_vec_256_out_of_bounds<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}

mod reference_structs {
    use alloy_primitives::address;
    use alloy_sol_types::SolValue;

    use super::*;

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

        function derefStruct(Foo x) external returns (Foo);
        function derefStructRef(Foo y) external returns (Foo);
        function callDerefStructRef(Foo x) external returns (Foo);
        function derefNestedStruct(Foo x) external returns (Foo);
        function derefMutArg(Foo x) external returns (Foo);
        function writeMutRef(Foo x) external returns (Foo);
        function writeMutRef2(Foo x) external returns (Foo);
        function freezeRef(Foo x) external returns (Foo);
        function identityStructRef(Foo x) external returns (Foo);
        function identityStaticStructRef(Bar x) external returns (Bar);
    );

    fn get_foo() -> Foo {
        Foo {
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bar: Bar {
                a: u16::MAX - 1,
                b: u128::MAX,
            },
            baz: Baz {
                a: 42,
                b: vec![U256::MAX],
            },
        }
    }

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "structs";
        const SOURCE_PATH: &str = "tests/references/structs.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefStructCall::new((get_foo(),)),get_foo())]
    #[case(derefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(callDerefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(derefNestedStructCall::new((get_foo(),)),get_foo())]
    #[case(derefMutArgCall::new((get_foo(),)),get_foo())]
    #[case(freezeRefCall::new((get_foo(),)),get_foo())]
    #[case(writeMutRefCall::new(
        (get_foo(),)),
        Foo {
            q: address!("0x00000000000000000000000000000000deadbeef"),
            r: vec![0, 3, 0, 3, 4, 5, 6],
            s: vec![6, 5, 4, 3, 0, 3, 0],
            t: false,
            u: 42,
            v: 4242,
            w: 424242,
            x: 42424242,
            y: 4242424242,
            z: U256::from(424242424242_u128),
            bar: Bar {
                a: 42,
                b: 4242
            },
            baz: Baz {
                a: 4242,
                b: vec![
                    U256::from(3),
                ]
            },
        }
    )]
    #[case(writeMutRef2Call::new(
        (get_foo(),)),
        Foo {
            q: address!("0x00000000000000000000000000000000deadbeef"),
            r: vec![0, 3, 0, 3, 4, 5, 6],
            s: vec![6, 5, 4, 3, 0, 3, 0],
            t: false,
            u: 42,
            v: 4242,
            w: 424242,
            x: 42424242,
            y: 4242424242,
            z: U256::from(424242424242_u128),
            bar: Bar {
                a: 42,
                b: 4242
            },
            baz: Baz {
                a: 4242,
                b: vec![
                    U256::from(3),
                ]
            },
        }
    )]
    #[case(identityStructRefCall::new((get_foo(),)),get_foo())]
    fn test_struct_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Foo,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(identityStaticStructRefCall::new((Bar { a: 42, b: 4242 },)), Bar { a: 42, b: 4242 })]
    fn test_static_struct_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Bar,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod reference_structs_generic {
    use alloy_primitives::address;
    use alloy_sol_types::SolValue;

    use super::*;

    sol!(
        #[allow(missing_docs)]
        struct Foo {
            uint16 g;
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
            uint16 g;
            uint16 a;
            uint128 b;
        }

        struct Baz {
            uint16 a;
            uint256[] b;
        }

        function derefStruct(Foo x) external returns (Foo);
        function derefStructRef(Foo y) external returns (Foo);
        function callDerefStructRef(Foo x) external returns (Foo);
        function derefNestedStruct(Foo x) external returns (Foo);
        function derefMutArg(Foo x) external returns (Foo);
        function writeMutRef(Foo x) external returns (Foo);
        function writeMutRef2(Foo x) external returns (Foo);
        function freezeRef(Foo x) external returns (Foo);
        function identityStructRef(Foo x) external returns (Foo);
        function identityStaticStructRef(Bar x) external returns (Bar);
    );

    fn get_foo() -> Foo {
        Foo {
            g: 111,
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bar: Bar {
                g: 222,
                a: u16::MAX - 1,
                b: u128::MAX,
            },
            baz: Baz {
                a: 42,
                b: vec![U256::MAX],
            },
        }
    }

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "structs_generic";
        const SOURCE_PATH: &str = "tests/references/structs_generic.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(derefStructCall::new((get_foo(),)),get_foo())]
    #[case(derefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(callDerefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(derefNestedStructCall::new((get_foo(),)),get_foo())]
    #[case(derefMutArgCall::new((get_foo(),)),get_foo())]
    #[case(freezeRefCall::new((get_foo(),)),get_foo())]
    #[case(writeMutRefCall::new(
        (get_foo(),)),
        Foo {
            g: 111,
            q: address!("0x00000000000000000000000000000000deadbeef"),
            r: vec![0, 3, 0, 3, 4, 5, 6],
            s: vec![6, 5, 4, 3, 0, 3, 0],
            t: false,
            u: 42,
            v: 4242,
            w: 424242,
            x: 42424242,
            y: 4242424242,
            z: U256::from(424242424242_u128),
            bar: Bar {
                g: 222,
                a: 42,
                b: 4242
            },
            baz: Baz {
                a: 4242,
                b: vec![
                    U256::from(3),
                ]
            },
        }
    )]
    #[case(writeMutRef2Call::new(
        (get_foo(),)),
        Foo {
            g: 111,
            q: address!("0x00000000000000000000000000000000deadbeef"),
            r: vec![0, 3, 0, 3, 4, 5, 6],
            s: vec![6, 5, 4, 3, 0, 3, 0],
            t: false,
            u: 42,
            v: 4242,
            w: 424242,
            x: 42424242,
            y: 4242424242,
            z: U256::from(424242424242_u128),
            bar: Bar {
                g: 222,
                a: 42,
                b: 4242
            },
            baz: Baz {
                a: 4242,
                b: vec![
                    U256::from(3),
                ]
            },
        }
    )]
    #[case(identityStructRefCall::new((get_foo(),)),get_foo())]
    fn test_struct_generic_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Foo,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(identityStaticStructRefCall::new((Bar { g: 222, a: 42, b: 4242 },)), Bar { g: 222, a: 42, b: 4242 })]
    fn test_static_struct_generic_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Bar,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod external_struct {
    use alloy_primitives::address;
    use alloy_sol_types::SolValue;

    use crate::common::translate_test_complete_package;

    use super::*;

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

        function derefStruct(Foo x) external returns (Foo);
        function derefStructRef(Foo y) external returns (Foo);
        function callDerefStructRef(Foo x) external returns (Foo);
        function derefNestedStruct(Foo x) external returns (Foo);
        function derefMutArg(Foo x) external returns (Foo);
        function freezeRef(Foo x) external returns (Foo);
        function writeRef(Foo x, Foo y) external returns (Foo);
        function identityStructRef(Foo x) external returns (Foo);
        function identityStaticStructRef(Bar x) external returns (Bar);
    );

    fn get_foo() -> Foo {
        Foo {
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bar: Bar {
                a: u16::MAX - 1,
                b: u128::MAX,
            },
            baz: Baz {
                a: 42,
                b: vec![U256::MAX],
            },
        }
    }

    fn get_foo2() -> Foo {
        Foo {
            q: address!("0xcafe00000000000000000000000000000000cafe"),
            r: vec![1, 2],
            s: vec![1, 2],
            t: false,
            u: 1,
            v: 2,
            w: 3,
            x: 4,
            y: 5,
            z: U256::from(6),
            bar: Bar { a: 7, b: 8 },
            baz: Baz {
                a: 9,
                b: vec![U256::from(10)],
            },
        }
    }

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_packages = translate_test_complete_package("tests/references/external");

        let translated_package = translated_packages.get_mut("external_struct").unwrap();
        RuntimeSandbox::new(translated_package)
    }

    #[rstest]
    #[case(derefStructCall::new((get_foo(),)),get_foo())]
    #[case(derefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(callDerefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(derefNestedStructCall::new((get_foo(),)),get_foo())]
    #[case(derefMutArgCall::new((get_foo(),)),get_foo())]
    #[case(freezeRefCall::new((get_foo(),)),get_foo())]
    #[case(writeRefCall::new((get_foo(),get_foo2())),get_foo2())]
    #[case(identityStructRefCall::new((get_foo(),)),get_foo())]
    fn test_externalstruct_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Foo,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(identityStaticStructRefCall::new((Bar { a: 42, b: 4242 },)), Bar { a: 42, b: 4242 })]
    fn test_external_struct_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Bar,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod external_generic_struct {
    use alloy_primitives::address;
    use alloy_sol_types::SolValue;

    use crate::common::translate_test_complete_package;

    use super::*;

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

        function derefStruct(Foo x) external returns (Foo);
        function derefStructRef(Foo y) external returns (Foo);
        function callDerefStructRef(Foo x) external returns (Foo);
        function derefNestedStruct(Foo x) external returns (Foo);
        function derefMutArg(Foo x) external returns (Foo);
        function freezeRef(Foo x) external returns (Foo);
        function writeRef(Foo x, Foo y) external returns (Foo);
        function identityStructRef(Foo x) external returns (Foo);
        function identityStaticStructRef(Bar x) external returns (Bar);
    );

    fn get_foo() -> Foo {
        Foo {
            g: 314,
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bar: Bar {
                g: 314,
                a: u16::MAX - 1,
                b: u128::MAX,
            },
            baz: Baz {
                g: 314,
                a: 42,
                b: vec![U256::MAX],
            },
        }
    }

    fn get_foo2() -> Foo {
        Foo {
            g: 31415,
            q: address!("0xcafe00000000000000000000000000000000cafe"),
            r: vec![1, 2],
            s: vec![1, 2],
            t: false,
            u: 1,
            v: 2,
            w: 3,
            x: 4,
            y: 5,
            z: U256::from(6),
            bar: Bar {
                g: 31415,
                a: 7,
                b: 8,
            },
            baz: Baz {
                g: 31415,
                a: 9,
                b: vec![U256::from(10)],
            },
        }
    }

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_packages = translate_test_complete_package("tests/references/external");

        let translated_package = translated_packages
            .get_mut("external_generic_struct")
            .unwrap();
        RuntimeSandbox::new(translated_package)
    }

    #[rstest]
    #[case(derefStructCall::new((get_foo(),)),get_foo())]
    #[case(derefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(callDerefStructRefCall::new((get_foo(),)),get_foo())]
    #[case(derefNestedStructCall::new((get_foo(),)),get_foo())]
    #[case(derefMutArgCall::new((get_foo(),)),get_foo())]
    #[case(freezeRefCall::new((get_foo(),)),get_foo())]
    #[case(writeRefCall::new((get_foo(),get_foo2())),get_foo2())]
    #[case(identityStructRefCall::new((get_foo(),)),get_foo())]
    fn test_external_generic_struct_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Foo,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(identityStaticStructRefCall::new((Bar { g: 314, a: 42, b: 4242 },)), Bar { g: 314, a: 42, b: 4242 })]
    fn test_external_generic_struct_ref_id<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Bar,
    ) {
        let expected_result = expected_result.abi_encode();
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}

mod reference_arguments {
    use alloy_primitives::{U256, address};

    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "reference_args";
        const SOURCE_PATH: &str = "tests/references/arguments.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        struct Bar {
            uint32 a;
            uint128 b;
        }

        struct Foo {
            Bar c;
            address d;
            uint128[] e;
            bool f;
            uint16 g;
            uint256 h;
        }

        function testForward(uint32 x, bool inner) external returns (bool, uint32);
        function test(uint32 x, bool inner) external returns (bool, uint32);
        function testInv(bool inner, uint32 x) external returns (bool, uint32);
        function testMix(uint32 x, bool inner, uint64 v, uint64 w) external returns (bool, uint32, uint64, uint64);
        function testForwardGenerics(uint32 x, bool inner, uint64 y) external returns (bool, uint64, uint32);
        function testForwardGenerics2(Bar x, uint128 b, Foo y) external returns (uint128, Foo, Bar);
    }

    #[rstest]
    #[case(testForwardCall::new((
        55, true)),
        (true, 55))]
    #[case(testForwardCall::new((
        55, false)),
        (false, 55))]
    #[case(testCall::new((
        55, true)),
        (true, 55))]
    #[case(testInvCall::new((
        true, 55)),
        (true, 55))]
    #[case(testMixCall::new((
        55, true, 66, 77)),
        (true, 55, 66, 77))]
    #[case(testForwardGenericsCall::new((
        55, true, 66)),
        (true, 66, 55))]
    #[case(testForwardGenericsCall::new((
        55, false, 66)),
        (false, 66, 55))]
    #[case(testForwardGenerics2Call::new((
        Bar { a: 55, b: 66 },
        77,
        Foo {
            c: Bar { a: 88, b: 99 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![77],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        },
    )),
        (77, Foo {
            c: Bar { a: 88, b: 99 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![77],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        },
        Bar { a: 55, b: 66 }
    ,))]
    fn test_generic_args<T: SolCall, V: SolValue>(
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
