use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "vec_external_struct",
    "tests/primitives/move_sources/external"
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

    function getLiteral() external returns (Foo[]);
    function getCopiedLocal() external returns (Foo[]);
    function echo(Foo[] x) external returns (Foo[]);
    function vecFromStruct(Foo x, Foo y) external returns (Foo[]);
    function vecFromVec(Foo[] x, Foo[] y) external returns (Foo[][]);
    function vecFromVecAndStruct(Foo[] x, Foo y) external returns (Foo [][]);
    function vecLen(Foo[] x) external returns (uint64);
    function vecPopBack(Foo[] x) external returns (Foo[]);
    function vecSwap(Foo[] x, uint64 id1, uint64 id2) external returns (Foo[]);
    function vecPushBack(Foo[] x, Foo y) external returns (Foo[]);
    function vecPushAndPopBack(Foo[] x, Foo y) external returns (Foo[]);
    function vecEq(Foo[] x, Foo[] y) external returns (bool);
    function vecNeq(Foo[] x, Foo[] y) external returns (bool);
    function vecBorrow(Foo[] x) external returns (Foo);
    function vecMutBorrow(Foo[] x) external returns (Foo);
);

fn get_foo_vector() -> Vec<Foo> {
    vec![
        Foo {
            q: address!("0x00000000000000000000000000000001deadbeef"),
            r: vec![1, 3, 0, 3, 4, 5, 6],
            s: vec![1, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 41,
            v: 14242,
            w: 1424242,
            x: 142424242,
            y: 14242424242,
            z: U256::from(1424242424242_u128),
            bar: Bar { a: 142, b: 14242 },
            baz: Baz {
                a: 14242,
                b: vec![U256::from(1)],
            },
        },
        Foo {
            q: address!("0x00000000000000000000000000000002deadbeef"),
            r: vec![2, 3, 0, 3, 4, 5, 6],
            s: vec![2, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 42,
            v: 24242,
            w: 2424242,
            x: 242424242,
            y: 24242424242,
            z: U256::from(2424242424242_u128),
            bar: Bar { a: 242, b: 24242 },
            baz: Baz {
                a: 24242,
                b: vec![U256::from(2)],
            },
        },
        Foo {
            q: address!("0x00000000000000000000000000000003deadbeef"),
            r: vec![3, 3, 0, 3, 4, 5, 6],
            s: vec![3, 5, 4, 3, 0, 3, 0],
            t: true,
            u: 43,
            v: 34242,
            w: 3424242,
            x: 342424242,
            y: 34242424242,
            z: U256::from(3424242424242_u128),
            bar: Bar { a: 342, b: 34242 },
            baz: Baz {
                a: 34242,
                b: vec![U256::from(3)],
            },
        },
    ]
}

fn get_new_fooo() -> Foo {
    Foo {
        q: address!("0x00000000000000000000000000000004deadbeef"),
        r: vec![4, 3, 0, 3, 4, 5, 6],
        s: vec![4, 5, 4, 3, 0, 3, 0],
        t: true,
        u: 44,
        v: 44242,
        w: 4424242,
        x: 442424242,
        y: 44242424242,
        z: U256::from(4424242424242_u128),
        bar: Bar { a: 442, b: 44242 },
        baz: Baz {
            a: 44242,
            b: vec![U256::from(4)],
        },
    }
}

#[rstest]
#[case(getLiteralCall::new(()), get_foo_vector())]
#[case(getCopiedLocalCall::new(()), get_foo_vector())]
#[case(echoCall::new((get_foo_vector(),)), get_foo_vector())]
#[case(
        vecFromStructCall::new((
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            }
        )),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
#[case(vecFromVecCall::new((get_foo_vector(), get_foo_vector())), vec![get_foo_vector(), get_foo_vector()])]
#[case(
        vecFromVecAndStructCall::new((
            get_foo_vector(),
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        )),
        vec![
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar { a: 142, b: 14242 },
                    baz: Baz {
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                },
                Foo {
                    q: address!("0x00000000000000000000000000000001deadbeef"),
                    r: vec![1, 3, 0, 3, 4, 5, 6],
                    s: vec![1, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 41,
                    v: 14242,
                    w: 1424242,
                    x: 142424242,
                    y: 14242424242,
                    z: U256::from(1424242424242_u128),
                    bar: Bar { a: 142, b: 14242 },
                    baz: Baz {
                        a: 14242,
                        b: vec![U256::from(1)],
                    },
                }
            ]
        ]
    )]
