module 0x00::structs_with_enums;
use std::ascii::String;
use std::ascii as ascii;

const E_SUPERNOVA: u64 = 1;

public enum Core has drop, copy {
    Hydrogen,
    Helium,
    Carbon,
    Nitrogen,
    Oxygen,
}

public enum StarType has drop, copy {
    RedDwarf,
    YellowDwarf,
    RedGiant,
    BlueGiant,
}

public struct Star has drop, copy {
    name: String,
    class: StarType,
    core: Core,
    size: u32,
}

/// Creates a new star with the given properties.
entry fun create_star(name: String, class: StarType, core: Core, size: u32): Star {
    Star { name, class, core, size }
}

/// Evolves the star's core to the next element in the periodic table.
/// This also changes the star's class and size according to stellar evolution.
entry fun evolve_star(star: &mut Star): Star {
    match (&star.core) {
        Core::Hydrogen => {
            star.core = Core::Helium;
            star.class = StarType::RedGiant;  // Hydrogen depletion leads to red giant phase
            star.size = star.size * 100;     // Stars expand dramatically as red giants
        },
        Core::Helium => {
            star.core = Core::Carbon;
            star.class = StarType::RedGiant;  // Still in red giant phase
            star.size = star.size * 2;        // Further expansion
        },
        Core::Carbon => {
            star.core = Core::Nitrogen;
            star.class = StarType::BlueGiant; // More massive stars become blue giants
            star.size = star.size * 5;        // Blue giants are very large
        },
        Core::Nitrogen => {
            star.core = Core::Oxygen;
            star.class = StarType::BlueGiant; // Still blue giant
            star.size = star.size * 3;        // Continued expansion
        },
        Core::Oxygen => {
            // If already at last, star collapses (supernova)
            abort(E_SUPERNOVA)
        },
    };
    *star
}

/// Returns (atomic_number, group_number) for the star's core element.
entry fun get_core_properties(star: &Star): (u8, u8) {
    match (&star.core) {
        Core::Hydrogen => (1, 1),
        Core::Helium => (2, 18),
        Core::Carbon => (6, 14),
        Core::Nitrogen => (7, 15),
        Core::Oxygen => (8, 16),
    }
}