use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

mod primitives {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "equality";
        const SOURCE_PATH: &str = "tests/operations-equality/primitives.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

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

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "equality_vectors";
        const SOURCE_PATH: &str = "tests/operations-equality/vectors.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
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
    );

    #[rstest]
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

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "equality_references";
        const SOURCE_PATH: &str = "tests/operations-equality/references.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

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

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "equality_structs";
        const SOURCE_PATH: &str = "tests/operations-equality/structs.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

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
    use crate::common::translate_test_complete_package;

    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_packages =
            translate_test_complete_package("tests/operations-equality/external");

        let translated_package = translated_packages
            .get_mut("equality_external_structs")
            .unwrap();
        RuntimeSandbox::new(translated_package)
    }

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
