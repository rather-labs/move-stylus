use std::hash::Hasher;

use hashers::jenkins::spooky_hash::SpookyHasher;

const SEED_1: u64 = 0;
const SEED_2: u64 = 0;

pub fn get_hasher() -> impl Hasher {
    SpookyHasher::new(SEED_1, SEED_2)
}
