module 0x00::simple_enums_match;

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

public fun match_number_enum(x: NumberEnum): u32 {
    match (x) {
        NumberEnum::One => 11,
        NumberEnum::Two => 22,
        NumberEnum::Three => 33,
        _ => 44,
    }
}

public fun match_nested_enum(x: NumberEnum, y: ColorEnum, z: YinYangEnum): u32 {
    match (x) {
        NumberEnum::One => 11,
        NumberEnum::Two => {
            match (y) {
                ColorEnum::Red => 22,
                ColorEnum::Green => 33,
                ColorEnum::Blue => 44,
            }
        },
        _ => {
            match (z) {
                YinYangEnum::Yin => 55,
                YinYangEnum::Yang => 66,
            }
        },
    }
}