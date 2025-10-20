module hello_world::enums;

// This module defines simple enums without any fields and includes functions for matching on these enums.

public enum NumberEnum has drop {
    One,
    Two,
    Three,
    Four,
    Five,
}

public enum ColorEnum has drop {
    Red,
    Green,
    Blue,
}

// This enum produces an if/else flow instead of a switch flow
public enum YinYangEnum has drop {
    Yin,
    Yang,
}

entry fun match_number_enum(x: NumberEnum): u32 {
    match (x) {
        NumberEnum::One => 11,
        NumberEnum::Two => abort,
        NumberEnum::Three => 22,
        _ => 33,
    }
}