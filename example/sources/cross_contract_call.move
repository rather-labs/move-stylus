module hello_world::cross_contract_call;

use erc20call::erc20call as erc20call;
use stylus::contract_calls as ccc;

entry fun balance_of_erc20(erc20_address: address, balance_address: address): u256 {
    let erc20 = erc20call::new(ccc::new(erc20_address));
    erc20.balance_of(balance_address).get_result()
}

entry fun total_supply(erc20_address: address): u256 {
    let erc20 = erc20call::new(ccc::new(erc20_address));
    erc20.total_supply().get_result()
}

entry fun transfer_from_erc20(
    erc20_address: address,
    sender: address,
    recipient: address,
    amount: u256,
): bool {
    let erc20 = erc20call::new(ccc::new(erc20_address));
    erc20.transfer_from(sender, recipient, amount).get_result()
}
