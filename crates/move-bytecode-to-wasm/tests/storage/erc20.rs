use crate::common::runtime;
use alloy_primitives::{U256, address, keccak256};
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]

    struct String {
        uint8[] bytes;
    }


    function mint(address to, uint256 amount) external view;
    function create() public view;
    function burn(address from, uint256 amount) external view;
    function balanceOf(address address) public view returns (uint256);
    function totalSupply() external view returns (uint256);
    function transfer(address recipient, uint256 amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
    function name() external view returns (string);
    function symbol() external view returns (string);
    function decimals() external view returns (uint8);
);

#[rstest]
fn test_erc20(#[with("erc20", "tests/storage/move_sources/erc20.move")] runtime: RuntimeSandbox) {
    let address_1 = address!("0xcafecafecafecafecafecafecafecafecafecafe");
    runtime.set_msg_sender(**address_1);
    runtime.set_tx_origin(**address_1);
    let address_2 = address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef");
    let address_3 = address!("0xabcabcabcabcabcabcabcabcabcabcabcabcabca");

    // Create the contract
    let call_data = createCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Check frozen info
    let call_data = decimalsCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(18.abi_encode(), result_data);

    let call_data = nameCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!("Test Coin".abi_encode(), result_data);

    let call_data = symbolCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!("TST".abi_encode(), result_data);

    // Mint new coins
    let call_data = totalSupplyCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(0.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(0.abi_encode(), result_data);

    let call_data = mintCall::new((address_1, U256::from(9999999))).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = totalSupplyCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9999999.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9999999.abi_encode(), result_data);

    // Transfer
    let call_data = transferCall::new((address_2, U256::from(1111))).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(true.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9998888.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_2,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(1111.abi_encode(), result_data);

    // Burn
    let call_data = totalSupplyCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9999999.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9998888.abi_encode(), result_data);

    let call_data = burnCall::new((address_1, U256::from(2222))).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = totalSupplyCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9997777.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9996666.abi_encode(), result_data);

    // Burn more than the balance to trigger error
    let call_data = burnCall::new((address_1, U256::from(12345678))).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(1, result);
    let expected_data = [
        keccak256(b"Error(string)")[..4].to_vec(),
        <sol!((string,))>::abi_encode_params(&("Insufficient funds" as &str,)),
    ]
    .concat();
    assert_eq!(expected_data, result_data);

    // Allowance
    // Allow address_1 to spend 100 TST from address_2
    let call_data = allowanceCall::new((address_2, address_1)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(0.abi_encode(), result_data);

    runtime.set_msg_sender(**address_2);
    runtime.set_tx_origin(**address_2);
    let call_data = approveCall::new((address_1, U256::from(100))).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    runtime.set_msg_sender(**address_1);
    runtime.set_tx_origin(**address_1);
    let call_data = allowanceCall::new((address_2, address_1)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(100.abi_encode(), result_data);

    // Transfer from
    // Transfer from address_2 100 TST using address_1 to address_3
    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9996666.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_2,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(1111.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_3,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(0.abi_encode(), result_data);

    let call_data = transferFromCall::new((address_2, address_3, U256::from(100))).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = balanceOfCall::new((address_1,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(9996666.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_2,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(1011.abi_encode(), result_data);

    let call_data = balanceOfCall::new((address_3,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    assert_eq!(100.abi_encode(), result_data);
}
