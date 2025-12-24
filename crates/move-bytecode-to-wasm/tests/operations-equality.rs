mod common;

use crate::common::run_test;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

mod primitives {
    use super::*;

    declare_fixture!("equality", "tests/operations-equality/primitives.move");

    sol!(
        #[allow(missing_docs)]
        function eqAddress(address x, address y) external returns (bool);
        function eqU256(uint256 x, uint256 y) external returns (bool);
        function eqU128(uint128 x, uint128 y) external returns (bool);
        function eqU64(uint64 x, uint64 y) external returns (bool);
        function eqU32(uint32 x, uint32 y) external returns (bool);
        function eqU16(uint16 x, uint16 y) external returns (bool);
        function eqU8(uint8 x, uint8 y) external returns (bool);
        function neqAddress(address x, address y) external returns (bool);
        function neqU256(uint256 x, uint256 y) external returns (bool);
        function neqU128(uint128 x, uint128 y) external returns (bool);
        function neqU64(uint64 x, uint64 y) external returns (bool);
        function neqU32(uint32 x, uint32 y) external returns (bool);
        function neqU16(uint16 x, uint16 y) external returns (bool);
        function neqU8(uint8 x, uint8 y) external returns (bool);
    );

    #[rstest]
    #[case(eqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xcafe000000000000000000000000000000007357"))),
        true
    )]
    #[case(eqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xdeadbeef0000000000000000000000000000cafe"))),
        false
    )]
    #[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqU256Call::new((U256::from(0), U256::from(1) << 255)), false)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), false)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqU128Call::new((u128::MAX, u128::MAX - 42)), false)]
    #[case(eqU128Call::new((0, 1 << 127)), false)]
    #[case(eqU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(eqU64Call::new((u64::MAX, u64::MAX - 42)), false)]
    #[case(eqU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(eqU64Call::new((u64::MAX, u64::MAX - 42)), false)]
    #[case(eqU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(eqU32Call::new((u32::MAX, u32::MAX - 42)), false)]
    #[case(eqU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(eqU16Call::new((u16::MAX, u16::MAX - 42)), false)]
    #[case(eqU8Call::new((u8::MAX, u8::MAX)), true)]
    #[case(eqU8Call::new((u8::MAX, u8::MAX - 42)), false)]
    fn test_equality_primitive_types<T: SolCall>(
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
    #[case(neqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xcafe000000000000000000000000000000007357"))),
        false
    )]
    #[case(neqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xdeadbeef0000000000000000000000000000cafe"))),
        true
    )]
    #[case(neqU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqU256Call::new((U256::from(0), U256::from(1) << 255)), true)]
    #[case(neqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), true)]
    #[case(neqU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqU128Call::new((u128::MAX, u128::MAX - 42)), true)]
    #[case(neqU128Call::new((0, 1 << 127)), true)]
    #[case(neqU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(neqU64Call::new((u64::MAX, u64::MAX - 42)), true)]
    #[case(neqU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(neqU64Call::new((u64::MAX, u64::MAX - 42)), true)]
    #[case(neqU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(neqU32Call::new((u32::MAX, u32::MAX - 42)), true)]
    #[case(neqU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(neqU16Call::new((u16::MAX, u16::MAX - 42)), true)]
    #[case(neqU8Call::new((u8::MAX, u8::MAX)), false)]
    #[case(neqU8Call::new((u8::MAX, u8::MAX - 42)), true)]
    fn test_not_equality_primitive_types<T: SolCall>(
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
}

mod vector {
    use super::*;

    declare_fixture!("equality_vectors", "tests/operations-equality/vectors.move");

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
}

mod references {
    use super::*;

    declare_fixture!(
        "equality_references",
        "tests/operations-equality/references.move"
    );

