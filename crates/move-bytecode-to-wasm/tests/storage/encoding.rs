use crate::common::runtime;
use alloy_primitives::{U256, address, hex};
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]

    #[derive(Debug)]
    struct ID {
       bytes32 bytes;
    }

    #[derive(Debug)]
    struct UID {
       ID id;
    }

    struct StaticFields {
        UID id;
        uint256 a;
        uint128 b;
        uint64 c;
        uint32 d;
        uint16 e;
        uint8 f;
        address g;
    }

    struct StaticFields2 {
        UID id;
        uint8 a;
        address b;
        uint64 c;
        uint16 d;
        uint8 e;
    }

    struct StaticFields3 {
        UID id;
        uint8 a;
        address b;
        uint64 c;
        address d;
    }

    struct StaticNestedStruct {
        UID id;
        uint64 a;
        bool b;
        StaticNestedStructChild c;
        uint128 f;
        uint32 g;
    }

    struct StaticNestedStructChild {
        uint64 d;
        address e;
    }

    function saveStaticFields(
        uint256 a,
        uint128 b,
        uint64 c,
        uint32 d,
        uint16 e,
        uint8 f,
        address g
    ) public view;
    function readStaticFields(uint256 id) public view returns (StaticFields);

    function saveStaticFields2(
        uint8 a,
        address b,
        uint64 c,
        uint16 d,
        uint8 e
    ) public view;
    function readStaticFields2(uint256 id) public view returns (StaticFields2);

    function saveStaticFields3(
        uint8 a,
        address b,
        uint64 c,
        address d
    ) public view;
    function readStaticFields3(uint256 id) public view returns (StaticFields3);

    function saveStaticNestedStruct(
        uint64 a,
        bool b,
        uint64 d,
        address e,
        uint128 f,
        uint32 g
    ) public view;
    function readStaticNestedStruct(uint256 id) public view returns (StaticNestedStruct);

    // Dynamic structs
    struct DynamicStruct {
        UID id;
        uint32 a;
        bool b;
        uint32[] c;
        uint128[] d;
        uint64 e;
        uint128 f;
        uint256 g;
    }

    struct DynamicStruct2 {
        UID id;
        bool[] a;
        uint8[] b;
        uint16[] c;
        uint32[] d;
        uint64[] e;
        uint128[] f;
        uint256[] g;
        address[] h;
    }

    struct DynamicStruct3 {
        UID id;
        uint8[][] a;
        uint32[][] b;
        uint64[][] c;
        uint128[][] d;
    }

    struct DynamicStruct4 {
        UID id;
        DynamicNestedStructChild[] a;
        StaticNestedStructChild[] b;
    }

    struct DynamicNestedStructChild {
        uint32[] a;
        uint128 b;
    }

    struct NestedStructChildWrapper {
        DynamicNestedStructChild[] a;
        StaticNestedStructChild[] b;
    }

    struct DynamicStruct5 {
        UID id;
        NestedStructChildWrapper[] a;
    }

    struct GenericStruct32 {
        UID id;
        uint32[] a;
        uint32 b;
    }
    function saveDynamicStruct(
        uint32 a,
        bool b,
        uint64[] c,
        uint128[] d,
        uint64 e,
        uint128 f,
        uint256 g,
    ) public view;
    function readDynamicStruct(uint256 id) public view returns (DynamicStruct);

    function saveDynamicStruct2(
        bool[] a,
        uint8[] b,
        uint16[] c,
        uint32[] d,
        uint64[] e,
        uint128[] f,
        uint256[] g,
        address[] h,
    ) public view;
    function readDynamicStruct2(uint256 id) public view returns (DynamicStruct2);

    function saveDynamicStruct3(
        uint8[][] a,
        uint32[][] b,
        uint64[][] c,
        uint128[][] d,
    ) public view;
    function readDynamicStruct3(uint256 id) public view returns (DynamicStruct3);

    function saveDynamicStruct4(
        uint32[] x,
        uint64 y,
        uint128 z,
        address w,
    ) public view;
    function readDynamicStruct4(uint256 id) public view returns (DynamicStruct4);

    function saveDynamicStruct5(
        uint32 x,
        uint64 y,
        uint128 z,
        address w,
    ) public view;
    function readDynamicStruct5(uint256 id) public view returns (DynamicStruct5);

    function saveGenericStruct32(
        uint32 x,
    ) public view;
    function readGenericStruct32(uint256 id) public view returns (GenericStruct32);

    //// Wrapped objects ////
    struct Foo {
        UID id;
        uint64 a;
        Bar b;
        uint32 c;
    }

    struct Bar {
        UID id;
        uint64 a;
    }

    function saveFoo() public view;
    function readFoo(uint256 id) public view returns (Foo);

    struct MegaFoo {
        UID id;
        uint64 a;
        Foo b;
        uint32 c;
    }
    function saveMegaFoo() public view;
    function readMegaFoo(uint256 id) public view returns (MegaFoo);

    struct Var {
        UID id;
        Bar a;
        Foo b;
        Bar[] c;
    }

    function saveVar() public view;
    function readVar(uint256 id) public view returns (Var);

    struct GenericWrapper32 {
        UID id;
        uint32 a;
        GenericStruct32 b;
        uint32 c;
    }

    function saveGenericWrapper32() public view;
    function readGenericWrapper32(uint256 id) public view returns (GenericWrapper32);

    // Enums encoding
    function saveBarStruct() public view;
    function saveFooAStructA() public view;
    function saveFooAStructB() public view;
    function saveFooAStructC() public view;
    function saveFooBStructA() public view;
    function saveFooBStructB() public view;
    function saveFooBStructC() public view;
);

