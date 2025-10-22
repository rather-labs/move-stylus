module 0x00::structs_with_enums;
use std::ascii::String;
use std::ascii as ascii;

public enum Core has drop {
    Hydrogen,
    Helium,
    Carbon,
    Nitrogen,
    Oxygen,
}

public enum StarType has drop {
    RedDwarf,
    YellowDwarf,
    RedGiant,
    BlueGiant,
}

public struct Star has drop {
    name: String,
    class: StarType,
    distance: u64,
    core: Core,
    size: u32,
}

entry fun create_star(name: String, class: StarType, distance: u64, core: Core, size: u32): Star {
    Star { name, class, distance, core, size }
}

fun change_star_class(star: &mut Star, new_class: StarType) {
    star.class = new_class;
}

fun change_star_core(star: &mut Star, new_core: Core) {
    star.core = new_core;
}

entry fun test_star(name: String, class: StarType, distance: u64, core: Core, size: u32, new_class: StarType, new_core: Core): Star {
    let mut star = create_star(name, class, distance, core, size);
    change_star_class(&mut star, new_class);
    change_star_core(&mut star, new_core);
    star
}