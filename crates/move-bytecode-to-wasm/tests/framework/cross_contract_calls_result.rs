use crate::common::run_test;
use crate::common::runtime;
use alloy_primitives::{Address, U256, address, hex, keccak256};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::{
    constants::{
        BLOCK_BASEFEE, BLOCK_GAS_LIMIT, BLOCK_NUMBER, BLOCK_TIMESTAMP, GAS_PRICE,
        MSG_SENDER_ADDRESS, MSG_VALUE,
    },
    wasm_runner::{CrossContractCallType, ExecutionData, RuntimeSandbox},
};
use rstest::rstest;
const GET_RESULT_ERROR_CODE: &str = "101";
const DATA_ABORT_MESSAGE_PTR_OFFSET: usize = 256;

sol!(
    #[allow(missing_docs)]
    struct Baz {
        uint16 a;
        uint128 b;
    }

    struct Foo {
        address q;
        bool t;
        uint8 u;
        uint16 v;
        uint32 w;
        uint64 x;
        uint128 y;
        uint256 z;
        Baz baz;
    }

    struct Bazz {
        uint16 a;
        uint256[] b;
    }

    struct Bar {
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
        Bazz bazz;
        Baz baz;
    }

    function ccCallViewRes1(address contract_address) external returns (uint64);
    function ccCallViewRes2(address contract_address) external returns (Foo);
    function ccCallViewRes3(address contract_address) external returns (Bar);
    function ccCallViewRes4(address contract_address) external returns (uint8[]);
    function ccCallPureRes1(address contract_address) external returns (uint64);
    function ccCallPureRes2(address contract_address) external returns (Foo);
    function ccCallPureRes3(address contract_address) external returns (Bar);
    function ccCallPureRes4(address contract_address) external returns (uint8[]);
    function ccCallViewPureRes1(address contract_address) external returns (uint64);
    function ccCallViewPureRes2(address contract_address) external returns (Foo);
    function ccCallViewPureRes3(address contract_address) external returns (Bar);
    function ccCallViewPureRes4(address contract_address) external returns (uint8[]);
    function ccCall1(address contract_address) external returns (uint64);
    function ccCall2(address contract_address) external returns (Foo);
    function ccCall3(address contract_address) external returns (Bar);
    function ccCall4(address contract_address) external returns (uint8[]);
    function ccCall1WithGas(address contract_address, uint64 gas) external returns (uint64);
    function ccCall2WithGas(address contract_address, uint64 gas) external returns (Foo);
    function ccCall3WithGas(address contract_address, uint64 gas) external returns (Bar);
    function ccCall4WithGas(address contract_address, uint64 gas) external returns (uint8[]);
    function ccCall1WithGasDelegate(address contract_address, uint64 gas) external returns (uint64);
    function ccCall2WithGasDelegate(address contract_address, uint64 gas) external returns (Foo);
    function ccCall3WithGasDelegate(address contract_address, uint64 gas) external returns (Bar);
    function ccCall4WithGasDelegate(address contract_address, uint64 gas) external returns (uint8[]);
    function ccCall1WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (uint64);
    function ccCall2WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (Foo);
    function ccCall3WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (Bar);
    function ccCall4WithGasPayable(address contract_address, uint64 gas, uint256 value) external returns (uint8[]);
    function ccCall1Payable(address contract_address, uint256 value) external returns (uint64);
    function ccCall2Payable(address contract_address, uint256 value) external returns (Foo);
    function ccCall3Payable(address contract_address, uint256 value) external returns (Bar);
    function ccCall4Payable(address contract_address, uint256 value) external returns (uint8[]);
    function ccCall1Delegate(address contract_address) external returns (uint64);
    function ccCall2Delegate(address contract_address) external returns (Foo);
    function ccCall3Delegate(address contract_address) external returns (Bar);
    function ccCall4Delegate(address contract_address) external returns (uint8[]);
    function ccCall1WithArgs(address contract_address, uint64) external returns (uint64);
    function ccCall2WithArgs(address contract_address, uint256 value, uint64, Foo) external returns (Foo);
    function ccCall3WithArgs(address contract_address, uint64 gas, uint64, Foo, Bar) external returns (Bar);
    function ccCall4WithArgs(address contract_address, uint64, Foo, Bar, uint8[]) external returns (uint8[]);

    // The following functions are used to obtain their calldata and compare them
    function callView1() external;
    function callView2() external;
    function callView3() external;
    function callView4() external;
    function callPure1() external;
    function callPure2() external;
    function callPure3() external;
    function callPure4() external;
    function callViewPure1() external;
    function callViewPure2() external;
    function callViewPure3() external;
    function callViewPure4() external;
    function call1() external;
    function call2() external;
    function call3() external;
    function call4() external;
    function call1Payable() external;
    function call2Payable() external;
    function call3Payable() external;
    function call4Payable() external;
    function call1WithArgs(uint64) external;
    function call2WithArgs(uint64,Foo) external;
    function call3WithArgs(uint64,Foo,Bar) external;
    function call4WithArgs(uint64,Foo,Bar,uint8[]) external;
);