#[rstest]
#[case(saveStaticFieldsCall::new((
        U256::from_str_radix("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 16).unwrap(),
        0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        0xcccccccccccccccc,
        0xdddddddd,
        0xeeee,
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
    )), vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000da3b039f3b767d4d", 16).unwrap().to_be_bytes(),
        [0xaa; 32],
        U256::from_str_radix("ffeeeeddddddddccccccccccccccccbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],
        readStaticFieldsCall::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticFields {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
            a: U256::from_str_radix("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 16).unwrap(),
            b: 0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
            c: 0xcccccccccccccccc,
            d: 0xdddddddd,
            e: 0xeeee,
            f: 0xff,
            g: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        }
    )]
#[case(saveStaticFieldsCall::new((
        U256::from(1),
        2,
        3,
        4,
        5,
        6,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
    )), vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000da3b039f3b767d4d", 16).unwrap().to_be_bytes(),
        U256::from(1).to_be_bytes(),
        U256::from_str_radix("06000500000004000000000000000300000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],
        readStaticFieldsCall::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticFields {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
            a: U256::from(1),
            b: 2,
            c: 3,
            d: 4,
            e: 5,
            f: 6,
            g: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        }
    )]
#[case(saveStaticFields2Call::new((
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccc,
        0xeeee,
        0xff,
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafeffb9818d466d119af7", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000ffeeeecccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields2Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticFields2 {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
            a: 0xff,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 0xcccccccccccccccc,
            d: 0xeeee,
            e: 0xff,
        }
    )]
#[case(saveStaticFields2Call::new((
        1,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        2,
        3,
        4,
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafe01b9818d466d119af7", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000400030000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields2Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticFields2 {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
            a: 1,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 2,
            d: 3,
            e: 4,
        }
    )]
#[case(saveStaticFields3Call::new((
        1,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        2,
        address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafe01aba4dab37080822a", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000beefbeefbeefbeefbeefbeefbeefbeefbeefbeef0000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields3Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticFields3 {
           id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: 1,
           b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           c: 2,
           d: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        }
    )]
#[case(saveStaticFields3Call::new((
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccc,
        address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafeffaba4dab37080822a", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000beefbeefbeefbeefbeefbeefbeefbeefbeefbeefcccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields3Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticFields3 {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
            a: 0xff,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 0xcccccccccccccccc,
            d: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        }
    )]
#[case(saveStaticNestedStructCall::new((
        1,
        true,
        2,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        3,
        4
    )), vec![
        U256::from_str_radix("0000000000000000000000000000020100000000000000012d7ba8bfbb75aa1b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000400000000000000000000000000000003", 16).unwrap().to_be_bytes(),
    ],
        readStaticNestedStructCall::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticNestedStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: 1,
           b: true,
           c: StaticNestedStructChild {
                d: 2,
                e: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           },
           f: 3,
           g: 4,
        }
    )]
