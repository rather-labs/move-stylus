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

mod struct_packing_unpacking {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "struct_packing_unpacking";
        const SOURCE_PATH: &str = "tests/structs/struct_packing_unpacking.move";

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
            Baz baz;
        }

        function echoFooUnpack(Foo foo) external returns (
            address,
            bool,
            uint8,
            uint16,
            uint32,
            uint64,
            uint128,
            uint256,
            uint16,
            uint128
        );
        function echoBarUnpack(Bar bar) external returns (
            address,
            uint32[],
            uint128[],
            bool,
            uint8,
            uint16,
            uint32,
            uint64,
            uint128,
            uint256,
            uint16,
            uint256[],
            uint16,
            uint128
        );
        function echoFooPack(
            address q,
            bool t,
            uint8 u,
            uint16 v,
            uint32 w,
            uint64 x,
            uint128 y,
            uint256 z,
            uint16 ba,
            uint128 bb
        ) external returns (Foo);
        function echoBarPack(
            address q,
            uint32[] r,
            uint128[] s,
            bool t,
            uint8 u,
            uint16 v,
            uint32 w,
            uint64 x,
            uint128 y,
            uint256 z,
            uint16 ba,
            uint256[] bb,
            uint16 bba,
            uint128 bbb
        ) external returns (Bar bar);
        function packUnpackStatic(Foo foo) external returns (Foo);
        function packUnpackDynamic(Bar bar) external returns (Bar);
        function packUnpackBetweenValsStatic(
            bool v1,
            Foo foo,
            uint128[] v4
        ) external returns (bool, Foo, uint128[]);
        function packUnpackBetweenValsDynamic(
            bool v1,
            uint32[] v2,
            Bar foo,
            bool v3,
            uint128[] v4
        ) external returns (bool, Bar, uint128[]);
    }

    #[rstest]
    #[case(echoFooUnpackCall::new(
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
    #[case(echoBarUnpackCall::new(
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
                    U256::MAX,
                ]
            },
            baz: Baz { a: 111, b: 1111111111 }
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
                U256::MAX,
            ],
            111,
            1111111111,
        )
    )]
    #[rstest]
    #[case(echoFooPackCall::new(
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
    #[case(echoBarPackCall::new(
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
                U256::MAX,
                U256::from(8),
                U256::from(7),
                U256::from(6)
            ],
            111,
            1111111111,
        )),
        Bar {
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
                    U256::MAX,
                    U256::from(8),
                    U256::from(7),
                    U256::from(6)
                ]
            },
            baz: Baz { a: 111, b: 1111111111 },
        }
    )]
    #[case(packUnpackStaticCall::new(
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
    #[case(packUnpackDynamicCall::new(
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
                    U256::MAX,
                    U256::from(8),
                    U256::from(7),
                    U256::from(6)
                ]
            },
            baz: Baz { a: 111, b: 1111111111 },
        },)),
        Bar {
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
                    U256::MAX,
                    U256::from(8),
                    U256::from(7),
                    U256::from(6)
                ]
            },
            baz: Baz { a: 111, b: 1111111111 },
        }
    )]
    #[case(packUnpackBetweenValsStaticCall::new(
        (
            true,
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
            },
            vec![7,8,9,10,11],
        )),
        (
            true,
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
            },
            vec![7,8,9,10,11],
    ))]
    #[case(packUnpackBetweenValsDynamicCall::new(
        (
            true,
            vec![1,2,3,4,5],
            Bar {
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
                        U256::MAX,
                        U256::from(8),
                        U256::from(7),
                        U256::from(6)
                    ]
                },
                baz: Baz { a: 111, b: 1111111111 },
            },
            false,
            vec![7,8,9,10,11],
        )),
        (
            true,
            Bar {
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
                        U256::MAX,
                        U256::from(8),
                        U256::from(7),
                        U256::from(6)
                    ]
                },
                baz: Baz { a: 111, b: 1111111111 },
            },
            vec![7,8,9,10,11],
    ))]
    fn test_struct_packing_unpacking<T: SolCall, V: SolValue>(
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

mod struct_copy {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "struct_copy";
        const SOURCE_PATH: &str = "tests/structs/struct_copy.move";

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

        function structCopy(Foo foo) external returns (Foo,Foo);
        function structCopy2() external returns (Foo,Foo);
    );

    #[rstest]
    #[case(structCopyCall::new(
        (Foo {
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
                b: u128::MAX
            },
            baz: Baz {
                a: 42,
                b: vec![
                    U256::MAX,
                ]
            },
        },)),
        (
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
                b: u128::MAX
            },
            baz: Baz {
                a: 42,
                b: vec![
                    U256::MAX,
                ]
            },
        },
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
    ))]
    #[case(structCopy2Call::new(
        ()),
        (
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
        },
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
    ))]
    fn test_struct_copy<T: SolCall, V: SolValue>(
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