    sol!(
        #[allow(missing_docs)]
        function eqAddress(address x, address y) external returns (bool);
        function eqU256(uint256 x, uint256 y) external returns (bool);
        function eqU128(uint128 x, uint128 y) external returns (bool);
        function eqU64(uint64 x, uint64 y) external returns (bool);
        function eqU32(uint32 x, uint32 y) external returns (bool);
        function eqU16(uint16 x, uint16 y) external returns (bool);
        function eqU8(uint8 x, uint8 y) external returns (bool);
        function eqVecStackType(uint16[], uint16[]) external returns (bool);
        function eqVecHeapType(uint128[], uint128[]) external returns (bool);
        function eqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
        function eqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
        function neqAddress(address x, address y) external returns (bool);
        function neqU256(uint256 x, uint256 y) external returns (bool);
        function neqU128(uint128 x, uint128 y) external returns (bool);
        function neqU64(uint64 x, uint64 y) external returns (bool);
        function neqU32(uint32 x, uint32 y) external returns (bool);
        function neqU16(uint16 x, uint16 y) external returns (bool);
        function neqU8(uint8 x, uint8 y) external returns (bool);
        function neqVecStackType(uint16[], uint16[]) external returns (bool);
        function neqVecHeapType(uint128[], uint128[]) external returns (bool);
        function neqVecNestedStackType(uint16[][], uint16[][]) external returns (bool);
        function neqVecNestedHeapType(uint128[][], uint128[][]) external returns (bool);
    );

    #[rstest]
    #[case(eqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xcafe000000000000000000000000000000007357"))),
        true
    )]
    #[case(eqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xdeadbeef0000000000000000000000000000cafe"))),
        false
    )]
    #[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqU256Call::new((U256::from(0), U256::from(1) << 255)), false)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), false)]
    #[case(eqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqU128Call::new((u128::MAX, u128::MAX - 42)), false)]
    #[case(eqU128Call::new((0, 1 << 127)), false)]
    #[case(eqU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(eqU64Call::new((u64::MAX, u64::MAX - 42)), false)]
    #[case(eqU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(eqU64Call::new((u64::MAX, u64::MAX - 42)), false)]
    #[case(eqU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(eqU32Call::new((u32::MAX, u32::MAX - 42)), false)]
    #[case(eqU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(eqU16Call::new((u16::MAX, u16::MAX - 42)), false)]
    #[case(eqU8Call::new((u8::MAX, u8::MAX)), true)]
    #[case(eqU8Call::new((u8::MAX, u8::MAX - 42)), false)]
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
    fn test_equality_references<T: SolCall>(
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
    #[case(neqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xcafe000000000000000000000000000000007357"))),
        false
    )]
    #[case(neqAddressCall::new((
        address!("0xcafe000000000000000000000000000000007357"),
        address!("0xdeadbeef0000000000000000000000000000cafe"))),
        true
    )]
    #[case(neqU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqU256Call::new((U256::from(0), U256::from(1) << 255)), true)]
    #[case(neqU256Call::new((U256::MAX, U256::MAX - U256::from(42))), true)]
    #[case(neqU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqU128Call::new((u128::MAX, u128::MAX - 42)), true)]
    #[case(neqU128Call::new((0, 1 << 127)), true)]
    #[case(neqU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(neqU64Call::new((u64::MAX, u64::MAX - 42)), true)]
    #[case(neqU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(neqU64Call::new((u64::MAX, u64::MAX - 42)), true)]
    #[case(neqU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(neqU32Call::new((u32::MAX, u32::MAX - 42)), true)]
    #[case(neqU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(neqU16Call::new((u16::MAX, u16::MAX - 42)), true)]
    #[case(neqU8Call::new((u8::MAX, u8::MAX)), false)]
    #[case(neqU8Call::new((u8::MAX, u8::MAX - 42)), true)]
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
    fn test_not_equality_references<T: SolCall>(
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
}

mod structs {
    use super::*;

    declare_fixture!("equality_structs", "tests/operations-equality/structs.move");

