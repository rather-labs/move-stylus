use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "equality_vectors",
    "tests/operations_equality/move_sources/vectors.move"
);

sol!(
    #[allow(missing_docs)]
    function eqVecU8(uint8[], uint8[]) external returns (bool);
    function eqVecStackType(uint16[], uint16[]) external returns (bool);
    function eqVecHeapType(uint128[], uint128[]) external returns (bool);
    function eqVecHeapType2(address[], address[]) external returns (bool);
    function eqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
    function eqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
    function eqVecNestedHeapType2(address[][], address[][]) external returns (bool);
    function neqVecStackType(uint16[], uint16[]) external returns (bool);
    function neqVecHeapType(uint128[], uint128[]) external returns (bool);
    function neqVecHeapType2(address[], address[]) external returns (bool);
    function neqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
    function neqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
    function neqVecNestedHeapType2(address[][], address[][]) external returns (bool);
    function eqVecBar(uint32 n1, uint32 n2, uint32 n3, uint32 n4) external returns (bool);
);

#[rstest]
#[case(eqVecU8Call::new((
        vec![u8::MAX, u8::MAX, 0, 1, 2, 3, u8::MAX],
        vec![u8::MAX, u8::MAX, 0, 1, 2, 3, u8::MAX])),
        true
    )]
#[case(eqVecU8Call::new((
        vec![u8::MAX, u8::MAX, 0, 1, 2, 3, u8::MAX],
        vec![u8::MAX, u8::MAX, 9, 8, 7, 6, u8::MAX])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        true
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 9, 8, 7, 6, u16::MAX])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, 4])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        false
    )]
#[case(eqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],)),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        true
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 9, 8, 7, 6, u128::MAX])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, 4])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        false
    )]
#[case(eqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ],
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ])),
        true
    )]
#[case(eqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ],
        vec![
            address!("0xcafe0000000cafecafe000000000000000007357"),
            address!("0xdeadbeef0000000000000000000000000000cafe")
        ])),
        false
    )]
#[case(eqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ],
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
        ])),
        false
    )]
#[case(eqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
        ],
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ])),
        false
    )]
#[case(eqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        true
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 2], vec![2, 3, u16::MAX]])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, 4]])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        false
    )]
#[case(eqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        true
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![50], vec![61], vec![70]],
        vec![vec![50], vec![62], vec![70]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, 1], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, 4]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]])),
        false
    )]
#[case(eqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX - 1]])),
        false
    )]
#[case(eqVecNestedHeapType2Call::new((
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ],
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ])),
        true
    )]
#[case(eqVecNestedHeapType2Call::new((
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ],
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xcafe0000000cafecafecafecafe0000000007357"),
                address!("0xdeadbeef0002000000000000000000000000cafe"),
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ])),
        false
    )]
#[case(eqVecNestedHeapType2Call::new((
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ],
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xcafe0000000cafecafecafecafe0000000007357"),
                address!("0xdeadbeef0002000000000000000000000000cafe"),
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
            ],
        ])),
        false
    )]
#[case(eqVecBarCall::new((42, 43, 42, 43)), true)]
#[case(eqVecBarCall::new((42, 43, 42, 42)), false)]
#[case(eqVecBarCall::new((42, 43, 43, 43)), false)]
fn test_equality_vector<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: bool,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        <sol!((bool,))>::abi_encode(&(expected_result,)),
    )
    .unwrap();
}

#[rstest]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        false
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 9, 8, 7, 6, u16::MAX])),
        true
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, 4])),
        true
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX])),
        true
    )]
#[case(neqVecStackTypeCall::new((
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3, u16::MAX],
        vec![u16::MAX, u16::MAX, 0, 1, 2, 3],)),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        false
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 9, 8, 7, 6, u128::MAX])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, 4])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        true
    )]
#[case(neqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ],
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ])),
        false
    )]
#[case(neqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ],
        vec![
            address!("0xcafe0000000cafecafe000000000000000007357"),
            address!("0xdeadbeef0000000000000000000000000000cafe")
        ])),
        true
    )]
#[case(neqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ],
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
        ])),
        true
    )]
#[case(neqVecHeapType2Call::new((
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
        ],
        vec![
            address!("0xdeadbeef0000000000000000000000000000cafe"),
            address!("0xcafe000000000000000000000000000000007357")
        ])),
        true
    )]
#[case(neqVecHeapTypeCall::new((
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3, u128::MAX],
        vec![u128::MAX, u128::MAX, 0, 1, 2, 3])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        false
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 2], vec![2, 3, u16::MAX]])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, 4]])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]])),
        true
    )]
#[case(neqVecNestedStackTypeCall::new((
        vec![vec![u16::MAX, u16::MAX], vec![0, 1], vec![2, 3, u16::MAX]],
        vec![vec![u16::MAX, u16::MAX], vec![0, 1]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        false
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![50], vec![61], vec![70]],
        vec![vec![50], vec![62], vec![70]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, 1], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, 4]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1]])),
        true
    )]
#[case(neqVecNestedHeapTypeCall::new((
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX]],
        vec![vec![u128::MAX, u128::MAX], vec![0, 1], vec![2, 3, u128::MAX - 1]])),
        true
    )]
#[case(neqVecNestedHeapType2Call::new((
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ],
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ])),
        false
    )]
#[case(neqVecNestedHeapType2Call::new((
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ],
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xcafe0000000cafecafecafecafe0000000007357"),
                address!("0xdeadbeef0002000000000000000000000000cafe"),
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ])),
        true
    )]
#[case(neqVecNestedHeapType2Call::new((
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0002000000000000000000000000cafe"),
                address!("0xcafe000000020000000000000000000000007357")
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
                address!("0xcafe000000030000000000000000000000007357")
            ],
        ],
        vec![
            vec![
                address!("0xdeadbeef0000000000000000000000000000cafe"),
                address!("0xcafe000000000000000000000000000000007357")
            ],
            vec![
                address!("0xcafe0000000cafecafecafecafe0000000007357"),
                address!("0xdeadbeef0002000000000000000000000000cafe"),
            ],
            vec![
                address!("0xdeadbeef0003000000000000000000000000cafe"),
            ],
        ])),
        true
    )]
fn test_not_equality_vector<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: bool,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        <sol!((bool,))>::abi_encode(&(expected_result,)),
    )
    .unwrap();
}