#[case(saveStaticNestedStructCall::new((
        0xaaaaaaaaaaaaaaaa,
        true,
        0xbbbbbbbbbbbbbbbb,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccccccccccccccccccc,
        0xdddddddd,
    )), vec![
        U256::from_str_radix("00000000000000bbbbbbbbbbbbbbbb01aaaaaaaaaaaaaaaa2d7ba8bfbb75aa1b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000ddddddddcccccccccccccccccccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticNestedStructCall::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        StaticNestedStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: 0xaaaaaaaaaaaaaaaa,
           b: true,
           c: StaticNestedStructChild {
                d: 0xbbbbbbbbbbbbbbbb,
                e: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           },
           f: 0xcccccccccccccccccccccccccccccccc,
           g: 0xdddddddd,
        }
    )]
fn test_static_fields<T: SolCall, U: SolCall, V: SolValue>(
    #[with("storage_encoding", "tests/storage/move_sources/encoding.move")] runtime: RuntimeSandbox,
    #[case] call_data_encode: T,
    #[case] expected_encode: Vec<[u8; 32]>,
    #[case] call_data_decode: U,
    #[case] expected_decode: V,
) {
    let (result, _) = runtime
        .call_entrypoint(call_data_encode.abi_encode())
        .unwrap();
    assert_eq!(0, result);

    // Check if it is encoded correctly in storage
    for (i, expected) in expected_encode.iter().enumerate() {
        let storage = runtime.get_storage_at_slot(U256::from(i).to_be_bytes());
        assert_eq!(expected, &storage, "Mismatch at slot {i}");
    }

    // Use the read function to check if it decodes correctly
    let (result, result_data) = runtime
        .call_entrypoint(call_data_decode.abi_encode())
        .unwrap();
    assert_eq!(0, result);
    assert_eq!(expected_decode.abi_encode(), result_data);
}

#[rstest]
#[case(saveDynamicStructCall::new((
        46,
        true,
        vec![2, 3, 4, 5, 6],
        vec![7, 8, 9],
        47,
        48,
        U256::from(49),
    )),
    vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01 (vector header)
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // 0x02 (vector header)
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // 0x03 u64 and u128 slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // 0x04 u64 and u128 slot

        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // vector elements second slot

        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7", 16).unwrap().to_be_bytes(), // vector elements second slot

    ],
    vec![
        U256::from_str_radix("00000000000000000000000000000000000000010000002e8b51f1c0eaf0a4e3", 16).unwrap().to_be_bytes(), // type hash + u32 + bool
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000030000000000000002f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000031", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000800000000000000000000000000000007", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000009", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("0000000000000005000000000000000400000000000000030000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStructCall::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        DynamicStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: 46,
           b: true,
           c: vec![2, 3, 4, 5, 6],
           d: vec![7, 8, 9],
           e: 47,
           f: 48,
           g: U256::from(49),
        }
    )]
#[case(saveDynamicStructCall::new((
        u32::MAX,
        true,
        vec![],
        vec![7, 8, 9],
        u64::MAX,
        48,
        U256::from(49),
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000001ffffffff8b51f1c0eaf0a4e3", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000000000000000000000000030ffffffffffffffff", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000031", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000800000000000000000000000000000007", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000009", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStructCall::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        DynamicStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: u32::MAX,
           b: true,
           c: vec![],
           d: vec![7, 8, 9],
           e: u64::MAX,
           f: 48,
           g: U256::from(49),
        }
    )]
