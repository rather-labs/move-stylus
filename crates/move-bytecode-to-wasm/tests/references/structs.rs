use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_structs",
    "tests/references/move_sources/structs.move"
);

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
