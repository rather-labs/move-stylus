use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("struct_misc", "tests/structs/move_sources/struct_misc.move");

sol! {
    // Empty structs in Move are filled with a dummy field. We need to explicitly define it so
    // the ABI encoding works correctly.
    struct Empty {
        bool dummy_field;
    }

    // Tuples structs in Move are defined as common structs with fields named `pos0`, `pos1`,
    // etc. To be able to use them in ABI encoding, we need to define them explicitly.
    struct Tuple {
        uint32 pos0;
        uint8[] pos1;
    }

    struct TupleGeneric {
        uint64 pos0;
        uint8[] pos1;
    }

    struct Coin {
        uint64 amount;
    }

    function packUnpackAbiEmpty(Empty empty) external returns (Empty empty);
    function packUnpackAbiTuple(Tuple tuple) external returns (Tuple tuple);
    function packUnpackAbiTupleGeneric(TupleGeneric tuple) external returns (TupleGeneric tuple);
    function exchangeUsdToJpy(Coin coin) external returns (Coin coin);
}

#[rstest]
#[case(packUnpackAbiEmptyCall::new(
        (Empty { dummy_field: true },)),
        Empty { dummy_field: true }
    )]
#[case(packUnpackAbiEmptyCall::new(
        (Empty { dummy_field: false },)),
        Empty { dummy_field: false }
    )]
#[case(packUnpackAbiTupleCall::new(
        (Tuple {
            pos0: 42,
            pos1: vec![1, 2, 3, 4, 5],
        },)),
        Tuple {
            pos0: 42,
            pos1: vec![1, 2, 3, 4, 5],
        }
    )]
#[case(packUnpackAbiTupleGenericCall::new(
        (TupleGeneric {
            pos0: 4242424242424242,
            pos1: vec![1, 2, 3, 4, 5],
        },)),
        TupleGeneric {
            pos0: 4242424242424242,
            pos1: vec![1, 2, 3, 4, 5],
        }
    )]
#[case(exchangeUsdToJpyCall::new(
        (Coin { amount: 1000 },)),
        Coin { amount: 1000 * 150 }
    )]
fn test_struct_misc<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}