#[case(saveDynamicStruct2Call::new((
        vec![true, false, true],
        vec![1, 2, 3, 4, 5], // u8
        vec![6, 7, 8, 9], // u16
        vec![10, 11, 12, 13, 14, 15], // u32
        vec![16, 17, 18, 19, 20], // u64
        vec![21, 22, 23], // u128
        vec![U256::from(24), U256::from(25)], // u256
        vec![address!("0x1111111111111111111111111111111111111111"), address!("0x2222222222222222222222222222222222222222")] // address
    )),
    vec![
        [0x00; 32], // 0x0 UID slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01 bool vector header
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // bool vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // u8 vec, header slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // u8 vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // u16 vec, header slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(), // u16 vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // u32 vec, header slot
        U256::from_str_radix("8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b", 16).unwrap().to_be_bytes(), // u32 vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(), // u64 vec, header slot
        U256::from_str_radix("036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db0", 16).unwrap().to_be_bytes(), // u64 vec, elem slot #1
        U256::from_str_radix("036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db1", 16).unwrap().to_be_bytes(), // u64 vec, elem slot #2
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(), // u128 vec, header slot
        U256::from_str_radix("f652222313e28459528d920b65115c16c04f3efc82aaedc97be59f3f377c0d3f", 16).unwrap().to_be_bytes(), // u128 vec, elem slot #1
        U256::from_str_radix("f652222313e28459528d920b65115c16c04f3efc82aaedc97be59f3f377c0d40", 16).unwrap().to_be_bytes(), // u128 vec, elem slot #2
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000007", 16).unwrap().to_be_bytes(), // u256 vec, header slot
        U256::from_str_radix("a66cc928b5edb82af9bd49922954155ab7b0942694bea4ce44661d9a8736c688", 16).unwrap().to_be_bytes(), // u256 vec, elem slot #1
        U256::from_str_radix("a66cc928b5edb82af9bd49922954155ab7b0942694bea4ce44661d9a8736c689", 16).unwrap().to_be_bytes(), // u256 vec, elem slot #2
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000008", 16).unwrap().to_be_bytes(), // address vec, header slot
        U256::from_str_radix("f3f7a9fe364faab93b216da50a3214154f22a0a2b415b23a84c8169e8b636ee3", 16).unwrap().to_be_bytes(), // address vec, elem slot #1
        U256::from_str_radix("f3f7a9fe364faab93b216da50a3214154f22a0a2b415b23a84c8169e8b636ee4", 16).unwrap().to_be_bytes(), // address vec, elem slot #2
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000002f093a11b671c4bf", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000010001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000504030201", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000009000800070006", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000f0000000e0000000d0000000c0000000b0000000a", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000013000000000000001200000000000000110000000000000010", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000014", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000001600000000000000000000000000000015", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000017", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000018", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000019", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000001111111111111111111111111111111111111111", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000002222222222222222222222222222222222222222", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct2Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        DynamicStruct2 {
        id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: vec![true, false, true],
           b: vec![1, 2, 3, 4, 5],
           c: vec![6, 7, 8, 9],
           d: vec![10, 11, 12, 13, 14, 15],
           e: vec![16, 17, 18, 19, 20],
           f: vec![21, 22, 23],
           g: vec![U256::from(24), U256::from(25)],
           h: vec![address!("0x1111111111111111111111111111111111111111"), address!("0x2222222222222222222222222222222222222222")],
        }
    )]
#[case(saveDynamicStruct3Call::new((
        vec![vec![1, 2, 3], vec![4, 5]],
        vec![vec![6, 7], vec![8], vec![9, 10]],
        vec![vec![11, 12, 13, 14], vec![], vec![15, 16]],
        vec![vec![17, 18, 19]],
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01 (u8[][] header) slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // first u8[] header slot
        U256::from_str_radix("b5d9d894133a730aa651ef62d26b0ffa846233c74177a591a4a896adfda97d22", 16).unwrap().to_be_bytes(), // first u8[] elements slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7", 16).unwrap().to_be_bytes(), // second u8[] header slot
        U256::from_str_radix("ea7809e925a8989e20c901c4c1da82f0ba29b26797760d445a0ce4cf3c6fbd31", 16).unwrap().to_be_bytes(), // second u8[] elements slot

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // 0x02 (u32[][] header) slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // first u32[] header slot
        U256::from_str_radix("1ab0c6948a275349ae45a06aad66a8bd65ac18074615d53676c09b67809099e0", 16).unwrap().to_be_bytes(), // first u32[] elements slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // second u32[] header slot
        U256::from_str_radix("2f2149d90beac0570c7f26368e4bc897ca24bba51b1a0f4960d358f764f11f31", 16).unwrap().to_be_bytes(), // second u32[] elements slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ad0", 16).unwrap().to_be_bytes(), // third u32[] header slot
        U256::from_str_radix("4aee6d38ad948303a0117a3e3deee4d912b62481681bd892442a7d720eee5d2c", 16).unwrap().to_be_bytes(), // third u32[] elements slot

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // 0x03 (u64[][] header) slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(), // first u64[] header slot
        U256::from_str_radix("2584db4a68aa8b172f70bc04e2e74541617c003374de6eb4b295e823e5beab01", 16).unwrap().to_be_bytes(), // first u64[] elements slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c", 16).unwrap().to_be_bytes(), // second u64[] header slot (empty vector)
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85d", 16).unwrap().to_be_bytes(), // third u64[] header slot
        U256::from_str_radix("3f8a9ffd58db029f2bac46056dbc53052839d91105f501f2db6ecb9566ee6832", 16).unwrap().to_be_bytes(), // third u64[] elements slot

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // 0x04 (u128[][] header) slot
        U256::from_str_radix("8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b", 16).unwrap().to_be_bytes(), // u128[] header slot
        U256::from_str_radix("c167b0e3c82238f4f2d1a50a8b3a44f96311d77b148c30dc0ef863e1a060dcb6", 16).unwrap().to_be_bytes(), // u128[] elements slot #1
        U256::from_str_radix("c167b0e3c82238f4f2d1a50a8b3a44f96311d77b148c30dc0ef863e1a060dcb7", 16).unwrap().to_be_bytes(), // u128[] elements slot #2
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000003dbcd75f64945893", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // u32[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // first u8[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000030201", 16).unwrap().to_be_bytes(), // first u8[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // second u8[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000504", 16).unwrap().to_be_bytes(), // second u8[] elements

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // u32[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // first u32[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000700000006", 16).unwrap().to_be_bytes(), // first u32[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // second u32[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000008", 16).unwrap().to_be_bytes(), // second u32[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // third u32[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000a00000009", 16).unwrap().to_be_bytes(), // third u32[] elements

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // u64[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // first u64[] len
        U256::from_str_radix("000000000000000e000000000000000d000000000000000c000000000000000b", 16).unwrap().to_be_bytes(), // first u64[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(), // second u64[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // third u64[] len
        U256::from_str_radix("000000000000000000000000000000000000000000000010000000000000000f", 16).unwrap().to_be_bytes(), // third u64[] elements

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // u128[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // first u128[] len
        U256::from_str_radix("0000000000000000000000000000001200000000000000000000000000000011", 16).unwrap().to_be_bytes(), // u128[] elements #1
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000013", 16).unwrap().to_be_bytes(), // u128[] elements #2
    ],
        readDynamicStruct3Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        DynamicStruct3 {
           id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: vec![vec![1, 2, 3], vec![4, 5]],
           b: vec![vec![6, 7], vec![8], vec![9, 10]],
           c: vec![vec![11, 12, 13, 14], vec![], vec![15, 16]],
           d: vec![vec![17, 18, 19]],
        }
    )]
