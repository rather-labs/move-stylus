use super::*;
use crate::common::runtime;
use alloy_primitives::{FixedBytes, U256, address, hex, keccak256};
use alloy_sol_types::sol;
use alloy_sol_types::{SolCall, SolValue};
use move_test_runner::constants::MSG_SENDER_ADDRESS;
use move_test_runner::constants::SIGNER_ADDRESS;
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

    struct OptionSword {
        Sword[] vec;
    }

    struct OptionShield {
        Shield[] vec;
    }

    struct Sword {
        UID id;
        uint8 strength;
    }

    struct Shield {
        UID id;
        uint8 armor;
    }

    struct Warrior {
        UID id;
        OptionSword sword;
        OptionShield shield;
        Faction faction;
    }

    enum Faction {
        Alliance,
        Horde,
        Rebel
    }

    function createWarrior() public view;
    function createSword(uint8 strength) public view;
    function createShield(uint8 armor) public view;
    function equipSword(bytes32 id, bytes32 sword) public;
    function equipShield(bytes32 id, bytes32 shield) public;
    function inspectWarrior(bytes32 id) public view returns (Warrior);
    function inspectSword(bytes32 id) public view returns (Sword);
    function inspectShield(bytes32 id) public view returns (Shield);
    function destroyWarrior(bytes32 id) public;
    function destroySword(bytes32 id) public;
    function changeFaction(bytes32 id, uint8 faction) public;
);

#[rstest]
fn test_equip_warrior(
    #[with("simple_warrior", "tests/storage/move_sources/simple_warrior.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_msg_sender(SIGNER_ADDRESS);

    // Create warrior
    let call_data = createWarriorCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let warrior_id = runtime.obtain_uid();

    // Inspect warrior and assert it has no sword or shield
    let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
    let expected_return_data = Warrior::abi_encode(&Warrior {
        id: UID {
            id: ID { bytes: warrior_id },
        },
        sword: OptionSword { vec: vec![] },
        shield: OptionShield { vec: vec![] },
        faction: Faction::Rebel,
    });
    assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
    assert_eq!(0, result);

    // Create sword
    let call_data = createSwordCall::new((66,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let sword_id = runtime.obtain_uid();
    let sword_slot = derive_object_slot(&SIGNER_ADDRESS, &sword_id.0);

    // Equip sword
    let call_data = equipSwordCall::new((warrior_id, sword_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Inspect warrior and assert it has the sword equiped
    let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
    let expected_return_data = Warrior::abi_encode(&Warrior {
        id: UID {
            id: ID { bytes: warrior_id },
        },
        sword: OptionSword {
            vec: vec![Sword {
                id: UID {
                    id: ID { bytes: sword_id },
                },
                strength: 66,
            }],
        },
        shield: OptionShield { vec: vec![] },
        faction: Faction::Rebel,
    });
    assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
    assert_eq!(0, result);

    // Assert that the original sword slot (under the sender's address) is now empty
    assert_eq!(runtime.get_storage_at_slot(sword_slot.0), [0u8; 32]);
    // Create new sword
    let call_data = createSwordCall::new((77,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let new_sword_id = runtime.obtain_uid();
    let new_sword_slot = derive_object_slot(&SIGNER_ADDRESS, &new_sword_id.0);

    // Equip new sword
    let call_data = equipSwordCall::new((warrior_id, new_sword_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Inspect warrior and assert it has the new sword equiped
    let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
    let expected_return_data = Warrior::abi_encode(&Warrior {
        id: UID {
            id: ID { bytes: warrior_id },
        },
        sword: OptionSword {
            vec: vec![Sword {
                id: UID {
                    id: ID {
                        bytes: new_sword_id,
                    },
                },
                strength: 77,
            }],
        },
        shield: OptionShield { vec: vec![] },
        faction: Faction::Rebel,
    });
    assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
    assert_eq!(0, result);

    // Assert that the original new sword slot (under the sender's address) is now empty
    assert_eq!(runtime.get_storage_at_slot(new_sword_slot.0), [0u8; 32]);

    // Assert that the original old sword slot (under the sender's address) holds the old sword now
    assert_ne!(runtime.get_storage_at_slot(sword_slot.0), [0u8; 32]);

    let call_data = inspectSwordCall::new((sword_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = inspectSwordCall::abi_decode_returns(&return_data).unwrap();
    let expected_return_data = Sword::abi_encode(&Sword {
        id: UID {
            id: ID { bytes: sword_id },
        },
        strength: 66,
    });
    assert_eq!(Sword::abi_encode(&return_data), expected_return_data);
    assert_eq!(0, result);

    // Create shield
    let call_data = createShieldCall::new((42,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let shield_id = runtime.obtain_uid();
    let shield_slot = derive_object_slot(&SIGNER_ADDRESS, &shield_id.0);

    let call_data = equipShieldCall::new((warrior_id, shield_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Inspect warrior and assert it has the shield equiped
    let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
    let expected_return_data = Warrior::abi_encode(&Warrior {
        id: UID {
            id: ID { bytes: warrior_id },
        },
        sword: OptionSword {
            vec: vec![Sword {
                id: UID {
                    id: ID {
                        bytes: new_sword_id,
                    },
                },
                strength: 77,
            }],
        },
        shield: OptionShield {
            vec: vec![Shield {
                id: UID {
                    id: ID { bytes: shield_id },
                },
                armor: 42,
            }],
        },
        faction: Faction::Rebel,
    });
    assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
    assert_eq!(0, result);

    // Assert that the original shield slot (under the sender's address) is now empty
    assert_eq!(runtime.get_storage_at_slot(shield_slot.0), [0u8; 32]);

    // Change faction
    let call_data = changeFactionCall::new((warrior_id, 0)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Inspect warrior and assert it has the new faction
    let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
    let expected_return_data = Warrior::abi_encode(&Warrior {
        id: UID {
            id: ID { bytes: warrior_id },
        },
        sword: OptionSword {
            vec: vec![Sword {
                id: UID {
                    id: ID {
                        bytes: new_sword_id,
                    },
                },
                strength: 77,
            }],
        },
        shield: OptionShield {
            vec: vec![Shield {
                id: UID {
                    id: ID { bytes: shield_id },
                },
                armor: 42,
            }],
        },
        faction: Faction::Alliance,
    });
    assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
    assert_eq!(0, result);

    let storage_before_destroy = runtime.get_storage();
    // Destroy warrior
    let call_data = destroyWarriorCall::new((warrior_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Destroy the old sword too, just to make the test simpler
    let call_data = destroySwordCall::new((sword_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_destroy = runtime.get_storage();

    // Assert that the storage is empty
    assert_empty_storage(&storage_before_destroy, &storage_after_destroy);
}
