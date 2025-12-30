use crate::common::runtime;
use alloy_primitives::{Address, U256, address};
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::wasm_runner::{CrossContractCallType, RuntimeSandbox};
use rstest::rstest;

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

    function ccCallEmptyRes1(address contract_address) external returns (bool);
    function ccCallEmptyRes2(address contract_address, uint64 v) external returns (bool);
    function ccCallEmptyRes3(address contract_address, Foo v) external returns (bool);
    function ccCallEmptyRes4(address contract_address, Bar v) external returns (bool);
    function ccCallEmptyRes5(address contract_address, uint8[] v) external returns (bool);
    function ccCallEmptyRes1WithGas(address contract_address, uint64 gas) external returns (bool);
    function ccCallEmptyRes2WithGas(address contract_address, uint64 gas, uint64 v) external returns (bool);
    function ccCallEmptyRes3WithGas(address contract_address, uint64 gas, Foo v) external returns (bool);
    function ccCallEmptyRes1Payable(address contract_address, uint256 payable_value) external returns (bool);
    function ccCallEmptyRes2Payable(address contract_address, uint256 payable_value, uint64 v) external returns (bool);
    function ccCallEmptyRes3Payable(address contract_address, uint256 payable_value, Foo v) external returns (bool);
    function ccCallEmptyRes4Payable(address contract_address, uint256 payable_value, Bar v) external returns (bool);
    function ccCallEmptyRes5Payable(address contract_address, uint256 payable_value, uint8[] v) external returns (bool);
    function ccCallEmptyRes1PayableGas(address contract_address, uint256 payable_value, uint64 gas) external returns (bool);
    function ccCallEmptyRes2PayableGas(address contract_address, uint256 payable_value, uint64 gas, uint64 v) external returns (bool);
    function ccCallEmptyRes3PayableGas(address contract_address, uint256 payable_value, uint64 gas, Foo v) external returns (bool);
    function ccCallEmptyRes1Delegate(address contract_address) external returns (bool);
    function ccCallEmptyRes2Delegate(address contract_address, uint64 v) external returns (bool);
    function ccCallEmptyRes3Delegate(address contract_address, Foo v) external returns (bool);
    function ccCallEmptyRes4Delegate(address contract_address, Bar v) external returns (bool);
    function ccCallEmptyRes5Delegate(address contract_address, uint8[] v) external returns (bool);
    function ccCallEmptyRes1WithGasDelegate(address contract_address, uint64 gas) external returns (bool);
    function ccCallEmptyRes2WithGasDelegate(address contract_address, uint64 gas, uint64 v) external returns (bool);
    function ccCallEmptyRes3WithGasDelegate(address contract_address, uint64 gas, Foo v) external returns (bool);

    // The following functions are used to obtain their calldata and compare them
    function callEmptyRes1() external;
    function callEmptyRes2(uint64 v) external;
    function callEmptyRes3(Foo v) external;
    function callEmptyRes4(Bar v) external;
    function callEmptyRes5(uint8[] v) external;
    function callEmptyRes1Payable() external;
    function callEmptyRes2Payable(uint64 v) external;
    function callEmptyRes3Payable(Foo v) external;
    function callEmptyRes4Payable(Bar v) external;
    function callEmptyRes5Payable(uint8[] v) external;
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
        ccCallEmptyRes1Call::new((ADDRESS,)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes2Call::new((ADDRESS, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes3Call::new((ADDRESS, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes4Call::new((ADDRESS, get_bar())),
        callEmptyRes4Call::new((get_bar(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes5Call::new((ADDRESS, vec![1,2,3,4,5])),
        callEmptyRes5Call::new((vec![1,2,3,4,5],)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes1WithGasCall::new((ADDRESS, 1)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        1,
    )]
#[case(
        ccCallEmptyRes2WithGasCall::new((ADDRESS, 2, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        2,
    )]
#[case(
        ccCallEmptyRes3WithGasCall::new((ADDRESS, 3, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(0),
        3,
    )]
#[case(
        ccCallEmptyRes1PayableCall::new((ADDRESS, U256::from(u16::MAX))),
        callEmptyRes1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes2PayableCall::new((ADDRESS, U256::from(u32::MAX), 42)),
        callEmptyRes2PayableCall::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes3PayableCall::new((ADDRESS, U256::from(u64::MAX), get_foo())),
        callEmptyRes3PayableCall::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes4PayableCall::new((ADDRESS, U256::from(u128::MAX), get_bar())),
        callEmptyRes4PayableCall::new((get_bar(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u128::MAX),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes5PayableCall::new((ADDRESS, U256::MAX, vec![1,2,3,4,5])),
        callEmptyRes5PayableCall::new((vec![1,2,3,4,5],)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::MAX,
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes1PayableGasCall::new((ADDRESS, U256::from(u16::MAX), 1)),
        callEmptyRes1PayableCall::new(()).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u16::MAX),
        1,
    )]
#[case(
        ccCallEmptyRes2PayableGasCall::new((ADDRESS, U256::from(u32::MAX), 2, 42)),
        callEmptyRes2PayableCall::new((42,)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u32::MAX),
        2,
    )]
#[case(
        ccCallEmptyRes3PayableGasCall::new((ADDRESS, U256::from(u64::MAX), 3, get_foo())),
        callEmptyRes3PayableCall::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::Call,
        U256::from(u64::MAX),
        3,
    )]
#[case(
        ccCallEmptyRes1DelegateCall::new((ADDRESS,)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes2DelegateCall::new((ADDRESS, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes3DelegateCall::new((ADDRESS, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes4DelegateCall::new((ADDRESS, get_bar())),
        callEmptyRes4Call::new((get_bar(),)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes5DelegateCall::new((ADDRESS, vec![1,2,3,4,5])),
        callEmptyRes5Call::new((vec![1,2,3,4,5],)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes1WithGasDelegateCall::new((ADDRESS, 1)),
        callEmptyRes1Call::new(()).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        1,
    )]
#[case(
        ccCallEmptyRes2WithGasDelegateCall::new((ADDRESS, 2, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        2,
    )]
#[case(
        ccCallEmptyRes3WithGasDelegateCall::new((ADDRESS, 3, get_foo())),
        callEmptyRes3Call::new((get_foo(),)).abi_encode(),
        true,
        CrossContractCallType::DelegateCall,
        U256::from(0),
        3,
    )]
// --
#[case(
        ccCallEmptyRes1Call::new((ADDRESS,)),
        callEmptyRes1Call::new(()).abi_encode(),
        false,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
#[case(
        ccCallEmptyRes2Call::new((ADDRESS, 42)),
        callEmptyRes2Call::new((42,)).abi_encode(),
        false,
        CrossContractCallType::Call,
        U256::from(0),
        u64::MAX,
    )]
fn test_cross_contract_call_empty_calls<T: SolCall>(
    #[with("cross_contract_calls", "tests/framework/move_sources")] runtime: RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_cross_contract_calldata: Vec<u8>,
    #[case] success: bool,
    #[case] expected_call_type: CrossContractCallType,
    #[case] expected_payable_value: U256,
    #[case] expected_gas: u64,
) {
    runtime.set_cross_contract_call_success(success);

    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(0, result);

    if success {
        assert_eq!(true.abi_encode(), return_data);

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