#[case(saveDynamicStruct4Call::new((
        vec![1, 2, 3],
        47,
        123,
        address!("1111111111111111111111111111111111111111"),
    )),
    vec![
        // Field uid
        [0x00; 32], // 0x0
        // Field a: DynamicNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Header slot
        // First element
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b5d9d894133a730aa651ef62d26b0ffa846233c74177a591a4a896adfda97d22", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7", 16).unwrap().to_be_bytes(), // u128
        // Second element
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf8", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b32787652f8eacc66cda8b4b73a1b9c31381474fe9e723b0ba866bfbd5dde02b", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf9", 16).unwrap().to_be_bytes(), // u128

        // Field b: StaticNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // Header slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // First element
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // Second element
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ad0", 16).unwrap().to_be_bytes(), // Third element
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000ebebcbd6455c0890", 16).unwrap().to_be_bytes(),
        // Field a: DynamicNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        // First element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000000000000000000007b", 16).unwrap().to_be_bytes(),
        // Second element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000000000000000000007c", 16).unwrap().to_be_bytes(),
        // Field b: StaticNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000001111111111111111111111111111111111111111000000000000002f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000011111111111111111111111111111111111111110000000000000030", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000011111111111111111111111111111111111111110000000000000031", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct4Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        DynamicStruct4 {
        id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: vec![DynamicNestedStructChild { a: vec![1, 2, 3], b: 123 }, DynamicNestedStructChild { a: vec![1, 2, 3], b: 124 }],
           b: vec![StaticNestedStructChild { d: 47, e: address!("0x1111111111111111111111111111111111111111") }, StaticNestedStructChild { d: 48, e: address!("0x1111111111111111111111111111111111111111") }, StaticNestedStructChild { d: 49, e: address!("0x1111111111111111111111111111111111111111") }],
        }
    )]
#[case(saveDynamicStruct5Call::new((
        1,
        42,
        123,
        address!("0x1111111111111111111111111111111111111111"),
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Header slot
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000f9c87e6942ed1f5d", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct5Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        DynamicStruct5 {
        id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
           a: vec![
               NestedStructChildWrapper {
                   a: vec![
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 123 },
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 124 }
                   ],
                   b: vec![
                       StaticNestedStructChild { d: 42, e: address!("0x1111111111111111111111111111111111111111") },
                       StaticNestedStructChild { d: 43, e: address!("0x1111111111111111111111111111111111111111") },
                       StaticNestedStructChild { d: 44, e: address!("0x1111111111111111111111111111111111111111") }
                   ]
               },
               NestedStructChildWrapper {
                   a: vec![
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 125 },
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 126 },
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 127 }
                   ],
                   b: vec![
                       StaticNestedStructChild { d: 45, e: address!("0x1111111111111111111111111111111111111111") },
                       StaticNestedStructChild { d: 46, e: address!("0x1111111111111111111111111111111111111111") },
                   ]
               }
           ],
        }
    )]
