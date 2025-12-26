use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "generic_struct_pack_unpack",
    "tests/structs/move_sources/generic_struct_pack_unpack.move"
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
        Baz baz,
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
        Bazz bazz,
        Baz baz,
    ) external returns (Bar bar);
    function echoFooUnpack(Foo foo) external returns (
        uint32,
        address,
        bool,
        uint8,
        uint16,
        uint32,
        uint64,
        (uint128,uint256),
        (uint16,uint128)
    );
    function echoFooUnpackIgnoreFields(Foo foo) external returns (
        uint32,
        bool,
        uint16,
        uint64,
        uint256,
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
        (uint16,uint256[]),
        (uint16,uint128)
    );
    function echoBarUnpackIgnoreFields(Bar bar) external returns (
        uint32[],
        uint32[],
        bool,
        uint16,
        uint64,
        uint256,
        (uint16,uint128)
    );
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
            Baz { a: 42, b: 4242},
        ),),
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
            Bazz {
                a: 42,
                b: vec![
                    U256::MAX,
                ]
            },
            Baz { a: 42, b: 4242},
        ),),
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
                ]
            },
            baz: Baz { a: 42, b: 4242}
        }
    )]
fn test_generic_struct_pack<T: SolCall, V: SolValue>(
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
            Baz { a: 42, b: 4242},
        )
    )]
#[case(echoFooUnpackIgnoreFieldsCall::new(
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
            true,
            u16::MAX,
            u64::MAX,
            U256::MAX,
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
            Bazz {
                a: 42,
                b: vec![
                    U256::MAX,
                ]
            },
            Baz { a: 111, b: 1111111111 }
        )
    )]
#[case(echoBarUnpackIgnoreFieldsCall::new(
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
            vec![1, 2, u32::MAX],
            true,
            u16::MAX,
            u64::MAX,
            U256::MAX,
            Baz { a: 111, b: 1111111111 }
        )
    )]
fn test_generic_struct_unpack<T: SolCall, V: SolValue>(
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
