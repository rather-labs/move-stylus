module hello_world::cross_contract_call;

use erc20call::erc20call::{ERC20Call, new};

entry fun balance_of_erc20(erc20_address: address, balance_address: address): u256 {
    let erc20 = new(erc20_address, false);
    erc20.balance_of(balance_address).get_result()
}