#[case(saveGenericStruct32Call::new((
        1,
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // uint32 b
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000e926c0e2f5491027", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // Header slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(), // First element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Second element
    ],
        readGenericStruct32Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
        GenericStruct32 {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into() } },
            a: vec![1, 2, 3],
            b: 1,
        }
    )]
fn test_dynamic_fields<T: SolCall, U: SolCall, V: SolValue>(
    #[with("storage_encoding", "tests/storage/move_sources/encoding.move")] runtime: RuntimeSandbox,
    #[case] call_data_encode: T,
    #[case] expected_slots: Vec<[u8; 32]>,
    #[case] expected_encode: Vec<[u8; 32]>,
    #[case] call_data_decode: U,
    #[case] expected_decode: V,
) {
    let (result, _) = runtime
        .call_entrypoint(call_data_encode.abi_encode())
        .unwrap();
    assert_eq!(0, result);

    // Check if it is encoded correctly in storage
    for (i, slot) in expected_slots.iter().enumerate() {
        let storage = runtime.get_storage_at_slot(*slot);
        assert_eq!(expected_encode[i], storage, "Mismatch at slot {i}");
    }

    // Use the read function to check if it decodes correctly
    let (result, result_data) = runtime
        .call_entrypoint(call_data_decode.abi_encode())
        .unwrap();
    assert_eq!(0, result);
    assert_eq!(expected_decode.abi_encode(), result_data);
}

#[rstest]
#[case(saveFooCall::new(()),
    vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("544b730dcadfbf3c87d176fbcee0c1f462952c8bc9747841d1bfff2c9f84c07d", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000065c4a544c2e5b9f0a9", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("00000000000000000000000000000000000000000000002a7d4b6c5ec9959670", 16).unwrap().to_be_bytes(),
    ],
        readFooCall::new((U256::from_le_bytes(hex!("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7")),)),
        Foo {
            id: UID { id: ID { bytes: U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().into()  } },
            a: 101,
            b: Bar {
                id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into()  } },
                a: 42,
            },
            c: 102,
        }
    )]
#[case(saveMegaFooCall::new(()),
    vec![
        // MegaFoo
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        // Foo
        U256::from_str_radix("41ce687bc1e261a2e85acd0ef77dd1988f72f509c308effe56ce774de82de154", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("41ce687bc1e261a2e85acd0ef77dd1988f72f509c308effe56ce774de82de155", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("41ce687bc1e261a2e85acd0ef77dd1988f72f509c308effe56ce774de82de156", 16).unwrap().to_be_bytes(),
        //Bar
        U256::from_str_radix("544b730dcadfbf3c87d176fbcee0c1f462952c8bc9747841d1bfff2c9f84c07d", 16).unwrap().to_be_bytes(),
    ],
    vec![
        // MegaFoo
        U256::from_str_radix("00000000000000000000000000000000000000000000004d07b0556aa8b8d2d2", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000058", 16).unwrap().to_be_bytes(),
        // Foo
        U256::from_str_radix("000000000000000000000000000000000000000000000065c4a544c2e5b9f0a9", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),
        // Bar
        U256::from_str_radix("00000000000000000000000000000000000000000000002a7d4b6c5ec9959670", 16).unwrap().to_be_bytes(),

    ],
        readMegaFooCall::new((U256::from_le_bytes(hex!("79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09")),)),
    MegaFoo {
            id: UID { id: ID { bytes: U256::from_str_radix("79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09", 16).unwrap().into()  } },
            a: 77,
            b: Foo {
                id: UID { id: ID { bytes: U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().into()  } },
                a: 101,
                b: Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into()  } },
                    a: 42,
                },
                c: 102,
            },
            c: 88,
        }
    )]
