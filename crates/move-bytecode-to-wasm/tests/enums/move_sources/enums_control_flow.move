module 0x00::enums_control_flow;

// This module defines simple enums without any fields and includes functions for matching on these enums.

public enum Number has drop, copy {
    One,
    Two,
    Three,
    Four,
    Five,
}

public enum Color has drop, copy {
    R,
    G,
    B,
}

// This enum produces an if/else flow instead of a switch flow
public enum Boolean has drop, copy {
    True,
    False,
}

entry fun simple_match(n: Number): u32 {
    match (n) {
        Number::One => 1,
        Number::Two => 2,
        Number::Three => 3,
        Number::Four => 4,
        Number::Five => 5,
    }
}

entry fun simple_match_single_case(n: Number): u32 {
    match (n) {
          Number::One => 42,
        _ => abort(1)
    }
}

entry fun nested_match(n: Number, c: Color, b: Boolean): u32 {
    match (n) {
        Number::One => 1,
        Number::Two => {
            match (c) {
                Color::R => 2,
                Color::G => 3,
                Color::B => 4,
            }
        },
        _ => {
            match (b) {
                Boolean::True => 5,
                Boolean::False => 6,
            }
        },
    }
}

entry fun match_with_conditional(n: Number, a: bool, b: bool): u32 {
    if (a) {
        match(n) {
            Number::One => 1,
            _ => 2,
        }
    } else {
        match(n) {
            Number::Five => 3,
            Number::Four => {
                if (b) {
                    4
                } else {
                    5
                }
            },
            _ => 6,
        }
    }
}

entry fun nested_match_with_conditional(n: Number, c: Color, a: bool, b: bool): u32 {
    if (a) {
        match(n) {
            Number::One => 1,
            _ => 2,
        }
    } else {
        match(n) {
            Number::Five => 3,
            Number::Four => {
                if (b) {
                    match(c) {
                        Color::R => 4,
                        _ => 5,
                    }
                } else {
                    match(c) {
                        Color::R => 6,
                        _ => 7,
                    }
                }
            },
            _ => 8,
        }
    }
}

// Testing cases where most branches abort
entry fun match_with_many_aborts(n: Number, c: Color): u32 {
    match (n) {
        Number::One => abort(0),
        Number::Two => {
            match (c) {
                Color::R => abort(0),
                Color::G => 1,
                Color::B => abort(0),
            }
        },
        Number::Four => 2,
        _ => {
            abort(0)
        },
    }
}

// Same as above but all branches abort except one
entry fun match_with_single_yielding_branch(n: Number, c: Color): u32 {
    match (n) {
        Number::One => abort(0),
        Number::Two => {
            match (c) {
                Color::R => abort(0),
                Color::G => 1,
                Color::B => abort(0),
            }
        },
        Number::Four => abort(0),
        _ => {
            abort(0)
        },
    }
}

entry fun misc_control_flow(n: Number, c: Color, b: Boolean): u32 {
    match (n) {
        Number::One => abort(0),
        Number::Two => {
            match (c) {
                Color::R => abort(0),
                Color::G => 1,
                Color::B => abort(0),
            }
        },
        Number::Four => 2,
        _ => {
            abort(0)
        },
    } +
    match (b) {
        Boolean::True => 3,
        Boolean::False => 4,
    }
}

entry fun misc_control_flow_2(n: Number, c: Color, b: Boolean): u32 {
    match (n) {
        Number::One => abort(0),
        Number::Two => {
            match (c) {
                Color::R => abort(1),
                Color::G => 1,
                Color::B => abort(1),
            }
        },
        Number::Four => abort(0),
        _ => {
            abort(0)
        },
    } + 
    match (b) {
        Boolean::True => abort(1),
        Boolean::False => 2,
    }
}

entry fun misc_control_flow_3(c: Color): u64 {
    match (c) {
        Color::R => {
            let mut i = 0;
            while (i < 5) {
                i = i + 1;
            };
            i
        },
        Color::G => 7,
        Color::B => 11,
    }
}

entry fun misc_control_flow_4(n: Number, b: Boolean): u64 {
    let rounds = match (b) {
        Boolean::True  => 2,
        Boolean::False => 3,
    };

    let mut i = 0;
    let mut total = 0;
    while (i < rounds) {
        total = total + match (n) {
            Number::One => 1,
            Number::Two => 2,
            _ => {
                let mut j = 0;
                let mut s = 0;
                while (j < 5) {
                    s = s + 3;
                    j = j + 1;
                };
                s
            }
        };
        i = i + 1;
    };
    total
}

entry fun misc_control_flow_5(n: &mut Number): u64 {
    let mut flag = true;
    let mut acc = 6;
    while (flag) {
        flag = match (n) {
            Number::One => {*n = Number::Two; true},
            Number::Two => {*n = Number::Three; true},
            Number::Three => {*n = Number::Four; true},
            Number::Four => {*n = Number::Five; true},
            Number::Five => false,
        };
        acc = acc - 1;
    };
    acc
}
