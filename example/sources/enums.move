module hello_world::enums;

public enum Foo<phantom T, phantom U> has drop {
    A,
    B,
}

fun pack_foo<T: drop, U: drop>(i: u8): Foo<T, U> {
    match (i) {
        0 => Foo::A,
        1 => Foo::B,
        _ => abort(1),
    }
}

entry fun pack_unpack_foo(variant_index: u8): Foo<u64, u32> {
    pack_foo(variant_index)
}