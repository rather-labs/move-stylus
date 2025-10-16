module hello_world::cross_contract_call;

use stylus::contract_calls::new_contract_call;

use erc20call::erc20call::{ERC20Call, new};

entry fun test(): u256 {
    let erc20 = new(@0x3, false);
    erc20.balance_of(@0x5).get_result()
}
