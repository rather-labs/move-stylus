use crate::common::runtime;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::constants::MSG_SENDER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

use super::*;

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

    struct Alpha {
        UID id;
        uint64 value;
    }

    struct Beta {
        UID id;
        Alpha a;
    }

    struct Gamma {
        UID id;
        Beta a;
    }

    struct Delta {
        UID id;
        Alpha[] a;
    }

    struct Epsilon {
        UID id;
        Delta[] a;
    }
    struct Zeta {
        UID id;
        Astra b;
    }

    struct Astra {
        Alpha[] a;
    }

    struct Eta {
        UID id;
        Bora b;
    }

    struct Bora {
        uint64[] a;
        uint64[][] b;
    }
    function createAlpha(uint64 value) public view;
    function createBeta() public view;
    function createGamma() public view;
    function createDelta() public view;
    function createEmptyDelta() public view;
    function createEpsilon() public view;
    function createEmptyZeta() public view;
    function createBetaTto(bytes32 a) public view;
    function createGammaTto(bytes32 a) public view;
    function createDeltaTto(bytes32 a, bytes32 b) public view;
    function createEpsilonTto(bytes32 a, bytes32 b) public view;
    function createEta() public view;
    function readAlpha(bytes32 a) public view returns (Alpha);
    function readBeta(bytes32 b) public view returns (Beta);
    function readGamma(bytes32 g) public view returns (Gamma);
    function readDelta(bytes32 d) public view returns (Delta);
    function readEpsilon(bytes32 e) public view returns (Epsilon);
    function readZeta(bytes32 z) public view returns (Zeta);
    function readEta(bytes32 id) public view returns (Eta);
    function deleteAlpha(bytes32 a) public view;
    function deleteBeta(bytes32 b) public view;
    function deleteGamma(bytes32 g) public view;
    function deleteDelta(bytes32 d) public view;
    function deleteZeta(bytes32 z) public view;
    function deleteEpsilon(bytes32 e) public view;
    function transferBeta(bytes32 b, address recipient) public view;
    function transferGamma(bytes32 g, address recipient) public view;
    function transferDelta(bytes32 d, address recipient) public view;
    function transferZeta(bytes32 z, address recipient) public view;
    function rebuildGamma(bytes32 g, address recipient) public view;
    function destructDeltaToBeta(bytes32 d) public view;
    function pushAlphaToDelta(bytes32 d, bytes32 a) public view;
    function popAlphaFromDelta(bytes32 d) public view;
    function destructEpsilon(bytes32 e, bytes32 a) public view;
    function pushAlphaToZeta(bytes32 z, bytes32 a) public view;
    function popAlphaFromZeta(bytes32 z) public view;
    function pushToBora(bytes32 e, uint64 v) public view;
    function popFromBora(bytes32 e) public returns (uint64, uint64[]);
);