#[
        case(saveVarCall::new(()),
        vec![
            // Var
            [0x00; 32],
            U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
            //Bar
            U256::from_str_radix("634e0cfe4d3eccb1f12a03ba6ba3b01bd270c3c2c5b79677ad2457cdaf0f0a31", 16).unwrap().to_be_bytes(),
            // Foo
            U256::from_str_radix("c17378e604db2bc240aa6a3925e1a9ff01f240512daf5ebf77e81574fe46b1dc", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("c17378e604db2bc240aa6a3925e1a9ff01f240512daf5ebf77e81574fe46b1dd", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("c17378e604db2bc240aa6a3925e1a9ff01f240512daf5ebf77e81574fe46b1de", 16).unwrap().to_be_bytes(),
            //Bar in Foo
            U256::from_str_radix("569ec9813e0e506fe3c07267d57c7d60af218b1971df8a17e8c3d9422ee45112", 16).unwrap().to_be_bytes(),
            // Bar vector
            U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85d", 16).unwrap().to_be_bytes(),

            U256::from_str_radix("d23d7ae789a511af9316daeb224298ce268bff3b0086cd9cc109986d5c6866c8", 16).unwrap().to_be_bytes(),

            U256::from_str_radix("eb6730eee37055d961becf7da68a370e7d01e385e23eccc77adff27323431635", 16).unwrap().to_be_bytes(),

            U256::from_str_radix("00721786a36420c69f024f5947485b51f91128b6a3167578dd192e106df958cf", 16).unwrap().to_be_bytes(),
        ],
        vec![
            // Var
            U256::from_str_radix("000000000000000000000000000000000000000000000000c208b5ff54db6c91", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("0f10fee34b569ef88274c8700225c115c5bc8e1db0ffddd1133715912144d3ee", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
            // Bar
            U256::from_str_radix("00000000000000000000000000000000000000000000002a7d4b6c5ec9959670", 16).unwrap().to_be_bytes(),
            // Foo
            U256::from_str_radix("000000000000000000000000000000000000000000000065c4a544c2e5b9f0a9", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),
            // Bar in Foo
            U256::from_str_radix("0000000000000000000000000000000000000000000000297d4b6c5ec9959670", 16).unwrap().to_be_bytes(),
            // Bar vector
            U256::from_str_radix("79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("12b23b08610619d2c73d9c594768afa7bcc248bd34e1f202173e5c92014ae02e", 16).unwrap().to_be_bytes(),
            U256::from_str_radix("b082f003cf7e89a005efbd95cd08519ae08b6e8e31de5fed37659f47fc64181d", 16).unwrap().to_be_bytes(),

            U256::from_str_radix("00000000000000000000000000000000000000000000002b7d4b6c5ec9959670", 16).unwrap().to_be_bytes(),

            U256::from_str_radix("00000000000000000000000000000000000000000000002c7d4b6c5ec9959670", 16).unwrap().to_be_bytes(),

            U256::from_str_radix("00000000000000000000000000000000000000000000002d7d4b6c5ec9959670", 16).unwrap().to_be_bytes(),

        ],
        readVarCall::new((U256::from_le_bytes(hex!("8148947c60769a1ac082a29bf80e4ff473e568ad39ff9bc45c3144244974525f")),)),
            Var {
            id: UID { id: ID { bytes: U256::from_str_radix("8148947c60769a1ac082a29bf80e4ff473e568ad39ff9bc45c3144244974525f", 16).unwrap().into()  } },
            a: Bar {
                id: UID { id: ID { bytes: U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().into()  } },
                a: 42,
            },
            b: Foo {
                id: UID { id: ID { bytes: U256::from_str_radix("0f10fee34b569ef88274c8700225c115c5bc8e1db0ffddd1133715912144d3ee", 16).unwrap().into()  } },
                a: 101,
                b: Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into()  } },
                    a: 41,
                },
                c: 102,
            },
            c: vec![
                Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09", 16).unwrap().into()  } },
                    a: 43,
                },
                Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("12b23b08610619d2c73d9c594768afa7bcc248bd34e1f202173e5c92014ae02e", 16).unwrap().into()  } },
                    a: 44,
                },
                Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("b082f003cf7e89a005efbd95cd08519ae08b6e8e31de5fed37659f47fc64181d", 16).unwrap().into()  } },
                    a: 45,
                }
            ],
        }
    )]
