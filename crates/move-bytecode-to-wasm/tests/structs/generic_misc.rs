use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "generic_struct_misc",
    "tests/structs/move_sources/generic_struct_misc.move"
);

sol! {
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

    struct Foo2 {
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
        Bar2 bar;
        Baz2 baz;
    }

    struct Bar2 {
        uint32[] g;
        uint16 a;
        uint128 b;
    }

    struct Baz2 {
        uint32[] g;
        uint16 a;
        uint256[] b;
    }

    struct Fu2 {
        uint32 a;
        uint32[] b;
    }

    struct GenericStruct {
        uint16 a;
        uint32 b;
        uint64 c;
    }

    struct ComplexGenericStruct {
        uint16 a;
        GenericStruct b;
        uint64[] c;
        uint64[][] d;
    }


    function createFooU32(uint32 g) external returns (Foo);
    function createFooVecU32(uint32[] g) external returns (Foo2);
    function createFuU32(uint32 t) external returns (Fu2);
    function createGenericStruct(uint16 a, uint32 b, uint64 c) external returns (GenericStruct);
    function createComplexGenericStruct(uint16 a, uint32 b, uint64 c) external returns (ComplexGenericStruct);

}

#[rstest]
#[case(
        createFooU32Call::new((314,)),
        Foo {
            g: 314,
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![0, 3, 0, 3, 4, 5, 6],
            s: vec![6, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 42,
            v: 4242,
            w: 424242,
            x: 42424242,
            y: 4242424242,
            z: U256::from(424242424242_u128),
            bar: Bar { g: 314, a: 42, b: 4242 },
            baz: Baz { g: 314, a: 4242, b: vec![U256::from(3)] },
        }
    )]
#[case(
        createFooVecU32Call::new((vec![u32::MAX],)),
        Foo2 {
            g: vec![u32::MAX],
            q: address!("0xcafe000000000000000000000000000000007357"),
            r: vec![0, 3, 0, 3, 4, 5, 6],
            s: vec![6, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 42,
            v: 4242,
            w: 424242,
            x: 42424242,
            y: 4242424242,
            z: U256::from(424242424242_u128),
            bar: Bar2 { g: vec![u32::MAX], a: 42, b: 4242 },
            baz: Baz2 { g: vec![u32::MAX], a: 4242, b: vec![U256::from(3)] },
        }
    )]
#[case(
        createFuU32Call::new((42,)),
        Fu2 { a: 42, b: vec![42, 42, 42] }
    )]
#[case(
        createGenericStructCall::new((42, 4242, 424242)),
        GenericStruct { a: 42, b: 4242, c: 424242 }
    )]
#[case(
        createComplexGenericStructCall::new((42, 43, 44)),
        ComplexGenericStruct { a: 42, b: GenericStruct { a: 42, b: 43, c: 44 }, c: vec![44, 44, 44], d: vec![vec![44], vec![44, 44]] }
    )]
fn test_generic_struct_misc<T: SolCall, V: SolValue>(
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
