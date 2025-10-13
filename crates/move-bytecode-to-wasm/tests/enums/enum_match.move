module 0x00::enum_match;

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

public fun match_number_enum(x: NumberEnum): u32 {
    match (x) {
        NumberEnum::One => 11,
        NumberEnum::Two => 22,
        NumberEnum::Three => 33,
        _ => 44,
    }
}

public fun match_nested_enum(x: NumberEnum, y: ColorEnum): u32 {
    match (x) {
        NumberEnum::One => 11,
        NumberEnum::Two => {
            match (y) {
                ColorEnum::Red => 22,
                ColorEnum::Green => 33,
                ColorEnum::Blue => 44,
            }
        },
        _ => 55,
    }
}