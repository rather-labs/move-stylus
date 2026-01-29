// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

/// Converts the input string to camel case.
pub fn snake_to_camel(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    // .len returns byte count but ok in this case!

    #[derive(PartialEq)]
    enum ChIs {
        FirstOfStr,
        NextOfSepMark,
        Other,
    }

    let mut flag = ChIs::FirstOfStr;

    for ch in input.chars() {
        if flag == ChIs::FirstOfStr {
            result.push(ch.to_ascii_lowercase());
            flag = ChIs::Other;
        } else if ch == '_' {
            flag = ChIs::NextOfSepMark;
        } else if flag == ChIs::NextOfSepMark {
            result.push(ch.to_ascii_uppercase());
            flag = ChIs::Other;
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn snake_to_upper_camel(input: &str) -> String {
    let mut result = snake_to_camel(input);

    if let Some(first) = result.get_mut(0..1) {
        first.make_ascii_uppercase();
    }

    result
}
