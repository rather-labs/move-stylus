module 0x00::stars;
use std::ascii::String;
use std::ascii::{Self};

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

public struct Galaxy has drop {
    name: String,
    stars: vector<Star>,
}

fun create_galaxy(name: String): Galaxy {
    Galaxy { name, stars: vector::empty() }
}

fun add_star(galaxy: &mut Galaxy, star: Star) {
    vector::push_back(&mut galaxy.stars, star);
}

entry fun get_milky_way_mass(): u64 {
    let mut galaxy = create_galaxy(ascii::string(b"Milky Way"));
    add_star(&mut galaxy, create_star( ascii::string(b"Sun"), StarType::YellowDwarf, Core::Hydrogen, 55));
    add_star(&mut galaxy, create_star(ascii::string(b"Proxima Centauri"), StarType::RedDwarf, Core::Helium, 10));
    add_star(&mut galaxy, create_star(ascii::string(b"Alpha Centauri A"), StarType::YellowDwarf, Core::Helium, 20));
    add_star(&mut galaxy, create_star(ascii::string(b"Alpha Centauri B"), StarType::YellowDwarf, Core::Helium, 100));
    add_star(&mut galaxy, create_star(ascii::string(b"Betelgeuse"), StarType::RedGiant, Core::Carbon, 1000));
    add_star(&mut galaxy, create_star(ascii::string(b"Rigel"), StarType::BlueGiant, Core::Nitrogen, 10000));

    let mut mass = 0u64;
    let mut i = 0;
    while (i < vector::length(&galaxy.stars)) {
        let star = *vector::borrow(&galaxy.stars, i);
        mass = mass + (star.size as u64);
        i = i + 1;
    };
    mass
}