use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_external_generic_struct",
    "tests/references/move_sources/external"
);

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
