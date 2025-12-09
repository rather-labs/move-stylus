mod common;

use alloy_primitives::U256;
use alloy_primitives::{address, keccak256};
use alloy_sol_types::{SolCall, sol};
use move_test_runner::wasm_runner::{ExecutionData, RuntimeSandbox};
use rstest::{fixture, rstest};

declare_fixture!("hash_type_and_key", "tests/native/hash_type_and_key.move");

const ADDRESS: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe,
    0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe, 0xca, 0xfe,
];

const ADDRESS_2: &[u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef,
    0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef, 0xbe, 0xef,
];

fn merge_arrays<T: Clone>(arrays: &[&[T]]) -> Vec<T> {
    arrays
        .iter()
        .flat_map(|slice| slice.iter().cloned())
        .collect()
}

sol!(
    #[allow(missing_docs)]
    struct Bar {
        uint32 n;
        uint128 o;
    }

    struct Foo {
        Bar p;
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
    }

    struct BazU8 {
        uint8 g;
        Bar p;
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
    }

    struct BazVU16 {
        uint16[] g;
        Bar p;
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
    }

    function hashU8(uint8 a) public view;
    function hashU16(uint16 a) public view;
    function hashU32(uint32 a) public view;
    function hashU64(uint64 a) public view;
    function hashU128(uint128 a) public view;
    function hashU256(uint256 a) public view;
    function hashBool(bool a) public view;
    function hashAddress(address a) public view;
    function hashVectorU8(uint8[] a) public view;
    function hashVectorU16(uint16[] a) public view;
    function hashVectorU32(uint32[] a) public view;
    function hashVectorU64(uint64[] a) public view;
    function hashVectorU128(uint128[] a) public view;
    function hashVectorU256(uint256[] a) public view;
    function hashVectorBool(bool[] a) public view;
    function hashVectorAddress(address[] a) public view;
    function hashFoo(Foo a) public view;
    function hashBar(Bar a) public view;
    function hashBazU8(BazU8 a) public view;
    function hashBazVU16(BazVU16 a) public view;
);

#[rstest]
#[case(
        hashU8Call::new((42,)),
        merge_arrays(&[ADDRESS, &[42], b"u8".as_slice()])
    )]
#[case(
        hashU16Call::new((4242,)),
        merge_arrays(&[ADDRESS, &4242u16.to_le_bytes(), b"u16".as_slice()])
    )]
#[case(
        hashU32Call::new((42424242,)),
        merge_arrays(&[ADDRESS, &42424242u32.to_le_bytes(), b"u32".as_slice()])
    )]
#[case(
        hashU64Call::new((42424242424242,)),
        merge_arrays(&[ADDRESS, &42424242424242u64.to_le_bytes(), b"u64".as_slice()])
    )]
#[case(
        hashU128Call::new((42424242424242_u128,)),
        merge_arrays(&[ADDRESS, &42424242424242_u128.to_le_bytes(), b"u128".as_slice()])
    )]
#[case(
        hashU256Call::new((U256::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007913129639935", 10).unwrap(),)),
        merge_arrays(&[ADDRESS, &U256::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007913129639935", 10).unwrap().to_le_bytes::<32>(), b"u256".as_slice()])
    )]
#[case(
        hashBoolCall::new((true,)),
        merge_arrays(&[ADDRESS, &[1], b"bool".as_slice()])
    )]
#[case(
        hashBoolCall::new((false,)),
        merge_arrays(&[ADDRESS, &[0], b"bool".as_slice()])
    )]
#[case(
        hashAddressCall::new((address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),)),
        merge_arrays(&[ADDRESS, ADDRESS_2, b"address".as_slice()])
    )]
#[case(
        hashVectorU8Call::new((vec![1u8, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &[1u8, 2, 3, 4, 5], b"vector<u8>".as_slice()])
    )]
#[case(
        hashVectorU16Call::new((vec![1u16, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u16.to_le_bytes(), &2u16.to_le_bytes(), &3u16.to_le_bytes(), &4u16.to_le_bytes(), &5u16.to_le_bytes(), b"vector<u16>".as_slice()])
    )]