    sol!(
        #[allow(missing_docs)]
        function eqStructBool(bool a, bool b) external returns (bool);
        function eqStructAddress(address a, address b) external returns (bool);
        function eqStructU256(uint256 a, uint256 b) external returns (bool);
        function eqStructU128(uint128 a, uint128 b) external returns (bool);
        function eqStructU64(uint64 a, uint64 b) external returns (bool);
        function eqStructU32(uint32 a, uint32 b) external returns (bool);
        function eqStructU16(uint16 a, uint16 b) external returns (bool);
        function eqStructU8(uint8 a, uint8 b) external returns (bool);
        function eqStructVecStackType(uint32[] a, uint32[] b) external returns (bool);
        function eqStructVecHeapType(uint128[] a, uint128[] b) external returns (bool);
        function eqStructStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
        function neqStructBool(bool a, bool b) external returns (bool);
        function neqStructAddress(address a, address b) external returns (bool);
        function neqStructU256(uint256 a, uint256 b) external returns (bool);
        function neqStructU128(uint128 a, uint128 b) external returns (bool);
        function neqStructU64(uint64 a, uint64 b) external returns (bool);
        function neqStructU32(uint32 a, uint32 b) external returns (bool);
        function neqStructU16(uint16 a, uint16 b) external returns (bool);
        function neqStructU8(uint8 a, uint8 b) external returns (bool);
        function neqStructVecStackType(uint32[] a, uint32[] b) external returns (bool);
        function neqStructVecHeapType(uint128[] a, uint128[] b) external returns (bool);
        function neqStructStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    );

    #[rstest]
    #[case(eqStructBoolCall::new((true, true)), true)]
    #[case(eqStructBoolCall::new((false, true)), false)]
    #[case(eqStructU8Call::new((255, 255)), true)]
    #[case(eqStructU8Call::new((1, 255)), false)]
    #[case(eqStructU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(eqStructU16Call::new((1, u16::MAX)), false)]
    #[case(eqStructU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(eqStructU32Call::new((1, u32::MAX)), false)]
    #[case(eqStructU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(eqStructU64Call::new((1, u64::MAX)), false)]
    #[case(eqStructU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(eqStructU128Call::new((1, u128::MAX)), false)]
    #[case(eqStructU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqStructU256Call::new((U256::from(1), U256::MAX)), false)]
    #[case(eqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), true)]
    #[case(eqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), true)]
    #[case(eqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqStructAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         true
    )]
    #[case(eqStructAddressCall::new(
        (address!("0xcafe0000000000deadbeefdeadbeef0000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         false
    )]
    #[case(eqStructStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
    #[case(eqStructStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
    fn test_equality_struct<T: SolCall>(
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
    #[case(neqStructBoolCall::new((true, true)), false)]
    #[case(neqStructBoolCall::new((false, true)), true)]
    #[case(neqStructU8Call::new((255, 255)), false)]
    #[case(neqStructU8Call::new((1, 255)), true)]
    #[case(neqStructU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(neqStructU16Call::new((1, u16::MAX)), true)]
    #[case(neqStructU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(neqStructU32Call::new((1, u32::MAX)), true)]
    #[case(neqStructU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(neqStructU64Call::new((1, u64::MAX)), true)]
    #[case(neqStructU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(neqStructU128Call::new((1, u128::MAX)), true)]
    #[case(neqStructU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqStructU256Call::new((U256::from(1), U256::MAX)), true)]
    #[case(neqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), false)]
    #[case(neqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), false)]
    #[case(neqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqStructAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         false
    )]
    #[case(neqStructAddressCall::new(
        (address!("0xcafe0000000000deadbeefdeadbeef0000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         true
    )]
    #[case(neqStructStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
    #[case(neqStructStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
    fn test_not_equality_struct<T: SolCall>(
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
}

mod external_structs {
    use super::*;

    declare_fixture!(
        "equality_external_structs",
        "tests/operations-equality/external"
    );