#[case(vecLenCall::new((get_foo_vector(),)), (3u64,))]
#[case(
        vecPopBackCall::new((get_foo_vector(),)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            }
        ]
    )]
#[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 1u64)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            }
        ]
    )]
#[case(
        vecSwapCall::new((get_foo_vector(), 0u64, 2u64)),
        vec![
            Foo {
                q: address!("0x00000000000000000000000000000003deadbeef"),
                r: vec![3, 3, 0, 3, 4, 5, 6],
                s: vec![3, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 43,
                v: 34242,
                w: 3424242,
                x: 342424242,
                y: 34242424242,
                z: U256::from(3424242424242_u128),
                bar: Bar { a: 342, b: 34242 },
                baz: Baz {
                    a: 34242,
                    b: vec![U256::from(3)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000002deadbeef"),
                r: vec![2, 3, 0, 3, 4, 5, 6],
                s: vec![2, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 42,
                v: 24242,
                w: 2424242,
                x: 242424242,
                y: 24242424242,
                z: U256::from(2424242424242_u128),
                bar: Bar { a: 242, b: 24242 },
                baz: Baz {
                    a: 24242,
                    b: vec![U256::from(2)],
                },
            },
            Foo {
                q: address!("0x00000000000000000000000000000001deadbeef"),
                r: vec![1, 3, 0, 3, 4, 5, 6],
                s: vec![1, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 41,
                v: 14242,
                w: 1424242,
                x: 142424242,
                y: 14242424242,
                z: U256::from(1424242424242_u128),
                bar: Bar { a: 142, b: 14242 },
                baz: Baz {
                    a: 14242,
                    b: vec![U256::from(1)],
                },
            },
        ])]
#[case(
        vecPushBackCall::new((
            get_foo_vector(),
            Foo {
                q: address!("0x00000000000000000000000000000004deadbeef"),
                r: vec![4, 3, 0, 3, 4, 5, 6],
                s: vec![4, 5, 4, 3, 0, 3, 0],
                t: true,
                u: 44,
                v: 44242,
                w: 4424242,
                x: 442424242,
                y: 44242424242,
                z: U256::from(4424242424242_u128),
                bar: Bar { a: 442, b: 44242 },
                baz: Baz {
                    a: 44242,
                    b: vec![U256::from(4)],
                },
            }
        )),
        [get_foo_vector(), vec![get_new_fooo(), get_new_fooo()]].concat()
    )]
#[case(vecPushAndPopBackCall::new((get_foo_vector(), get_new_fooo())), get_foo_vector())]
#[case(vecEqCall::new((get_foo_vector(), get_foo_vector())), (true,))]
#[case(
        vecEqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar { a: 442, b: 44242 },
                    baz: Baz {
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (false,)
    )]
#[case(vecNeqCall::new((get_foo_vector(), get_foo_vector())), (false,))]
#[case(
        vecNeqCall::new((
            get_foo_vector(),
            vec![
                Foo {
                    q: address!("0x00000000000000000000000000000004deadbeef"),
                    r: vec![4, 3, 0, 3, 4, 5, 6],
                    s: vec![4, 5, 4, 3, 0, 3, 0],
                    t: true,
                    u: 44,
                    v: 44242,
                    w: 4424242,
                    x: 442424242,
                    y: 44242424242,
                    z: U256::from(4424242424242_u128),
                    bar: Bar { a: 442, b: 44242 },
                    baz: Baz {
                        a: 44242,
                        b: vec![U256::from(4)],
                    },
                }
            ]
        )),
        (true,)
    )]
#[case(vecBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
#[case(vecMutBorrowCall::new((get_foo_vector(),)), get_foo_vector()[0].clone())]
fn test_vec_external_struct<T: SolCall, V: SolValue>(
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
#[case(vecPopBackCall::new((vec![],)), )]
#[case(vecSwapCall::new((get_foo_vector(), 0u64, 3u64)),)]
fn test_vec_external_struct_runtime_error<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(result, 1);
    let expected_data = RuntimeError::OutOfBounds.encode_abi();
    assert_eq!(return_data, expected_data);
}