#[case(
        hashVectorU32Call::new((vec![1u32, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u32.to_le_bytes(), &2u32.to_le_bytes(), &3u32.to_le_bytes(), &4u32.to_le_bytes(), &5u32.to_le_bytes(), b"vector<u32>".as_slice()])
    )]
#[case(
        hashVectorU64Call::new((vec![1u64, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u64.to_le_bytes(), &2u64.to_le_bytes(), &3u64.to_le_bytes(), &4u64.to_le_bytes(), &5u64.to_le_bytes(), b"vector<u64>".as_slice()])
    )]
#[case(
        hashVectorU128Call::new((vec![1u128, 2, 3, 4, 5],)),
        merge_arrays(&[ADDRESS, &1u128.to_le_bytes(), &2u128.to_le_bytes(), &3u128.to_le_bytes(), &4u128.to_le_bytes(), &5u128.to_le_bytes(), b"vector<u128>".as_slice()])
    )]
#[case(
        hashVectorU256Call::new((vec![
            U256::from(1u64),
            U256::from(2u64),
            U256::from(3u64),
            U256::from(4u64),
            U256::from(5u64)
        ],)),
        merge_arrays(&[ADDRESS, &U256::from(1u64).to_le_bytes::<32>(), &U256::from(2u64).to_le_bytes::<32>(), &U256::from(3u64).to_le_bytes::<32>(), &U256::from(4u64).to_le_bytes::<32>(), &U256::from(5u64).to_le_bytes::<32>(), b"vector<u256>".as_slice()])
    )]
#[case(
        hashVectorBoolCall::new((vec![true, false, true, false],)),
        merge_arrays(&[ADDRESS, &[1, 0, 1, 0], b"vector<bool>".as_slice()])
    )]
#[case(
        hashVectorAddressCall::new((
            vec![
                address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
                address!("0xcafecafecafecafecafecafecafecafecafecafe")
            ]
        ,)),
        merge_arrays(&[ADDRESS, ADDRESS_2, ADDRESS, b"vector<address>".as_slice()])
    )]
#[case(
        hashBarCall::new((
            Bar {
                n: 42,
                o: 42424242424242_u128,
            },
        )),
        merge_arrays(&[
            ADDRESS,
            &42u32.to_le_bytes(),
            &42424242424242_u128.to_le_bytes(),
            b"Bar".as_slice(),
        ])
    )]
#[case(
        hashFooCall::new((
            Foo {
                p: Bar {
                    n: 42,
                    o: 42424242424242_u128,
                },
                q: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
                r: vec![1u32, 2, 3, 4, 5],
                s: vec![1u128, 2, 3, 4, 5],
                t: true,
                u: 8,
                v: 16,
                w: 32,
                x: 64,
                y: 128,
                z: U256::from(256),
            },
        )),
        merge_arrays(&[
            ADDRESS,
            // Bar
            &42u32.to_le_bytes(),
            &42424242424242_u128.to_le_bytes(),
            // address
            ADDRESS_2,
            // vector<u32>
            &1u32.to_le_bytes(),
            &2u32.to_le_bytes(),
            &3u32.to_le_bytes(),
            &4u32.to_le_bytes(),
            &5u32.to_le_bytes(),
            // vector<u128>
            &1u128.to_le_bytes(),
            &2u128.to_le_bytes(),
            &3u128.to_le_bytes(),
            &4u128.to_le_bytes(),
            &5u128.to_le_bytes(),
            // bool
            &[1],
            // u8
            &[8],
            // u16
            &16u16.to_le_bytes(),
            // u32
            &32u32.to_le_bytes(),
            // u64
            &64u64.to_le_bytes(),
            // u128
            &128u128.to_le_bytes(),
            // u256
            &U256::from(256).to_le_bytes::<32>(),
            b"Foo".as_slice(),
        ])
    )]