const ADDRESS: alloy_primitives::Address = address!("0xbeefbeef00000000000000000000000000007357");

fn get_foo() -> Foo {
    Foo {
        q: address!("0xcafe000000000000000000000000000000007357"),
        t: true,
        u: 255,
        v: u16::MAX,
        w: u32::MAX,
        x: u64::MAX,
        y: u128::MAX,
        z: U256::MAX,
        baz: Baz { a: 42, b: 4242 },
    }
}

fn get_bar() -> Bar {
    Bar {
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
        bazz: Bazz {
            a: 42,
            b: vec![U256::MAX, U256::from(8), U256::from(7), U256::from(6)],
        },
        baz: Baz {
            a: 111,
            b: 1111111111,
        },
    }
}

#[rstest]
#[case(
        ccCallViewRes1Call::new((ADDRESS,)),
        callView1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCallViewRes2Call::new((ADDRESS,)),
        callView2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCallViewRes3Call::new((ADDRESS,)),
        callView3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
#[case(
        ccCallViewRes4Call::new((ADDRESS,)),
        callView4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCallPureRes1Call::new((ADDRESS,)),
        callPure1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCallPureRes2Call::new((ADDRESS,)),
        callPure2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCallPureRes3Call::new((ADDRESS,)),
        callPure3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
#[case(
        ccCallPureRes4Call::new((ADDRESS,)),
        callPure4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCallViewPureRes1Call::new((ADDRESS,)),
        callViewPure1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCallViewPureRes2Call::new((ADDRESS,)),
        callViewPure2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCallViewPureRes3Call::new((ADDRESS,)),
        callViewPure3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
