use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_primitives::address;
use alloy_sol_types::abi::TokenSeq;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "reference_args",
    "tests/references/move_sources/arguments.move"
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