#[case(
        hashBazU8Call::new((
            BazU8 {
                g: 8,
                p: Bar {
                    n: 42,
                    o: 42424242424242_u128,
                },
                q: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
                r: vec![1u32, 2, 3, 4, 5],
                s: vec![1u128, 2, 3, 4, 5],
                t: true,
                u: 8,
                v: 16,
                w: 32,
                x: 64,
                y: 128,
                z: U256::from(256),
            },
        )),
        merge_arrays(&[
            ADDRESS,
            // u8
            &[8],
            // Bar
            &42u32.to_le_bytes(),
            &42424242424242_u128.to_le_bytes(),
            // address
            ADDRESS_2,
            // vector<u32>
            &1u32.to_le_bytes(),
            &2u32.to_le_bytes(),
            &3u32.to_le_bytes(),
            &4u32.to_le_bytes(),
            &5u32.to_le_bytes(),
            // vector<u128>
            &1u128.to_le_bytes(),
            &2u128.to_le_bytes(),
            &3u128.to_le_bytes(),
            &4u128.to_le_bytes(),
            &5u128.to_le_bytes(),
            // bool
            &[1],
            // u8
            &[8],
            // u16
            &16u16.to_le_bytes(),
            // u32
            &32u32.to_le_bytes(),
            // u64
            &64u64.to_le_bytes(),
            // u128
            &128u128.to_le_bytes(),
            // u256
            &U256::from(256).to_le_bytes::<32>(),
            b"Baz<u8>".as_slice(),
        ])
    )]
#[case(
        hashBazVU16Call::new((
            BazVU16 {
                g: vec![16u16, 32, 48, 64],
                p: Bar {
                    n: 42,
                    o: 42424242424242_u128,
                },
                q: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
                r: vec![1u32, 2, 3, 4, 5],
                s: vec![1u128, 2, 3, 4, 5],
                t: true,
                u: 8,
                v: 16,
                w: 32,
                x: 64,
                y: 128,
                z: U256::from(256),
            },
        )),
        merge_arrays(&[
            ADDRESS,
            // vector<u16>
            &16u16.to_le_bytes(),
            &32u16.to_le_bytes(),
            &48u16.to_le_bytes(),
            &64u16.to_le_bytes(),
            // Bar
            &42u32.to_le_bytes(),
            &42424242424242_u128.to_le_bytes(),
            // address
            ADDRESS_2,
            // vector<u32>
            &1u32.to_le_bytes(),
            &2u32.to_le_bytes(),
            &3u32.to_le_bytes(),
            &4u32.to_le_bytes(),
            &5u32.to_le_bytes(),
            // vector<u128>
            &1u128.to_le_bytes(),
            &2u128.to_le_bytes(),
            &3u128.to_le_bytes(),
            &4u128.to_le_bytes(),
            &5u128.to_le_bytes(),
            // bool
            &[1],
            // u8
            &[8],
            // u16
            &16u16.to_le_bytes(),
            // u32
            &32u32.to_le_bytes(),
            // u64
            &64u64.to_le_bytes(),
            // u128
            &128u128.to_le_bytes(),
            // u256
            &U256::from(256).to_le_bytes::<32>(),
            b"Baz<vector<u16>>".as_slice(),
        ])
    )]
// This checks that the data that will be hashed is correct by inspecting the memory of what
// we are about to hash and comparing it to the expected result
fn test_hash_type_and_key_hashing_data<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u8>,
) {
    let ExecutionData {
        return_data,
        instance,
        mut store,
        ..
    } = runtime
        .call_entrypoint_with_data(call_data.abi_encode())
        .unwrap();
    let last_allocation = i32::from_be_bytes(
        return_data[return_data.len() - 4..return_data.len()]
            .try_into()
            .unwrap(),
    );
    // The last allocated position belongs to the 32 bytes allocated for the keccak function,
    // so, to read what we are really hashing we need to extract that, and the expected result
    // length
    let read_from = last_allocation as usize - 32 - expected_result.len();

    let read_memory =
        RuntimeSandbox::read_memory_from(&instance, &mut store, read_from, expected_result.len())
            .unwrap();

    assert_eq!(expected_result, read_memory);

    let hashed_data =
        RuntimeSandbox::read_memory_from(&instance, &mut store, last_allocation as usize - 32, 32)
            .unwrap();

    let expected_hashed = keccak256(&expected_result);

    assert_eq!(expected_hashed.as_slice(), hashed_data.as_slice());
}
