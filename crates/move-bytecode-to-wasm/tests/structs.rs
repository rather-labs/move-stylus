use alloy_primitives::{U256, address};
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

mod struct_fields {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "struct_fields";
        const SOURCE_PATH: &str = "tests/structs/struct_fields.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }
    sol!(
        #[allow(missing_docs)]
        function echoBool(bool a) external returns (bool);
        function echoU8(uint8 a) external returns (uint8);
        function echoU16(uint16 a) external returns (uint16);
        function echoU32(uint32 a) external returns (uint32);
        function echoU64(uint64 a) external returns (uint64);
        function echoU128(uint128 a) external returns (uint128);
        function echoU256(uint256 a) external returns (uint256);
        function echoVecStackType(uint32[] a) external returns (uint32[]);
        function echoVecHeapType(uint128[] a) external returns (uint128[]);
        function echoAddress(address a) external returns (address);
        function echoBarStructFields(uint32 a, uint128 b) external returns (uint32, uint128);
    );

    #[rstest]
    #[case(echoBoolCall::new((true,)), (true,))]
    #[case(echoBoolCall::new((false,)), (false,))]
    #[case(echoU8Call::new((255,)), (255,))]
    #[case(echoU8Call::new((1,)), (1,))]
    #[case(echoU16Call::new((u16::MAX,)), (u16::MAX,))]
    #[case(echoU16Call::new((1,)), (1,))]
    #[case(echoU32Call::new((u32::MAX,)), (u32::MAX,))]
    #[case(echoU32Call::new((1,)), (1,))]
    #[case(echoU64Call::new((u64::MAX,)), (u64::MAX,))]
    #[case(echoU64Call::new((1,)), (1,))]
    #[case(echoU128Call::new((u128::MAX,)), (u128::MAX,))]
    #[case(echoU128Call::new((1,)), (1,))]
    #[case(echoU256Call::new((U256::MAX,)), (U256::MAX,))]
    #[case(echoU256Call::new((U256::from(1),)), (U256::from(1),))]
    #[case(echoVecStackTypeCall::new((vec![1,2,u32::MAX,3,4],)), (vec![1,2,u32::MAX,3,4],))]
    #[case(echoVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4],)), (vec![1,2,u128::MAX,3,4],))]
    #[case(echoAddressCall::new(
    (address!("0xcafe000000000000000000000000000000007357"),)),
    (address!("0xcafe000000000000000000000000000000007357"),))
    ]
    #[case(echoBarStructFieldsCall::new((u32::MAX, u128::MAX)), (u32::MAX, u128::MAX),)]
    #[case(echoBarStructFieldsCall::new((1, u128::MAX)), (1, u128::MAX),)]
    #[case(echoBarStructFieldsCall::new((u32::MAX, 1)), (u32::MAX, 1),)]
    #[case(echoBarStructFieldsCall::new((1, 1)), (1, 1),)]
    fn test_struct_field_reference<T: SolCall, V: SolValue>(
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

mod struct_mut_fields {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "struct_mut_fields";
        const SOURCE_PATH: &str = "tests/structs/struct_mut_fields.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function echoMutBool(bool a) external returns (bool);
        function echoMutU8(uint8 a) external returns (uint8);
        function echoMutU16(uint16 a) external returns (uint16);
        function echoMutU32(uint32 a) external returns (uint32);
        function echoMutU64(uint64 a) external returns (uint64);
        function echoMutU128(uint128 a) external returns (uint128);
        function echoMutU256(uint256 a) external returns (uint256);
        function echoMutVecStackType(uint32[] a) external returns (uint32[]);
        function echoMutVecHeapType(uint128[] a) external returns (uint128[]);
        function echoMutAddress(address a) external returns (address);
        function echoBarStructFields(uint32 a, uint128 b) external returns (uint32, uint128);
    );

    #[rstest]
    #[case(echoMutBoolCall::new((true,)), (true,))]
    #[case(echoMutU8Call::new((255,)), (255,))]
    #[case(echoMutU8Call::new((1,)), (1,))]
    #[case(echoMutU16Call::new((u16::MAX,)), (u16::MAX,))]
    #[case(echoMutU16Call::new((1,)), (1,))]
    #[case(echoMutU32Call::new((u32::MAX,)), (u32::MAX,))]
    #[case(echoMutU32Call::new((1,)), (1,))]
    #[case(echoMutU64Call::new((u64::MAX,)), (u64::MAX,))]
    #[case(echoMutU64Call::new((1,)), (1,))]
    #[case(echoMutU128Call::new((u128::MAX,)), (u128::MAX,))]
    #[case(echoMutU128Call::new((1,)), (1,))]
    #[case(echoMutU256Call::new((U256::MAX,)), (U256::MAX,))]
    #[case(echoMutU256Call::new((U256::from(1),)), (U256::from(1),))]
    #[case(echoMutVecStackTypeCall::new((vec![1,2,u32::MAX,3,4],)), (vec![1,2,u32::MAX,3,4],))]
    #[case(echoMutVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4],)), (vec![1,2,u128::MAX,3,4],))]
    #[case(echoMutAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),)),
        (address!("0xcafe000000000000000000000000000000007357"),))
    ]
    #[case(echoBarStructFieldsCall::new((u32::MAX, u128::MAX)), (u32::MAX, u128::MAX),)]
    #[case(echoBarStructFieldsCall::new((1, u128::MAX)), (1, u128::MAX),)]
    #[case(echoBarStructFieldsCall::new((u32::MAX, 1)), (u32::MAX, 1),)]
    #[case(echoBarStructFieldsCall::new((1, 1)), (1, 1),)]
    fn test_struct_field_mut_reference<T: SolCall, V: SolValue>(
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

mod struct_unpacking {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "struct_unpacking";
        const SOURCE_PATH: &str = "tests/structs/struct_unpacking.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        struct Baz {
            uint16 a;
            uint128 b;
        }

        struct Foo {
            address q;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Baz baz;
        }

        struct Bazz {
            uint16 a;
            uint256[] b;
        }

        struct Bar {
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
            Bazz bazz;
        }

        function echoFoo(Foo foo) external returns (address, bool, uint8, uint16, uint32, uint64, uint128, uint256, uint16, uint128);
        function echoBar(Bar bar) external returns (address, uint32[], uint128[], bool, uint8, uint16, uint32, uint64, uint128, uint256, uint16, uint256[]);
    }

    #[rstest]
    #[case(echoFooCall::new(
        (Foo {
            q: address!("0xcafe000000000000000000000000000000007357"),
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            baz: Baz { a: 42, b: 4242}
        },)),
        (
            address!("0xcafe000000000000000000000000000000007357"),
            true,
            255,
            u16::MAX,
            u32::MAX,
            u64::MAX,
            u128::MAX,
            U256::MAX,
            42,
            4242,
        )
    )]
    #[case(echoBarCall::new(
        (Bar {
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
            bazz: Bazz {
                a: 42,
                b: vec![
                    U256::from(9),
                    U256::from(8),
                    U256::from(7),
                    U256::from(6)
                ]
            }
        },)),
        (
            address!("0xcafe000000000000000000000000000000007357"),
            vec![1, 2, u32::MAX],
            vec![1, 2, u128::MAX],
            true,
            255,
            u16::MAX,
            u32::MAX,
            u64::MAX,
            u128::MAX,
            U256::MAX,
            42,
            vec![
                U256::from(9),
                U256::from(8),
                U256::from(7),
                U256::from(6)
            ]
        )
    )]
    fn test_struct_unpacking<T: SolCall, V: SolValue>(
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

mod struct_packing {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "struct_packing";
        const SOURCE_PATH: &str = "tests/structs/struct_packing.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol! {
        struct Baz {
            uint16 a;
            uint128 b;
        }

        struct Foo {
            address q;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Baz baz;
        }

        struct Bazz {
            uint16 a;
            uint256[] b;
        }

        struct Bar {
            address q;
            uint32[] r;
            /*
            uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
            Bazz bazz;
            */
        }

        function echoFoo(address q, bool t, uint8 u, uint16 v, uint32 w, uint64 x, uint128 y, uint256 z, uint16 ba, uint128 bb) external returns (Foo);
        function echoBar(address q, uint32[] r/*, uint128[] s, bool t, uint8 u, uint16 v, uint32 w, uint64 x, uint128 y, uint256 z, uint16 ba, uint256[] bb*/) external returns (Bar bar);
    }

    #[rstest]
    #[case(echoFooCall::new(
        (
            address!("0xcafe000000000000000000000000000000007357"),
            true,
            255,
            u16::MAX,
            u32::MAX,
            u64::MAX,
            u128::MAX,
            U256::MAX,
            42,
            4242,
        )),
        Foo {
            q: address!("0xcafe000000000000000000000000000000007357"),
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            baz: Baz { a: 42, b: 4242}
        }
    )]
    #[case(echoBarCall::new(
        (
            address!("0xcafe000000000000000000000000000000007357"),
            vec![1, 2, u32::MAX],
            /*
            vec![1, 2, u128::MAX],
            true,
            255,
            u16::MAX,
            u32::MAX,
            u64::MAX,
            u128::MAX,
            U256::MAX,
            42,
            vec![
                U256::from(9),
                U256::from(8),
                U256::from(7),
                U256::from(6)
            ]
            */
        )),
        Bar {
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![1, 2, u32::MAX],
            /*
            s: vec![1, 2, u128::MAX],
            t: true,
            u: 255,
            v: u16::MAX,
            w: u32::MAX,
            x: u64::MAX,
            y: u128::MAX,
            z: U256::MAX,
            bazz: Bazz {
                a: 42,
                b: vec![
                    U256::from(9),
                    U256::from(8),
                    U256::from(7),
                    U256::from(6)
                ]
            }
            */
        }
    )]
    fn test_struct_packing<T: SolCall, V: SolValue>(
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
