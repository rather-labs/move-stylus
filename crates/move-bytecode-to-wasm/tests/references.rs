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
    anyhow::ensure!(return_data == expected_result, "return data mismatch");

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
    fn test_bool_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        let expected_result = <sol!((bool,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((true,)), true)]
    #[case(writeMutRefCall::new((false,)), true)]
    fn test_bool_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        let expected_result = <sol!((bool,))>::abi_encode_params(&(expected_result,));
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
        function mutBorrowLocal() external returns (uint8);
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
    fn test_uint_8_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    #[case(mutBorrowLocalCall::new(()), 2)]
    fn test_uint_8_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode_params(&(expected_result,));
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
    fn test_uint_16_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    fn test_uint_16_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode_params(&(expected_result,));
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
    fn test_uint_32_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    fn test_uint_32_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode_params(&(expected_result,));
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
    fn test_uint_64_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    fn test_uint_64_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode_params(&(expected_result,));
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
    fn test_uint_128_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((1,)), 1)]
    #[case(writeMutRefCall::new((2,)), 1)]
    fn test_uint_128_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode_params(&(expected_result,));
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
    fn test_uint_256_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((U256::from(1),)), U256::from(1))]
    #[case(writeMutRefCall::new((U256::from(2),)), U256::from(1))]
    fn test_uint_256_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode_params(&(expected_result,));
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
    fn test_address_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Address,
    ) {
        let expected_result = <sol!((address,))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
    #[case(writeMutRefCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x0000000000000000000000000000000000000001"))]
    fn test_address_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Address,
    ) {
        let expected_result = <sol!((address,))>::abi_encode_params(&(expected_result,));
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
    #[case(useDummyCall::new(()), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 3, 5, 7])]
    fn test_signer_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: [u8; 20],
    ) {
        let expected_result = <sol!((address,))>::abi_encode_params(&(expected_result,));
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
        function miscellaneous() external returns (uint8[]);
        function derefMutArg(uint8[] x) external returns (uint8[]);
        function writeMutRef(uint8[] x) external returns (uint8[]);
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
    #[case(miscellaneousCall::new(()), vec![20u8, 40u8])]
    fn test_vec_8_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u8>,
    ) {
        let expected_result = <sol!((uint8[],))>::abi_encode_params(&(expected_result,));
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

    #[rstest]
    #[case(derefMutArgCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(writeMutRefCall::new((vec![4, 5, 6],)), vec![1, 2, 3])]
    fn test_vec_8_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u8>,
    ) {
        let expected_result = <sol!((uint8[],))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
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
        function miscellaneous() external returns (uint64[]);
        function derefMutArg(uint64[] x) external returns (uint64[]);
        function writeMutRef(uint64[] x) external returns (uint64[]);
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
    #[case(miscellaneousCall::new(()), vec![20, 40])]
    fn test_vec_64_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u64>,
    ) {
        let expected_result = <sol!((uint64[],))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(derefMutArgCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
    #[case(writeMutRefCall::new((vec![4, 5, 6],)), vec![1, 2, 3])]
    fn test_vec_64_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<u64>,
    ) {
        let expected_result = <sol!((uint64[],))>::abi_encode_params(&(expected_result,));
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
        function miscellaneous() external returns (uint256[]);
        function derefMutArg(uint256[] x) external returns (uint256[]);
        function writeMutRef(uint256[] x) external returns (uint256[]);
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
    #[case(miscellaneousCall::new(()), vec![U256::from(20), U256::from(40)])]
    fn test_vec_256_immutable_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<U256>,
    ) {
        let expected_result = <sol!((uint256[],))>::abi_encode_params(&(expected_result,));
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

    #[rstest]
    #[case(derefMutArgCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    #[case(writeMutRefCall::new((vec![U256::from(4), U256::from(5), U256::from(6)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
    fn test_vec_256_mut_ref<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: Vec<U256>,
    ) {
        let expected_result = <sol!((uint256[],))>::abi_encode_params(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
