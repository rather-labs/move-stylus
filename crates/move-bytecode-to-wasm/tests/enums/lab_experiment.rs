use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "elements_experiment",
    "tests/enums/move_sources/elements_experiment.move"
);

sol! {
    enum State {
        Solid,
        Liquid,
        Gas
    }
    enum Symbol  {
        H,
        He,
        C,
        N,
        O,
    }
    struct Element {
        Symbol symbol;
        uint64 boil_point;
        uint64 freezing_point;
        uint64 density;
    }

    function getElementState(Symbol symbol, uint64 temperature) external returns (State);
    function getPureSubstanceDensity(Symbol symbol) external returns (uint64);
    function getMixtureSubstanceDensity(Symbol a, Symbol b, uint8 concentration) external returns (uint64);
    function runExperiments() external returns (uint64[]);
    function getDensityOfSubstances() external returns (uint64[]);
}

#[rstest]
#[case(getElementStateCall::new((Symbol::H, 10u64)), State::Solid)]
#[case(getElementStateCall::new((Symbol::H, 30u64)), State::Gas)]
#[case(getElementStateCall::new((Symbol::He, 0u64)), State::Solid)]
#[case(getElementStateCall::new((Symbol::He, 2u64)), State::Liquid)]
#[case(getElementStateCall::new((Symbol::C, 2000u64)), State::Solid)]
#[case(getElementStateCall::new((Symbol::C, 4000u64)), State::Gas)]
#[case(getElementStateCall::new((Symbol::N, 50u64)), State::Solid)]
#[case(getElementStateCall::new((Symbol::N, 70u64)), State::Liquid)]
#[case(getElementStateCall::new((Symbol::O, 40u64)), State::Solid)]
#[case(getElementStateCall::new((Symbol::O, 70u64)), State::Liquid)]
#[case(getElementStateCall::new((Symbol::O, 100u64)), State::Gas)]
#[case(getPureSubstanceDensityCall::new((Symbol::H,)), 899u64)]
#[case(getPureSubstanceDensityCall::new((Symbol::C,)), 2260000u64)]
#[case(getPureSubstanceDensityCall::new((Symbol::O,)), 1429u64)]
#[case(getMixtureSubstanceDensityCall::new((Symbol::H, Symbol::He, 50u8)), 538u64)]
#[case(getMixtureSubstanceDensityCall::new((Symbol::O, Symbol::C, 10u8)), 2034142u64)]
#[case(getMixtureSubstanceDensityCall::new((Symbol::N, Symbol::O, 70u8)), 1304u64)]
#[case(getMixtureSubstanceDensityCall::new((Symbol::He, Symbol::N, 30u8)), 929u64)]
#[case(getMixtureSubstanceDensityCall::new((Symbol::C, Symbol::He, 90u8)), 2034017u64)]
#[case(runExperimentsCall::new(()), vec![5000u64, 6000u64, 4000u64])]
#[case(getDensityOfSubstancesCall::new(()), vec![1000u64, 800u64, 2000u64, 1000u64, 1200u64])]
fn test_elements_experiment<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) {
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}
