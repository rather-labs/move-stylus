use std::hash::Hasher;

pub fn get_hasher() -> impl Hasher {
    fxhash::FxHasher::default()
}