    sol!(
        #[allow(missing_docs)]
        function eqStructBool(bool a, bool b) external returns (bool);
        function eqStructAddress(address a, address b) external returns (bool);
        function eqStructU256(uint256 a, uint256 b) external returns (bool);
        function eqStructU128(uint128 a, uint128 b) external returns (bool);
        function eqStructU64(uint64 a, uint64 b) external returns (bool);
        function eqStructU32(uint32 a, uint32 b) external returns (bool);
        function eqStructU16(uint16 a, uint16 b) external returns (bool);
        function eqStructU8(uint8 a, uint8 b) external returns (bool);
        function eqStructVecStackType(uint32[] a, uint32[] b) external returns (bool);
        function eqStructVecHeapType(uint128[] a, uint128[] b) external returns (bool);
        function eqStructStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
        function neqStructBool(bool a, bool b) external returns (bool);
        function neqStructAddress(address a, address b) external returns (bool);
        function neqStructU256(uint256 a, uint256 b) external returns (bool);
        function neqStructU128(uint128 a, uint128 b) external returns (bool);
        function neqStructU64(uint64 a, uint64 b) external returns (bool);
        function neqStructU32(uint32 a, uint32 b) external returns (bool);
        function neqStructU16(uint16 a, uint16 b) external returns (bool);
        function neqStructU8(uint8 a, uint8 b) external returns (bool);
        function neqStructVecStackType(uint32[] a, uint32[] b) external returns (bool);
        function neqStructVecHeapType(uint128[] a, uint128[] b) external returns (bool);
        function neqStructStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
    );

    #[rstest]
    #[case(eqStructBoolCall::new((true, true)), true)]
    #[case(eqStructBoolCall::new((false, true)), false)]
    #[case(eqStructU8Call::new((255, 255)), true)]
    #[case(eqStructU8Call::new((1, 255)), false)]
    #[case(eqStructU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(eqStructU16Call::new((1, u16::MAX)), false)]
    #[case(eqStructU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(eqStructU32Call::new((1, u32::MAX)), false)]
    #[case(eqStructU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(eqStructU64Call::new((1, u64::MAX)), false)]
    #[case(eqStructU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(eqStructU128Call::new((1, u128::MAX)), false)]
    #[case(eqStructU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqStructU256Call::new((U256::from(1), U256::MAX)), false)]
    #[case(eqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), true)]
    #[case(eqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), true)]
    #[case(eqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqStructAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         true
    )]
    #[case(eqStructAddressCall::new(
        (address!("0xcafe0000000000deadbeefdeadbeef0000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         false
    )]
    #[case(eqStructStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
    #[case(eqStructStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
    fn test_equality_external_struct<T: SolCall>(
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
    #[case(neqStructBoolCall::new((true, true)), false)]
    #[case(neqStructBoolCall::new((false, true)), true)]
    #[case(neqStructU8Call::new((255, 255)), false)]
    #[case(neqStructU8Call::new((1, 255)), true)]
    #[case(neqStructU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(neqStructU16Call::new((1, u16::MAX)), true)]
    #[case(neqStructU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(neqStructU32Call::new((1, u32::MAX)), true)]
    #[case(neqStructU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(neqStructU64Call::new((1, u64::MAX)), true)]
    #[case(neqStructU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(neqStructU128Call::new((1, u128::MAX)), true)]
    #[case(neqStructU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqStructU256Call::new((U256::from(1), U256::MAX)), true)]
    #[case(neqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), false)]
    #[case(neqStructVecStackTypeCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), false)]
    #[case(neqStructVecHeapTypeCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqStructAddressCall::new(
        (address!("0xcafe000000000000000000000000000000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         false
    )]
    #[case(neqStructAddressCall::new(
        (address!("0xcafe0000000000deadbeefdeadbeef0000007357"),
         address!("0xcafe000000000000000000000000000000007357"))),
         true
    )]
    #[case(neqStructStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
    #[case(neqStructStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
    fn test_not_equality_extnernal_struct<T: SolCall>(
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
}

mod enums {
    use super::*;

    declare_fixture!("equality_enums", "tests/operations-equality/enums.move");