#[case(saveGenericWrapper32Call::new(()),
    vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("e7e785c40b41016ba8a2c189cbdbaa2cd93428804f2352d2d6e24604a35cbeb5", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("e7e785c40b41016ba8a2c189cbdbaa2cd93428804f2352d2d6e24604a35cbeb6", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("e7e785c40b41016ba8a2c189cbdbaa2cd93428804f2352d2d6e24604a35cbeb7", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("0922fff1cd0697e05be30fd001a86b5e89506d7c8304ebb077dc95f3791d7e86", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000657540b759386a85d0", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("000000000000000000000000000000000000000000000000e926c0e2f5491027", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000000000000000000000000000000000000000004d2", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("000000000000000000000000000000000000000000000063000000580000004d", 16).unwrap().to_be_bytes(),
    ],
    readGenericWrapper32Call::new((U256::from_le_bytes(hex!("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb")),)),
    GenericWrapper32 {
            id: UID { id: ID { bytes: U256::from_str_radix("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb", 16).unwrap().into()  } },
            a: 101,
            b: GenericStruct32 {
                id: UID { id: ID { bytes: U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().into()  } },
                a: vec![77, 88, 99],
                b: 1234,
            },
            c: 102,
        }
    )]
fn test_wrapped_objects<T: SolCall, U: SolCall, V: SolValue>(
    #[with("storage_encoding", "tests/storage/move_sources/encoding.move")] runtime: RuntimeSandbox,
    #[case] call_data_encode: T,
    #[case] expected_slots: Vec<[u8; 32]>,
    #[case] expected_encode: Vec<[u8; 32]>,
    #[case] call_data_decode: U,
    #[case] expected_decode: V,
) {
    let (result, _) = runtime
        .call_entrypoint(call_data_encode.abi_encode())
        .unwrap();
    assert_eq!(0, result);

    // Check if it is encoded correctly in storage
    for (i, slot) in expected_slots.iter().enumerate() {
        let storage = runtime.get_storage_at_slot(*slot);
        assert_eq!(expected_encode[i], storage, "Mismatch at slot {i}");
    }

    // println!("{:?}", call_data_decode.abi_encode());
    // Use the read function to check if it decodes correctly
    let (result, result_data) = runtime
        .call_entrypoint(call_data_decode.abi_encode())
        .unwrap();
    assert_eq!(0, result);
    assert_eq!(expected_decode.abi_encode(), result_data);
}

#[rstest]
#[case(saveBarStructCall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("e7e785c40b41016ba8a2c189cbdbaa2cd93428804f2352d2d6e24604a35cbeb5", 16).unwrap().to_be_bytes(),

    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000004e3a3639944ad360", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000002a01000000000000006300000058004d01", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000010000000000000000000000000000002b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000000000000000000006f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000000016345785d89ffff", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000000000000000000000000020106dce3d30b7932fe", 16).unwrap().to_be_bytes(),

    ],)]
#[case(saveFooAStructACall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("00000000000000000000000000000000000000002b002a00181038ad7354ad5b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],)]
#[case(saveFooAStructBCall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000002a01181038ad7354ad5b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000010000000000000000000000000000002b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],)]
#[case(saveFooAStructCCall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000020102181038ad7354ad5b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],)]
#[case(saveFooBStructACall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("00002a00cafecafecafecafecafecafecafecafecafecafe097b7c37e842ee57", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("01002d0000002c0000000000000000000000000000000000000000000000002b", 16).unwrap().to_be_bytes(),
    ],)]
#[case(saveFooBStructBCall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("00000001cafecafecafecafecafecafecafecafecafecafe097b7c37e842ee57", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00002d0000002c010000000000000000000000000000002b000000000000002a", 16).unwrap().to_be_bytes(),
    ],)]
#[case(saveFooBStructCCall::new(()),
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("00020102cafecafecafecafecafecafecafecafecafecafe097b7c37e842ee57", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00002d0000002c00000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
    ],)]
fn test_structs_with_enums<T: SolCall>(
    #[with("storage_encoding", "tests/storage/move_sources/encoding.move")] runtime: RuntimeSandbox,
    #[case] call_data_encode: T,
    #[case] expected_slots: Vec<[u8; 32]>,
    #[case] expected_encode: Vec<[u8; 32]>,
) {
    let (result, _) = runtime
        .call_entrypoint(call_data_encode.abi_encode())
        .unwrap();
    runtime.print_storage();

    assert_eq!(0, result);

    // Check if it is encoded correctly in storage
    for (i, slot) in expected_slots.iter().enumerate() {
        let storage = runtime.get_storage_at_slot(*slot);
        assert_eq!(expected_encode[i], storage, "Mismatch at slot {i}");
    }
}
