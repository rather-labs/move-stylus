use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "generic_functions",
    "tests/generic_functions/generic_functions.move"
);

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

    function echoU8(uint8 x) external returns (uint8);
    function echoU16(uint16 x) external returns (uint16);
    function echoU32(uint32 x) external returns (uint32);
    function echoU64(uint64 x) external returns (uint64);
    function echoU128(uint128 x) external returns (uint128);
    function echoU256(uint256 x) external returns (uint256);
    function echoAddress(address x) external returns (address);
    function echoVecU32(uint32[] x) external returns (uint32[]);
    function echoVecU128(uint128[] x) external returns (uint128[]);
    function echoStruct(Foo x) external returns (Foo);
    function echoU32U128(uint32 x, uint128) external returns (uint32, uint128);
    function echoAddressVecU128(address x, uint128[]) external returns (address, uint128[]);
    function echoStructVecU128(Foo x, uint128[]) external returns (Foo, uint128[]);
    function echoStructRef(bool inner, Foo x) external returns (Foo);
    function echoStructMutRef(bool inner, Foo x) external returns (Foo);
    function echoVecU128Ref(uint128[] x) external returns (uint128[]);
    function echoVecU128MutRef(uint128[] x) external returns (uint128[]);
}

#[rstest]
#[case(echoU8Call::new((u8::MAX,)), (u8::MAX as u32,))]
#[case(echoU16Call::new((u16::MAX,)), (u16::MAX,))]
#[case(echoU32Call::new((u32::MAX,)), (u32::MAX,))]
#[case(echoU64Call::new((u64::MAX,)), (u64::MAX,))]
#[case(echoU128Call::new((u128::MAX,)), (u128::MAX,))]
#[case(echoU256Call::new((U256::MAX,)), (U256::MAX,))]
#[case(echoAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),)),
        (address!("0xcafe000000000000000000000000000000007357"),))
    ]
#[case(echoVecU32Call::new((vec![1,2,3],)), (vec![1,2,3],))]
#[case(echoVecU128Call::new((vec![1,2,3],)), (vec![1,2,3],))]
#[case(echoU32U128Call::new((u32::MAX, u128::MAX)), (u32::MAX, u128::MAX))]
#[case(echoAddressVecU128Call::new((
        address!("0xcafe000000000000000000000000000000007357"),
        vec![1,2,3]
    )), (
        address!("0xcafe000000000000000000000000000000007357"),
        vec![1,2,3]
    ))]
#[case(echoStructCall::new((
        Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
    },)),
        (Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        }
    ,))]
#[case(echoStructVecU128Call::new((
        Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        },
        vec![1,2,3,4,5]
        )),
        (
            Foo {
                c: Bar { a: 1, b: 2 },
                d: address!("0xcafe000000000000000000000000000000007357"),
                e: vec![1,2,3],
                f: true,
                g: u16::MAX,
                h: U256::MAX,
            },
            vec![1,2,3,4,5]
    ))]
#[case(echoStructRefCall::new((
        true,
        Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
    })),
        (Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        }
    ,))]
#[case(echoStructRefCall::new((
        false,
        Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
    })),
        (Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        }
    ,))]
#[case(echoStructMutRefCall::new((true,
        Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
    })),
        (Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        }
    ,))]
#[case(echoStructMutRefCall::new((false,
        Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
    })),
        (Foo {
            c: Bar { a: 1, b: 2 },
            d: address!("0xcafe000000000000000000000000000000007357"),
            e: vec![1,2,3],
            f: true,
            g: u16::MAX,
            h: U256::MAX,
        }
    ,))]
#[case(echoVecU128RefCall::new((
        vec![1,2,3],
    )),
        (vec![1,2,3],))]
#[case(echoVecU128MutRefCall::new((
        vec![1,2,3],
    )),
        (vec![1,2,3],))]
fn test_generic_calls<T: SolCall, V: SolValue>(
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