    sol!(
        #[allow(missing_docs)]
        function eqSimpleEnumBool(bool a, bool b) external returns (bool);
        function eqSimpleEnumU8(uint8 a, uint8 b) external returns (bool);
        function eqSimpleEnumU16(uint16 a, uint16 b) external returns (bool);
        function eqSimpleEnumU32(uint32 a, uint32 b) external returns (bool);
        function eqSimpleEnumU64(uint64 a, uint64 b) external returns (bool);
        function eqSimpleEnumU128(uint128 a, uint128 b) external returns (bool);
        function eqSimpleEnumU256(uint256 a, uint256 b) external returns (bool);
        function eqSimpleEnumAddress(address a, address b) external returns (bool);
        function eqVectorEnumStack(uint32[] a, uint32[] b) external returns (bool);
        function eqVectorEnumHeap(uint128[] a, uint128[] b) external returns (bool);
        function eqVectorEnumBool(bool[] a, bool[] b) external returns (bool);
        function eqVectorEnumAddress(address[] a, address[] b) external returns (bool);
        function eqStructEnumWithStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
        function eqStructEnumWithPrimitives(uint8 a, uint16 b, uint32 c, uint8 d, uint16 e, uint32 f) external returns (bool);
        function eqStructEnumMixed(uint32 a, uint128 b, uint64 c, uint32 d, uint128 e, uint64 f) external returns (bool);
        function eqComplexEnumSimple(uint32 a, uint32 b) external returns (bool);
        function eqComplexEnumVector(uint64[] a, uint64[] b) external returns (bool);
        function eqComplexEnumStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
        function eqComplexEnumNested(uint32 a, uint128 b, uint32[] c, bool d, uint32 e, uint128 f, uint32[] g, bool h) external returns (bool);
        function neqSimpleEnumBool(bool a, bool b) external returns (bool);
        function neqSimpleEnumU8(uint8 a, uint8 b) external returns (bool);
        function neqSimpleEnumU16(uint16 a, uint16 b) external returns (bool);
        function neqSimpleEnumU32(uint32 a, uint32 b) external returns (bool);
        function neqSimpleEnumU64(uint64 a, uint64 b) external returns (bool);
        function neqSimpleEnumU128(uint128 a, uint128 b) external returns (bool);
        function neqSimpleEnumU256(uint256 a, uint256 b) external returns (bool);
        function neqSimpleEnumAddress(address a, address b) external returns (bool);
        function neqVectorEnumStack(uint32[] a, uint32[] b) external returns (bool);
        function neqVectorEnumHeap(uint128[] a, uint128[] b) external returns (bool);
        function neqVectorEnumBool(bool[] a, bool[] b) external returns (bool);
        function neqVectorEnumAddress(address[] a, address[] b) external returns (bool);
        function neqStructEnumWithStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
        function neqStructEnumWithPrimitives(uint8 a, uint16 b, uint32 c, uint8 d, uint16 e, uint32 f) external returns (bool);
        function neqStructEnumMixed(uint32 a, uint128 b, uint64 c, uint32 d, uint128 e, uint64 f) external returns (bool);
        function neqComplexEnumSimple(uint32 a, uint32 b) external returns (bool);
        function neqComplexEnumVector(uint64[] a, uint64[] b) external returns (bool);
        function neqComplexEnumStruct(uint32 a, uint128 b, uint32 c, uint128 d) external returns (bool);
        function neqComplexEnumNested(uint32 a, uint128 b, uint32[] c, bool d, uint32 e, uint128 f, uint32[] g, bool h) external returns (bool);
        function eqVectorSimpleEnums(uint8[] a, uint8[] b) external returns (bool);
        function neqVectorSimpleEnums(uint8[] a, uint8[] b) external returns (bool);
        function eqVectorStructEnums(uint32[] a, uint128[] b, uint32[] c, uint128[] d) external returns (bool);
        function neqVectorStructEnums(uint32[] a, uint128[] b, uint32[] c, uint128[] d) external returns (bool);
        function eqVectorComplexEnums(uint32[] a, uint32[] b) external returns (bool);
        function neqVectorComplexEnums(uint32[] a, uint32[] b) external returns (bool);
        function eqVectorMixedEnums(uint32[] a, uint64[] b) external returns (bool);
        function neqVectorMixedEnums(uint32[] a, uint64[] b) external returns (bool);

    );
    #[rstest]
    #[case(eqSimpleEnumBoolCall::new((true, true)), true)]
    #[case(eqSimpleEnumBoolCall::new((false, true)), false)]
    #[case(eqSimpleEnumU8Call::new((255, 255)), true)]
    #[case(eqSimpleEnumU8Call::new((1, 255)), false)]
    #[case(eqSimpleEnumU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(eqSimpleEnumU16Call::new((1, u16::MAX)), false)]
    #[case(eqSimpleEnumU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(eqSimpleEnumU32Call::new((1, u32::MAX)), false)]
    #[case(eqSimpleEnumU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(eqSimpleEnumU64Call::new((1, u64::MAX)), false)]
    #[case(eqSimpleEnumU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(eqSimpleEnumU128Call::new((1, u128::MAX)), false)]
    #[case(eqSimpleEnumU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(eqSimpleEnumU256Call::new((U256::from(1), U256::MAX)), false)]
    #[case(eqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), true)]
    #[case(eqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), true)]
    #[case(eqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,true])), true)]
    #[case(eqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,false])), false)]
    #[case(eqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")])), true)]
    #[case(eqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007358")])), false)]
    #[case(eqStructEnumWithStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
    #[case(eqStructEnumWithStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
    #[case(eqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 255, u16::MAX, u32::MAX)), true)]
    #[case(eqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 254, u16::MAX, u32::MAX)), false)]
    #[case(eqStructEnumMixedCall::new((u32::MAX, u128::MAX, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), true)]
    #[case(eqStructEnumMixedCall::new((u32::MAX, u128::MAX-1, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), false)]
    #[case(eqComplexEnumSimpleCall::new((u32::MAX, u32::MAX)), true)]
    #[case(eqComplexEnumSimpleCall::new((u32::MAX, 1)), false)]
    #[case(eqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,u64::MAX,3,4])), true)]
    #[case(eqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,3,4,5])), false)]
    #[case(eqComplexEnumStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), true)]
    #[case(eqComplexEnumStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), false)]
    #[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), true)]
    #[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,3,4,5], true)), false)]
    #[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false)), false)]
    #[case(eqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), false)]
    // Vector enum equality tests
    #[case(eqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), true)]
    #[case(eqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), false)]
    #[case(eqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,64])), true)]
    #[case(eqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,63])), false)]
    #[case(eqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,3], vec![100,200,300])), true)]
    #[case(eqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,4], vec![100,200,300])), false)]
    #[case(eqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1])), true)]
    #[case(eqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-2], vec![u128::MAX, u128::MAX-1])), false)]
    #[case(eqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), true)]
    #[case(eqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), false)]
    #[case(eqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 0])), true)]
    #[case(eqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 1])), false)]
    #[case(eqVectorMixedEnumsCall::new((vec![1,2,3], vec![1,2,3])), false)] // Different variants should never be equal
    #[case(eqVectorMixedEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u64::MAX, u64::MAX-1])), false)] // Different variants should never be equal
    fn test_equality_enum<T: SolCall>(
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
    #[case(neqSimpleEnumBoolCall::new((true, true)), false)]
    #[case(neqSimpleEnumBoolCall::new((false, true)), true)]
    #[case(neqSimpleEnumU8Call::new((255, 255)), false)]
    #[case(neqSimpleEnumU8Call::new((1, 255)), true)]
    #[case(neqSimpleEnumU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(neqSimpleEnumU16Call::new((1, u16::MAX)), true)]
    #[case(neqSimpleEnumU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(neqSimpleEnumU32Call::new((1, u32::MAX)), true)]
    #[case(neqSimpleEnumU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(neqSimpleEnumU64Call::new((1, u64::MAX)), true)]
    #[case(neqSimpleEnumU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(neqSimpleEnumU128Call::new((1, u128::MAX)), true)]
    #[case(neqSimpleEnumU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(neqSimpleEnumU256Call::new((U256::from(1), U256::MAX)), true)]
    #[case(neqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,u32::MAX,3,4])), false)]
    #[case(neqVectorEnumStackCall::new((vec![1,2,u32::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,u128::MAX,3,4])), false)]
    #[case(neqVectorEnumHeapCall::new((vec![1,2,u128::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,true])), false)]
    #[case(neqVectorEnumBoolCall::new((vec![true,false,true], vec![true,false,false])), true)]
    #[case(neqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")])), false)]
    #[case(neqVectorEnumAddressCall::new((vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007357")], vec![address!("0xcafe000000000000000000000000000000007357"), address!("0xcafe000000000000000000000000000000007358")])), true)]
    #[case(neqStructEnumWithStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
    #[case(neqStructEnumWithStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
    #[case(neqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 255, u16::MAX, u32::MAX)), false)]
    #[case(neqStructEnumWithPrimitivesCall::new((255, u16::MAX, u32::MAX, 254, u16::MAX, u32::MAX)), true)]
    #[case(neqStructEnumMixedCall::new((u32::MAX, u128::MAX, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), false)]
    #[case(neqStructEnumMixedCall::new((u32::MAX, u128::MAX-1, u64::MAX, u32::MAX, u128::MAX, u64::MAX)), true)]
    #[case(neqComplexEnumSimpleCall::new((u32::MAX, u32::MAX)), false)]
    #[case(neqComplexEnumSimpleCall::new((u32::MAX, 1)), true)]
    #[case(neqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,u64::MAX,3,4])), false)]
    #[case(neqComplexEnumVectorCall::new((vec![1,2,u64::MAX,3,4], vec![1,2,3,4,5])), true)]
    #[case(neqComplexEnumStructCall::new((u32::MAX, u128::MAX, u32::MAX, u128::MAX)), false)]
    #[case(neqComplexEnumStructCall::new((u32::MAX, u128::MAX, 1, u128::MAX)), true)]
    #[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), false)]
    #[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,3,4,5], true)), true)]
    #[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false)), true)]
    #[case(neqComplexEnumNestedCall::new((u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], false, u32::MAX, u128::MAX, vec![1,2,u32::MAX,3,4], true)), true)]
    // Vector enum inequality tests
    #[case(neqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), false)]
    #[case(neqVectorSimpleEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), true)]
    #[case(neqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,64])), false)]
    #[case(neqVectorSimpleEnumsCall::new((vec![255,128,64], vec![255,128,63])), true)]
    #[case(neqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,3], vec![100,200,300])), false)]
    #[case(neqVectorStructEnumsCall::new((vec![1,2,3], vec![100,200,300], vec![1,2,4], vec![100,200,300])), true)]
    #[case(neqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1])), false)]
    #[case(neqVectorStructEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u128::MAX, u128::MAX-1], vec![u32::MAX, u32::MAX-2], vec![u128::MAX, u128::MAX-1])), true)]
    #[case(neqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,5])), false)]
    #[case(neqVectorComplexEnumsCall::new((vec![1,2,3,4,5], vec![1,2,3,4,6])), true)]
    #[case(neqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 0])), false)]
    #[case(neqVectorComplexEnumsCall::new((vec![u32::MAX, u32::MAX-1, 0], vec![u32::MAX, u32::MAX-1, 1])), true)]
    #[case(neqVectorMixedEnumsCall::new((vec![1,2,3], vec![1,2,3])), true)] // Different variants should always be unequal
    #[case(neqVectorMixedEnumsCall::new((vec![u32::MAX, u32::MAX-1], vec![u64::MAX, u64::MAX-1])), true)] // Different variants should always be unequal
    fn test_not_equality_enum<T: SolCall>(
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
}