// In all tests, we use the tto flag to indicate if the creation method should take
// the object to be wrapped as argument or create it directly.
#[rstest]
#[case(false)]
#[case(true)]
fn test_creating_and_deleting_beta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
    #[case] tto: bool,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let (alpha_id, beta_id) = if tto {
        // Create alpha first for TTO method
        let call_data = createAlphaCall::new((102,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_id = runtime.obtain_uid().unwrap();

        // Create beta, passing alpha as argument to be wrapped in it
        let call_data = createBetaTtoCall::new((alpha_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let beta_id = runtime.obtain_uid().unwrap();

        (alpha_id, beta_id)
    } else {
        // Create beta directly
        let call_data = createBetaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Get the object ids
        let alpha_id = runtime.obtain_uid().unwrap();
        let beta_id = runtime.obtain_uid().unwrap();

        (alpha_id, beta_id)
    };

    // Read beta and assert the returned data
    let call_data = readBetaCall::new((beta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
    let expected_value = if tto { 102 } else { 101 };
    let beta_expected = Beta::abi_encode(&Beta {
        id: UID {
            id: ID { bytes: beta_id },
        },
        a: Alpha {
            id: UID {
                id: ID { bytes: alpha_id },
            },
            value: expected_value,
        },
    });
    assert_eq!(Beta::abi_encode(&return_data), beta_expected);
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete beta and assert the storage is empty afterwards
    let call_data = deleteBetaCall::new((beta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
#[case(false)]
#[case(true)]
fn test_creating_and_deleting_gamma(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
    #[case] tto: bool,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let (alpha_id, beta_id, gamma_id) = if tto {
        // Create beta first for TTO method
        let call_data = createBetaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_id = runtime.obtain_uid().unwrap();
        let beta_id = runtime.obtain_uid().unwrap();

        // Create gamma, passing beta as argument to be wrapped in it
        let call_data = createGammaTtoCall::new((beta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let gamma_id = runtime.obtain_uid().unwrap();

        (alpha_id, beta_id, gamma_id)
    } else {
        // Create gamma directly
        let call_data = createGammaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Get the object ids
        let alpha_id = runtime.obtain_uid().unwrap();
        let beta_id = runtime.obtain_uid().unwrap();
        let gamma_id = runtime.obtain_uid().unwrap();

        (alpha_id, beta_id, gamma_id)
    };

    // Read gamma and assert the returned data
    let call_data = readGammaCall::new((gamma_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readGammaCall::abi_decode_returns(&return_data).unwrap();
    let gamma_expected = Gamma::abi_encode(&Gamma {
        id: UID {
            id: ID { bytes: gamma_id },
        },
        a: Beta {
            id: UID {
                id: ID { bytes: beta_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_id },
                },
                value: 101,
            },
        },
    });
    assert_eq!(Gamma::abi_encode(&return_data), gamma_expected);
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete gamma and assert the storage is empty afterwards
    let call_data = deleteGammaCall::new((gamma_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
#[case(false)]
#[case(true)]
fn test_creating_and_deleting_delta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
    #[case] tto: bool,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let (alpha_1_id, alpha_2_id, delta_id) = if tto {
        // Create alphas first for TTO method
        let call_data = createAlphaCall::new((101,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid().unwrap();

        let call_data = createAlphaCall::new((102,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_2_id = runtime.obtain_uid().unwrap();

        // Create delta using TTO method
        let call_data = createDeltaTtoCall::new((alpha_1_id, alpha_2_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let delta_id = runtime.obtain_uid().unwrap();

        (alpha_1_id, alpha_2_id, delta_id)
    } else {
        // Create delta directly
        let call_data = createDeltaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid().unwrap();
        let alpha_2_id = runtime.obtain_uid().unwrap();
        let delta_id = runtime.obtain_uid().unwrap();

        (alpha_1_id, alpha_2_id, delta_id)
    };

    // Read delta and assert the returned data
    let call_data = readDeltaCall::new((delta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_id },
        },
        a: vec![
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            },
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_2_id },
                },
                value: 102,
            },
        ],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete delta and assert the storage is empty afterwards
    let call_data = deleteDeltaCall::new((delta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
#[case(false)]
#[case(true)]
fn test_creating_and_deleting_epsilon(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
    #[case] tto: bool,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let (alpha_1_id, alpha_2_id, alpha_3_id, alpha_4_id, delta_1_id, delta_2_id, epsilon_id) =
        if tto {
            let call_data = createAlphaCall::new((101,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_1_id = runtime.obtain_uid().unwrap();

            let call_data = createAlphaCall::new((102,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_2_id = runtime.obtain_uid().unwrap();

            // Create deltas first for TTO method
            let call_data = createDeltaTtoCall::new((alpha_1_id, alpha_2_id)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let delta_1_id = runtime.obtain_uid().unwrap();

            let call_data = createAlphaCall::new((103,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_3_id = runtime.obtain_uid().unwrap();

            let call_data = createAlphaCall::new((104,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_4_id = runtime.obtain_uid().unwrap();

            let call_data = createDeltaTtoCall::new((alpha_3_id, alpha_4_id)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let delta_2_id = runtime.obtain_uid().unwrap();

            // Create epsilon using TTO method
            let call_data = createEpsilonTtoCall::new((delta_1_id, delta_2_id)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let epsilon_id = runtime.obtain_uid().unwrap();

            (
                alpha_1_id, alpha_2_id, alpha_3_id, alpha_4_id, delta_1_id, delta_2_id, epsilon_id,
            )
        } else {
            // Create epsilon directly
            let call_data = createEpsilonCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let delta_1_id = runtime.obtain_uid().unwrap();
            let alpha_1_id = runtime.obtain_uid().unwrap();
            let alpha_2_id = runtime.obtain_uid().unwrap();
            let delta_2_id = runtime.obtain_uid().unwrap();
            let alpha_3_id = runtime.obtain_uid().unwrap();
            let alpha_4_id = runtime.obtain_uid().unwrap();
            let epsilon_id = runtime.obtain_uid().unwrap();

            (
                alpha_1_id, alpha_2_id, alpha_3_id, alpha_4_id, delta_1_id, delta_2_id, epsilon_id,
            )
        };

    // Read epsilon and assert the returned data
    let call_data = readEpsilonCall::new((epsilon_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readEpsilonCall::abi_decode_returns(&return_data).unwrap();
    let epsilon_expected = Epsilon::abi_encode(&Epsilon {
        id: UID {
            id: ID { bytes: epsilon_id },
        },
        a: vec![
            Delta {
                id: UID {
                    id: ID { bytes: delta_1_id },
                },
                a: vec![
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_1_id },
                        },
                        value: 101,
                    },
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_2_id },
                        },
                        value: 102,
                    },
                ],
            },
            Delta {
                id: UID {
                    id: ID { bytes: delta_2_id },
                },
                a: vec![
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_3_id },
                        },
                        value: 103,
                    },
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_4_id },
                        },
                        value: 104,
                    },
                ],
            },
        ],
    });
    assert_eq!(Epsilon::abi_encode(&return_data), epsilon_expected);
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete epsilon and assert the storage is empty afterwards
    let call_data = deleteEpsilonCall::new((epsilon_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

const RECIPIENT_ADDRESS: [u8; 20] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

#[rstest]
fn test_transferring_beta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createBetaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_id = runtime.obtain_uid().unwrap();
    let beta_id = runtime.obtain_uid().unwrap();

    let beta_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &beta_id.0);

    runtime.print_storage();

    // Transfer beta to the recipient
    let call_data = transferBetaCall::new((beta_id, RECIPIENT_ADDRESS.into())).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read beta from the recipient namespace in storage
    runtime.set_tx_origin(RECIPIENT_ADDRESS);
    let call_data = readBetaCall::new((beta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
    let beta_expected = Beta::abi_encode(&Beta {
        id: UID {
            id: ID { bytes: beta_id },
        },
        a: Alpha {
            id: UID {
                id: ID { bytes: alpha_id },
            },
            value: 101,
        },
    });
    assert_eq!(Beta::abi_encode(&return_data), beta_expected);
    assert_eq!(0, result);

    // Assert that beta is not in the original namespace anymore
    assert_eq!(
        runtime.get_storage_at_slot(beta_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );
    assert_eq!(
        runtime.get_storage_at_slot(get_next_slot(&beta_slot.0)),
        [0u8; 32],
        "Slot should be empty"
    );
}

#[rstest]
fn test_transferring_gamma(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createGammaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_id = runtime.obtain_uid().unwrap();
    let beta_id = runtime.obtain_uid().unwrap();
    let gamma_id = runtime.obtain_uid().unwrap();

    let gamma_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &gamma_id.0);

    // Transfer beta to the recipient
    let call_data = transferGammaCall::new((gamma_id, RECIPIENT_ADDRESS.into())).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read beta from the recipient namespace in storage
    runtime.set_tx_origin(RECIPIENT_ADDRESS);
    let call_data = readGammaCall::new((gamma_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readGammaCall::abi_decode_returns(&return_data).unwrap();
    let gamma_expected = Gamma::abi_encode(&Gamma {
        id: UID {
            id: ID { bytes: gamma_id },
        },
        a: Beta {
            id: UID {
                id: ID { bytes: beta_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_id },
                },
                value: 101,
            },
        },
    });
    assert_eq!(Gamma::abi_encode(&return_data), gamma_expected);
    assert_eq!(0, result);

    // Assert that beta is not in the original namespace anymore
    assert_eq!(
        runtime.get_storage_at_slot(gamma_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );
    assert_eq!(
        runtime.get_storage_at_slot(get_next_slot(&gamma_slot.0)),
        [0u8; 32],
        "Slot should be empty"
    );
}

#[rstest]
fn test_transferring_delta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createDeltaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_1_id = runtime.obtain_uid().unwrap();
    let alpha_2_id = runtime.obtain_uid().unwrap();
    let delta_id = runtime.obtain_uid().unwrap();

    let delta_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &delta_id.0);

    let call_data = transferDeltaCall::new((delta_id, RECIPIENT_ADDRESS.into())).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    runtime.set_tx_origin(RECIPIENT_ADDRESS);
    // Read delta and assert the returned data
    let call_data = readDeltaCall::new((delta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_id },
        },
        a: vec![
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            },
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_2_id },
                },
                value: 102,
            },
        ],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    // Assert delta was deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(delta_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );
    assert_eq!(
        runtime.get_storage_at_slot(get_next_slot(&delta_slot.0)),
        [0u8; 32],
        "Slot should be empty"
    );
}
#[rstest]
fn test_rebuilding_gamma(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createGammaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_id = runtime.obtain_uid().unwrap();
    let beta_id = runtime.obtain_uid().unwrap();
    let gamma_id = runtime.obtain_uid().unwrap();

    let gamma_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &gamma_id.0);

    // Rebuild gamma
    let call_data = rebuildGammaCall::new((gamma_id, RECIPIENT_ADDRESS.into())).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let new_gamma_id = runtime.obtain_uid().unwrap();

    // Read gamma from the recipient namespace in storage
    runtime.set_tx_origin(RECIPIENT_ADDRESS);
    let call_data = readGammaCall::new((new_gamma_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readGammaCall::abi_decode_returns(&return_data).unwrap();
    let gamma_expected = Gamma::abi_encode(&Gamma {
        id: UID {
            id: ID {
                bytes: new_gamma_id,
            },
        },
        a: Beta {
            id: UID {
                id: ID { bytes: beta_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_id },
                },
                value: 101,
            },
        },
    });
    assert_eq!(Gamma::abi_encode(&return_data), gamma_expected);
    assert_eq!(0, result);

    // Assert the old gamma was deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(gamma_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );
    assert_eq!(
        runtime.get_storage_at_slot(get_next_slot(&gamma_slot.0)),
        [0u8; 32],
        "Slot should be empty"
    );
}

#[rstest]
fn test_destruct_delta_to_beta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createDeltaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_1_id = runtime.obtain_uid().unwrap();
    let alpha_2_id = runtime.obtain_uid().unwrap();
    let delta_id = runtime.obtain_uid().unwrap();

    let storage_before_destruct = runtime.get_storage();

    let call_data = destructDeltaToBetaCall::new((delta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_destruct = runtime.get_storage();

    // Delta is deleted and each alpha is wrapped in a new beta, hence all the original slots should be empty
    assert_empty_storage(&storage_before_destruct, &storage_after_destruct);

    let beta_1_id = runtime.obtain_uid().unwrap();
    let beta_2_id = runtime.obtain_uid().unwrap();

    // Read the betas and assert the returned data is correct
    let call_data = readBetaCall::new((beta_1_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
    let beta_expected = Beta::abi_encode(&Beta {
        id: UID {
            id: ID { bytes: beta_1_id },
        },
        a: Alpha {
            id: UID {
                id: ID { bytes: alpha_2_id },
            },
            value: 102,
        },
    });
    assert_eq!(Beta::abi_encode(&return_data), beta_expected);
    assert_eq!(0, result);

    let call_data = readBetaCall::new((beta_2_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
    let beta_expected = Beta::abi_encode(&Beta {
        id: UID {
            id: ID { bytes: beta_2_id },
        },
        a: Alpha {
            id: UID {
                id: ID { bytes: alpha_1_id },
            },
            value: 101,
        },
    });
    assert_eq!(Beta::abi_encode(&return_data), beta_expected);
    assert_eq!(0, result);
}

#[rstest]
fn test_pushing_alpha_into_delta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    // Create empty delta
    let call_data = createEmptyDeltaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let delta_id = runtime.obtain_uid().unwrap();

    // Create alpha
    let call_data = createAlphaCall::new((101,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_1_id = runtime.obtain_uid().unwrap();
    let alpha_1_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_1_id.0);

    // Push alpha to delta
    let call_data = pushAlphaToDeltaCall::new((delta_id, alpha_1_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read delta and assert the returned data is correct
    let call_data = readDeltaCall::new((delta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_id },
        },
        a: vec![Alpha {
            id: UID {
                id: ID { bytes: alpha_1_id },
            },
            value: 101,
        }],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    // Assert alpha is deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(alpha_1_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    // Create second alpha
    let call_data = createAlphaCall::new((102,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_2_id = runtime.obtain_uid().unwrap();
    let alpha_2_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_2_id.0);

    // Push second alpha to delta
    let call_data = pushAlphaToDeltaCall::new((delta_id, alpha_2_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read delta and assert the returned data is correct
    let call_data = readDeltaCall::new((delta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_id },
        },
        a: vec![
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            },
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_2_id },
                },
                value: 102,
            },
        ],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    // Assert second alpha is deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(alpha_2_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    // Pop one alpha from delta and assert the returned data is correct
    let call_data = popAlphaFromDeltaCall::new((delta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read alpha_2 from the shared namespace and assert the data is correct
    let alpha_2_shared_slot = derive_object_slot(&SHARED, &alpha_2_id.0);
    assert_eq!(
        runtime.get_storage_at_slot(alpha_2_shared_slot.0),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 102, 198, 1, 192,
            204, 10, 101, 122, 43
        ],
        "Slot should not be empty"
    );

    // Read delta after the pop and assert the data is correct
    let call_data = readDeltaCall::new((delta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_id },
        },
        a: vec![Alpha {
            id: UID {
                id: ID { bytes: alpha_1_id },
            },
            value: 101,
        }],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    // Pop the last alpha from delta and assert the data is correct
    // In this case the beta vector is left empty.
    let call_data = popAlphaFromDeltaCall::new((delta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read alpha_1 from the shared namespace and assert the data is correct
    let alpha_1_shared_slot = derive_object_slot(&SHARED, &alpha_1_id.0);
    assert_eq!(
        runtime.get_storage_at_slot(alpha_1_shared_slot.0),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 198, 1, 192,
            204, 10, 101, 122, 43
        ],
        "Slot should not be empty"
    );

    // Read the popped alpha and assert the returned data is correct
    let call_data = readAlphaCall::new((alpha_1_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readAlphaCall::abi_decode_returns(&return_data).unwrap();
    let alpha_expected = Alpha::abi_encode(&Alpha {
        id: UID {
            id: ID { bytes: alpha_1_id },
        },
        value: 101,
    });
    assert_eq!(Alpha::abi_encode(&return_data), alpha_expected);
    assert_eq!(0, result);

    // Read delta after the pop and assert the data is correct
    let call_data = readDeltaCall::new((delta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_id },
        },
        a: vec![],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    // Create third alpha
    let call_data = createAlphaCall::new((103,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_3_id = runtime.obtain_uid().unwrap();

    // Push one more alpha to delta
    let call_data = pushAlphaToDeltaCall::new((delta_id, alpha_3_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete the shared alphas
    let call_data = deleteAlphaCall::new((alpha_1_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = deleteAlphaCall::new((alpha_2_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Delete delta
    let call_data = deleteDeltaCall::new((delta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_destruct_epsilon(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createEpsilonCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let delta_1_id = runtime.obtain_uid().unwrap();
    let alpha_1_id = runtime.obtain_uid().unwrap();
    let alpha_2_id = runtime.obtain_uid().unwrap();
    let delta_2_id = runtime.obtain_uid().unwrap();
    let alpha_3_id = runtime.obtain_uid().unwrap();
    let alpha_4_id = runtime.obtain_uid().unwrap();
    let epsilon_id = runtime.obtain_uid().unwrap();

    let epsilon_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &epsilon_id.0);

    let call_data = createAlphaCall::new((105,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_5_id = runtime.obtain_uid().unwrap();
    let alpha_5_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_5_id.0);

    let call_data = destructEpsilonCall::new((epsilon_id, alpha_5_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that epsilon is deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(epsilon_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    // Assert that alpha 5 is deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(alpha_5_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    // Read delta and assert the returned data
    let call_data = readDeltaCall::new((delta_2_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
    let delta_expected = Delta::abi_encode(&Delta {
        id: UID {
            id: ID { bytes: delta_2_id },
        },
        a: vec![
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_3_id },
                },
                value: 103,
            },
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_4_id },
                },
                value: 104,
            },
            Alpha {
                id: UID {
                    id: ID { bytes: alpha_5_id },
                },
                value: 105,
            },
        ],
    });
    assert_eq!(Delta::abi_encode(&return_data), delta_expected);
    assert_eq!(0, result);

    let new_epsilon_id = runtime.obtain_uid().unwrap();

    // Read epsilon and assert the returned data
    let call_data = readEpsilonCall::new((new_epsilon_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readEpsilonCall::abi_decode_returns(&return_data).unwrap();
    let epsilon_expected = Epsilon::abi_encode(&Epsilon {
        id: UID {
            id: ID {
                bytes: new_epsilon_id,
            },
        },
        a: vec![Delta {
            id: UID {
                id: ID { bytes: delta_1_id },
            },
            a: vec![
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_2_id },
                    },
                    value: 102,
                },
            ],
        }],
    });
    assert_eq!(Epsilon::abi_encode(&return_data), epsilon_expected);
    assert_eq!(0, result);
}

#[rstest]
fn test_pushing_and_popping_alpha_from_zeta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createEmptyZetaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let zeta_id = runtime.obtain_uid().unwrap();

    let call_data = readZetaCall::new((zeta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
    let zeta_expected = Zeta::abi_encode(&Zeta {
        id: UID {
            id: ID { bytes: zeta_id },
        },
        b: Astra { a: vec![] },
    });
    assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
    assert_eq!(0, result);

    // Create alpha 1 and alpha 2
    let call_data = createAlphaCall::new((101,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_1_id = runtime.obtain_uid().unwrap();
    let alpha_1_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_1_id.0);

    let call_data = createAlphaCall::new((102,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_2_id = runtime.obtain_uid().unwrap();
    let alpha_2_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_2_id.0);

    // Pushback alpha 1 and alpha 2 to zeta
    let call_data = pushAlphaToZetaCall::new((zeta_id, alpha_1_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = pushAlphaToZetaCall::new((zeta_id, alpha_2_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read zeta and assert the returned data
    let call_data = readZetaCall::new((zeta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
    let zeta_expected = Zeta::abi_encode(&Zeta {
        id: UID {
            id: ID { bytes: zeta_id },
        },
        b: Astra {
            a: vec![
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_2_id },
                    },
                    value: 102,
                },
            ],
        },
    });
    assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
    assert_eq!(0, result);

    // Assert that alpha 1 and alpha 2 are deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(alpha_1_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );
    assert_eq!(
        runtime.get_storage_at_slot(alpha_2_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    // Popback the last alpha from zeta
    let call_data = popAlphaFromZetaCall::new((zeta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read zeta and assert the returned data
    let call_data = readZetaCall::new((zeta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
    let zeta_expected = Zeta::abi_encode(&Zeta {
        id: UID {
            id: ID { bytes: zeta_id },
        },
        b: Astra {
            a: vec![Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            }],
        },
    });
    assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
    assert_eq!(0, result);

    // Assert that alpha 2 is under the shared namespace now
    let alpha_2_shared_slot = derive_object_slot(&SHARED, &alpha_2_id.0);
    assert_ne!(
        runtime.get_storage_at_slot(alpha_2_shared_slot.0),
        [0u8; 32],
        "Slot should not be empty"
    );

    let storage_before_delete = runtime.get_storage();

    // Delete zeta
    let call_data = deleteZetaCall::new((zeta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Delete alpha 2
    let call_data = deleteAlphaCall::new((alpha_2_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_popping_from_empty_zeta(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createEmptyZetaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let zeta_id = runtime.obtain_uid().unwrap();

    let call_data = readZetaCall::new((zeta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
    let zeta_expected = Zeta::abi_encode(&Zeta {
        id: UID {
            id: ID { bytes: zeta_id },
        },
        b: Astra { a: vec![] },
    });
    assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
    assert_eq!(0, result);

    // Create alpha 1 and alpha 2
    let call_data = createAlphaCall::new((101,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let alpha_1_id = runtime.obtain_uid().unwrap();
    let alpha_1_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_1_id.0);

    let call_data = createAlphaCall::new((102,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Pushback alpha 1 to zeta
    let call_data = pushAlphaToZetaCall::new((zeta_id, alpha_1_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read zeta and assert the returned data
    let call_data = readZetaCall::new((zeta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
    let zeta_expected = Zeta::abi_encode(&Zeta {
        id: UID {
            id: ID { bytes: zeta_id },
        },
        b: Astra {
            a: vec![Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            }],
        },
    });
    assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
    assert_eq!(0, result);

    // Assert that alpha 1 and alpha 2 are deleted from the original namespace
    assert_eq!(
        runtime.get_storage_at_slot(alpha_1_slot.0),
        [0u8; 32],
        "Slot should be empty"
    );

    // Popback alpha from zeta
    let call_data = popAlphaFromZetaCall::new((zeta_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read zeta and assert the returned data
    let call_data = readZetaCall::new((zeta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
    let zeta_expected = Zeta::abi_encode(&Zeta {
        id: UID {
            id: ID { bytes: zeta_id },
        },
        b: Astra { a: vec![] },
    });
    assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
    assert_eq!(0, result);

    // Assert that alpha 2 is under the shared namespace now
    let alpha_2_shared_slot = derive_object_slot(&SHARED, &alpha_1_id.0);
    assert_ne!(
        runtime.get_storage_at_slot(alpha_2_shared_slot.0),
        [0u8; 32],
        "Slot should not be empty"
    );

    // Popback again, even though the vector is empty
    let call_data = popAlphaFromZetaCall::new((zeta_id,)).abi_encode();
    let result = runtime.call_entrypoint(call_data);
    assert!(result.is_err());
}

#[rstest]
fn test_pushing_and_popping_from_bora(
    #[with("wrapped_objects", "tests/storage/move_sources/wrapped_objects.move")]
    runtime: RuntimeSandbox,
) {
    runtime.set_tx_origin(MSG_SENDER_ADDRESS);

    let call_data = createEtaCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let eta_id = runtime.obtain_uid().unwrap();

    let call_data = readEtaCall::new((eta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
    let eta_expected = Eta::abi_encode(&Eta {
        id: UID {
            id: ID { bytes: eta_id },
        },
        b: Bora {
            a: vec![],
            b: vec![],
        },
    });
    assert_eq!(Eta::abi_encode(&return_data), eta_expected);
    assert_eq!(0, result);

    // Push to bora
    let call_data = pushToBoraCall::new((eta_id, 1)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read bora and assert the returned data
    let call_data = readEtaCall::new((eta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
    let eta_expected = Eta::abi_encode(&Eta {
        id: UID {
            id: ID { bytes: eta_id },
        },
        b: Bora {
            a: vec![1],
            b: vec![vec![1, 2, 3]],
        },
    });
    assert_eq!(Eta::abi_encode(&return_data), eta_expected);
    assert_eq!(0, result);

    // Push to bora
    let call_data = pushToBoraCall::new((eta_id, 10)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read bora and assert the returned data
    let call_data = readEtaCall::new((eta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
    let eta_expected = Eta::abi_encode(&Eta {
        id: UID {
            id: ID { bytes: eta_id },
        },
        b: Bora {
            a: vec![1, 10],
            b: vec![vec![1, 2, 3], vec![10, 11, 12]],
        },
    });
    assert_eq!(Eta::abi_encode(&return_data), eta_expected);
    assert_eq!(0, result);

    // Pop from bora
    let call_data = popFromBoraCall::new((eta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = popFromBoraCall::abi_decode_returns(&return_data).unwrap();
    let value = return_data._0;
    let vector = return_data._1;
    assert_eq!(value, 10);
    assert_eq!(vector, vec![10, 11, 12]);
    assert_eq!(0, result);

    let call_data = readEtaCall::new((eta_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
    let eta_expected = Eta::abi_encode(&Eta {
        id: UID {
            id: ID { bytes: eta_id },
        },
        b: Bora {
            a: vec![1],
            b: vec![vec![1, 2, 3]],
        },
    });
    assert_eq!(Eta::abi_encode(&return_data), eta_expected);
    assert_eq!(0, result);
}
