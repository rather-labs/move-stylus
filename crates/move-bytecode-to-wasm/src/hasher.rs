// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use std::hash::Hasher;

pub fn get_hasher() -> impl Hasher {
    fxhash::FxHasher::default()
}
