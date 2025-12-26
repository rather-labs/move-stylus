use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "generic_struct_abi_packing_unpacking",
    "tests/structs/move_sources/generic_struct_abi_packing_unpacking.move"
);

sol! {
    struct Baz {
        uint16 a;
        uint128 b;
    }

    struct Foo {
        uint32 g;
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
        uint32[] g;
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
        uint32,
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
        uint32[],
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
        uint32 g,
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
        uint32[] g,
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
            g: 424242,
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
            424242,
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
            g: vec![4242, 424242],
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
            vec![4242, 424242],
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
#[case(packUnpackBetweenValsStaticCall::new(
        (
            true,
            Foo {
                g: 424242,
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
                g: 424242,
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
                g: vec![4242, 424242],
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
                g: vec![4242, 424242],
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
fn test_generic_struct_abi_unpacking<T: SolCall, V: SolValue>(
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
#[case(echoFooPackCall::new(
        (
            424242,
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
            g: 424242,
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
            vec![4242, 424242],
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
            g: vec![4242, 424242],
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
            g: 424242,
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
            g: 424242,
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
            g: vec![4242, 424242],
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
            g: vec![4242, 424242],
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
fn test_generic_struct_abi_packing<T: SolCall, V: SolValue>(
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
