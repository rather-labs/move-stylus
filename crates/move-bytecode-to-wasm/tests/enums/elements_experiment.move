module 0x00::elements_experiment;

public enum State has copy, drop {
    Solid,
    Liquid,
    Gas
}

public enum Symbol has copy, drop {
    H,
    He,
    C,
    N,
    O,
}

public struct Element has copy, drop {
    symbol: Symbol,
    boil_point: u64,
    freezing_point: u64,
    density: u64,
}

public enum Substance has copy, drop {
   Pure(Element),
   Mixture{a: Element, b: Element, concentration: u8},
}

public enum Vessel has copy, drop {
   Erlenmayer(u64), // volume in mL
   Beaker(u64),
   Flask(u64),
   TestTube(u64),
}

// Calculate density of a mixture using weighted average
fun get_substance_density(substance: Substance): u64 {
    match (substance) {
        Substance::Pure(element) => {
            element.density
        },
        Substance::Mixture{a, b, concentration} => {
            // concentration is percentage of element 'a' (0-100)
            // Weighted average: (concentration * a.density + (100 - concentration) * b.density) / 100
            ((concentration as u64) * a.density + ((100 - concentration) as u64) * b.density) / 100
        },
    }
}

fun get_element(symbol: Symbol): Element {
    match (symbol) {
        // Realistic values for common conditions (all values in SI units)
        // Boiling/freezing points: Kelvin, Density: kg/m^3 * 1000 (rounding as needed)
        Symbol::H => Element{
            symbol: Symbol::H,
            boil_point: 20,           // 20.27 K
            freezing_point: 14,       // 13.99 K
            density: 899,             // 0.08988 kg/m^3 * 10,000 = 898.8 ≈ 899
        },
        Symbol::He => Element{
            symbol: Symbol::He,
            boil_point: 4,            // 4.22 K
            freezing_point: 1,        // 0.95 K
            density: 178,             // 0.1786 kg/m^3 * 1000 ≈ 179 (but use 178 for more precision)
        },
        Symbol::C => Element{
            symbol: Symbol::C,
            boil_point: 3915,         // Sublimation point, 3915 K (doesn't boil at 1 atm)
            freezing_point: 3915,     // Same as "boil" (sublimation)
            density: 2260000,         // 2.260 g/cm^3 = 2260 kg/m^3 * 1000 = 2,260,000
        },
        Symbol::N => Element{
            symbol: Symbol::N,
            boil_point: 77,           // 77.36 K
            freezing_point: 63,       // 63.15 K
            density: 1251,            // 1.2506 kg/m^3 * 1000 = 1250.6 ≈ 1251
        },  
        Symbol::O => Element{
            symbol: Symbol::O,
            boil_point: 90,           // 90.19 K
            freezing_point: 54,       // 54.36 K
            density: 1429,            // 1.429 kg/m^3 * 1000 = 1429
        },
    }
}

entry fun get_pure_substance_density(symbol: Symbol): u64 {
    let element = get_element(symbol);
    let substance = Substance::Pure(element);
    get_substance_density(substance)
}

entry fun get_mixture_substance_density(a: Symbol, b: Symbol, concentration: u8): u64 {
    let element_a = get_element(a);
    let element_b = get_element(b);
    let substance = Substance::Mixture{a: element_a, b: element_b, concentration};
    get_substance_density(substance)
}

entry fun get_element_state(symbol: Symbol, temperature: u64): State {
    let element = get_element(symbol);
    if (temperature > element.boil_point) {
        State::Gas
    } else if (temperature > element.freezing_point) {
        State::Liquid
    } else {
        State::Solid
    }
}

public struct Experiment has copy, drop {
    vessel: Vessel,
    substance: Substance,
    temperature: u64,
}

fun make_experiment_1(): Experiment {
    // Result: (1000 * 100) / 25 = 4000
    let vessel = Vessel::Erlenmayer(100);
    let substance = Substance::Pure(Element{symbol: Symbol::H, boil_point: 20, freezing_point: 14, density: 1000});
    let temperature = 25;
    let experiment = Experiment{vessel: vessel, substance: substance, temperature: temperature};
    experiment
}

fun make_experiment_2(): Experiment {
    // Mixture: (50% of 800) + (50% of 1200) = 1000
    // Result: (1000 * 300) / 50 = 6000
    let vessel = Vessel::Beaker(300);
    let substance = Substance::Mixture{
        a: Element{symbol: Symbol::He, boil_point: 4, freezing_point: 1, density: 800},
        b: Element{symbol: Symbol::O, boil_point: 90, freezing_point: 54, density: 1200},
        concentration: 50
    };
    let temperature = 50;
    let experiment = Experiment{vessel: vessel, substance: substance, temperature: temperature};
    experiment
}

fun make_experiment_3(): Experiment {
    // Mixture: (25% of 2000) + (75% of 1000) = 1250
    // Result: (1250 * 500) / 125 = 5000
    let vessel = Vessel::Flask(500);
    let substance = Substance::Mixture{
        a: Element{symbol: Symbol::C, boil_point: 3915, freezing_point: 3915, density: 2000},
        b: Element{symbol: Symbol::N, boil_point: 77, freezing_point: 63, density: 1000},
        concentration: 25
    };
    let temperature = 125;
    let experiment = Experiment{vessel: vessel, substance: substance, temperature: temperature};
    experiment
}

entry fun run_experiments(): vector<u64> {
    let mut experiments = vector[make_experiment_1(), make_experiment_2(), make_experiment_3()];
    let mut results = vector::empty();
    while (vector::length(&experiments) > 0) {
        let experiment = vector::pop_back(&mut experiments);
        let density = get_substance_density(experiment.substance);
        let temperature = experiment.temperature;
        match (experiment.vessel) {
            Vessel::Erlenmayer(volume) | Vessel::Beaker(volume) | Vessel::Flask(volume) => {
                let result = (density * volume) / temperature;
                vector::push_back(&mut results, result);
            },
            Vessel::TestTube(volume)  => {
                let result = (density * volume / 2 ) / temperature;
                vector::push_back(&mut results, result);
            },
        }
    };
    results
}