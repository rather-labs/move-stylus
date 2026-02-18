module 0x01::signer_type;

use std::signer::address_of;

entry fun echo_address(x: signer): address {
    address_of(&x)
}