#[case(
        ccCallViewPureRes4Call::new((ADDRESS,)),
        callViewPure4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1Call::new((ADDRESS,)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2Call::new((ADDRESS,)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3Call::new((ADDRESS,)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4Call::new((ADDRESS,)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1DelegateCall::new((ADDRESS,)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2DelegateCall::new((ADDRESS,)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3DelegateCall::new((ADDRESS,)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4DelegateCall::new((ADDRESS,)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1WithGasCall::new((ADDRESS, 1)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        1,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2WithGasCall::new((ADDRESS, 2)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        2,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3WithGasCall::new((ADDRESS, 3)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        3,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4WithGasCall::new((ADDRESS, 4)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        4,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1WithGasDelegateCall::new((ADDRESS, 1)),
        call1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        1,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2WithGasDelegateCall::new((ADDRESS, 2)),
        call2Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        2,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3WithGasDelegateCall::new((ADDRESS, 3)),
        call3Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        3,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4WithGasDelegateCall::new((ADDRESS, 4)),
        call4Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        4,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1WithGasPayableCall::new((ADDRESS, 1, U256::from(u16::MAX))),
        call1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        1,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2WithGasPayableCall::new((ADDRESS, 2, U256::from(u32::MAX))),
        call2PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        2,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3WithGasPayableCall::new((ADDRESS, 3, U256::from(u64::MAX))),
        call3PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        3,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4WithGasPayableCall::new((ADDRESS, 4, U256::from(u128::MAX))),
        call4PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u128::MAX),
        4,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1PayableCall::new((ADDRESS, U256::from(u16::MAX))),
        call1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2PayableCall::new((ADDRESS, U256::from(u32::MAX))),
        call2PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3PayableCall::new((ADDRESS, U256::from(u64::MAX))),
        call3PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        u64::MAX,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4PayableCall::new((ADDRESS, U256::from(u128::MAX))),
        call4PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u128::MAX),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
#[case(
        ccCall1WithArgsCall::new((ADDRESS, 84)),
        call1WithArgsCall::new((84,)).abi_encode(),
        true,
        CrossContractCallType::StaticCall,
        U256::from(0),
        u64::MAX,
        42_u64.abi_encode(),
    )]
#[case(
        ccCall2WithArgsCall::new((ADDRESS, U256::from(u32::MAX), 84, get_foo())),
        call2WithArgsCall::new((84, get_foo())).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        u64::MAX,
        get_foo().abi_encode(),
    )]
#[case(
        ccCall3WithArgsCall::new((ADDRESS, 3, 84, get_foo(), get_bar())),
        call3WithArgsCall::new((84, get_foo(), get_bar())).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        3,
        get_bar().abi_encode(),
    )]
#[case(
        ccCall4WithArgsCall::new((ADDRESS, 84, get_foo(), get_bar(), vec![1, 2, 3, 4, 5])),
        call4WithArgsCall::new((84, get_foo(), get_bar(), vec![1, 2, 3, 4, 5])).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
        vec![3, 1, 4, 1, 5].abi_encode(),
    )]
fn test_cross_contract_call_with_result_calls<T: SolCall>(
    #[with("cross_contract_calls_result", "tests/framework/move_sources")] runtime: RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_cross_contract_calldata: Vec<u8>,
    #[case] success: bool,
    #[case] expected_call_type: CrossContractCallType,
    #[case] expected_payable_value: U256,
    #[case] expected_gas: u64,
    #[case] expected_result: Vec<u8>,
) {
    runtime.set_cross_contract_call_success(success);
    runtime.set_cross_contract_return_data(expected_result.clone());

    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(0, result);

    if success {
        assert_eq!(return_data, expected_result);

        let result = runtime.cross_contract_calls.lock().unwrap().recv().unwrap();
        assert_eq!(expected_call_type, result.call_type);
        assert_eq!(ADDRESS, Address::from(result.address));
        assert_eq!(expected_gas, result.gas);
        assert_eq!(expected_cross_contract_calldata, result.calldata);
        assert_eq!(expected_payable_value, result.value);
    } else {
        assert_eq!(false.abi_encode(), return_data);
    }
}

#[rstest]
#[case(ccCallViewRes1Call::new((ADDRESS,)))]
#[case(ccCallViewRes2Call::new((ADDRESS,)))]
#[case(ccCallViewRes3Call::new((ADDRESS,)))]
#[case(ccCallViewRes4Call::new((ADDRESS,)))]
#[case(ccCallPureRes1Call::new((ADDRESS,)))]
#[case(ccCallPureRes2Call::new((ADDRESS,)))]
#[case(ccCallPureRes3Call::new((ADDRESS,)))]
#[case(ccCallPureRes4Call::new((ADDRESS,)))]
#[case(ccCallViewPureRes1Call::new((ADDRESS,)))]
#[case(ccCallViewPureRes2Call::new((ADDRESS,)))]
#[case(ccCallViewPureRes3Call::new((ADDRESS,)))]
#[case(ccCallViewPureRes4Call::new((ADDRESS,)))]
#[case(ccCall1Call::new((ADDRESS,)))]
#[case(ccCall2Call::new((ADDRESS,)))]
#[case(ccCall3Call::new((ADDRESS,)))]
#[case(ccCall4Call::new((ADDRESS,)))]
#[case(ccCall1DelegateCall::new((ADDRESS,)))]
#[case(ccCall2DelegateCall::new((ADDRESS,)))]
#[case(ccCall3DelegateCall::new((ADDRESS,)))]
#[case(ccCall4DelegateCall::new((ADDRESS,)))]
#[case(ccCall1WithGasCall::new((ADDRESS, 1)))]
#[case(ccCall2WithGasCall::new((ADDRESS, 2)))]
#[case(ccCall3WithGasCall::new((ADDRESS, 3)))]
#[case(ccCall4WithGasCall::new((ADDRESS, 4)))]
#[case(ccCall1WithGasDelegateCall::new((ADDRESS, 1)))]
#[case(ccCall2WithGasDelegateCall::new((ADDRESS, 2)))]
#[case(ccCall3WithGasDelegateCall::new((ADDRESS, 3)))]
#[case(ccCall4WithGasDelegateCall::new((ADDRESS, 4)))]
#[case(ccCall1WithGasPayableCall::new((ADDRESS, 1, U256::from(u16::MAX))))]
#[case(ccCall2WithGasPayableCall::new((ADDRESS, 2, U256::from(u32::MAX))))]
#[case(ccCall3WithGasPayableCall::new((ADDRESS, 3, U256::from(u64::MAX))))]
#[case(ccCall4WithGasPayableCall::new((ADDRESS, 4, U256::from(u128::MAX))))]
#[case(ccCall1PayableCall::new((ADDRESS, U256::from(u16::MAX))))]
#[case(ccCall2PayableCall::new((ADDRESS, U256::from(u32::MAX))))]
#[case(ccCall3PayableCall::new((ADDRESS, U256::from(u64::MAX))))]
#[case(ccCall4PayableCall::new((ADDRESS, U256::from(u128::MAX))))]
#[case(ccCall1WithArgsCall::new((ADDRESS, 84)))]
#[case(ccCall2WithArgsCall::new((ADDRESS, U256::from(u32::MAX), 84, get_foo())))]
#[case(ccCall3WithArgsCall::new((ADDRESS, 3, 84, get_foo(), get_bar())))]
#[case(ccCall4WithArgsCall::new((ADDRESS, 84, get_foo(), get_bar(), vec![1, 2, 3, 4, 5])))]
fn test_cross_contract_call_with_result_get_result_panic_if_fails<T: SolCall>(
    #[with("cross_contract_calls_result", "tests/framework/move_sources")] runtime: RuntimeSandbox,
    #[case] call_data: T,
) {
    runtime.set_cross_contract_call_success(false);
    let ExecutionData {
        return_data: _,
        instance,
        mut store,
        ..
    } = runtime
        .call_entrypoint_with_data(call_data.abi_encode())
        .unwrap();

    // Read where the encoded error is
    let error_ptr =
        RuntimeSandbox::read_memory_from(&instance, &mut store, DATA_ABORT_MESSAGE_PTR_OFFSET, 4)
            .unwrap();
    let error_ptr = u32::from_le_bytes(error_ptr.try_into().unwrap());

    // Read the actual message length from the ABI header (4 bytes big-endian at offset 68)
    let msg_len_bytes =
        RuntimeSandbox::read_memory_from(&instance, &mut store, error_ptr as usize + 68, 4)
            .unwrap();
    let msg_len = u32::from_be_bytes(msg_len_bytes.try_into().unwrap()) as usize;

    // Read the raw error message bytes (not ABI-encoded, just UTF-8 string bytes)
    let error_bytes = RuntimeSandbox::read_memory_from(
        &instance,
        &mut store,
        // raw error data (skip 4-byte length header + 68-byte ABI header)
        error_ptr as usize + 72,
        // Use the actual message length from the ABI header
        msg_len,
    )
    .unwrap();

    // Convert raw UTF-8 bytes to string
    let error = String::from_utf8(error_bytes).unwrap();

    assert_eq!(GET_RESULT_ERROR_CODE, error);
}
