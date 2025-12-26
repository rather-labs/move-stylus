use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("stars", "tests/enums/move_sources/stars.move");

sol! {
    enum Core {
        Hydrogen,
        Helium,
        Carbon,
        Nitrogen,
        Oxygen,
    }

    enum StarType {
        RedDwarf,
        YellowDwarf,
        RedGiant,
        BlueGiant,
    }

    struct Star {
        string name;
        StarType class;
        Core core;
        uint32 size;
    }

    function createStar(string name, StarType class, Core core, uint32 size) external returns (Star);
    function evolveStar(Star star) external returns (Star);
    function getCoreProperties(Star star) external returns (uint8, uint8);
    function getMilkyWayMass() external returns (uint64);
}

#[rstest]
#[case(createStarCall::new((String::from("Sun"), StarType::YellowDwarf, Core::Hydrogen, 55)), Star { name: String::from("Sun"), class: StarType::YellowDwarf, core: Core::Hydrogen, size: 55 })]
#[case(createStarCall::new((String::from("Proxima Centauri"), StarType::RedDwarf, Core::Helium, 1)), Star { name: String::from("Proxima Centauri"), class: StarType::RedDwarf, core: Core::Helium, size: 1 })]
#[case(createStarCall::new((String::from("Betelgeuse"), StarType::RedGiant, Core::Carbon, 764)), Star { name: String::from("Betelgeuse"), class: StarType::RedGiant, core: Core::Carbon, size: 764 })]
#[case(createStarCall::new((String::from("Vega"), StarType::BlueGiant, Core::Nitrogen, 2)), Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 2 })]
#[case(createStarCall::new((String::from("Polaris"), StarType::YellowDwarf, Core::Oxygen, 37)), Star { name: String::from("Polaris"), class: StarType::YellowDwarf, core: Core::Oxygen, size: 37 })]
#[case(evolveStarCall::new((Star { name: String::from("Sun"), class: StarType::YellowDwarf, core: Core::Hydrogen, size: 55 },)), Star { name: String::from("Sun"), class: StarType::RedGiant, core: Core::Helium, size: 5500 })]
#[case(evolveStarCall::new((Star { name: String::from("Proxima Centauri"), class: StarType::RedDwarf, core: Core::Helium, size: 1 },)), Star { name: String::from("Proxima Centauri"), class: StarType::RedGiant, core: Core::Carbon, size: 2 })]
#[case(evolveStarCall::new((Star { name: String::from("Betelgeuse"), class: StarType::RedGiant, core: Core::Carbon, size: 764 },)), Star { name: String::from("Betelgeuse"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 3820 })]
#[case(evolveStarCall::new((Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 2 },)), Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Oxygen, size: 6 })]
#[should_panic]
#[case(evolveStarCall::new((Star { name: String::from("Polaris"), class: StarType::YellowDwarf, core: Core::Oxygen, size: 37 },)), Star { name: String::from("Polaris"), class: StarType::BlueGiant, core: Core::Oxygen, size: 111 })]
#[case(getCorePropertiesCall::new((Star { name: String::from("Sun"), class: StarType::YellowDwarf, core: Core::Hydrogen, size: 55 },)), (1, 1))]
#[case(getCorePropertiesCall::new((Star { name: String::from("Proxima Centauri"), class: StarType::RedDwarf, core: Core::Helium, size: 1 },)), (2, 18))]
#[case(getCorePropertiesCall::new((Star { name: String::from("Betelgeuse"), class: StarType::RedGiant, core: Core::Carbon, size: 764 },)), (6, 14))]
#[case(getCorePropertiesCall::new((Star { name: String::from("Vega"), class: StarType::BlueGiant, core: Core::Nitrogen, size: 2 },)), (7, 15))]
#[case(getCorePropertiesCall::new((Star { name: String::from("Polaris"), class: StarType::YellowDwarf, core: Core::Oxygen, size: 37 },)), (8, 16))]
#[case(getMilkyWayMassCall::new(()), 11185u64)]
fn test_star<T: SolCall, V: SolValue>(
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
