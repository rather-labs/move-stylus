module test::external_generic_struct_copy;

use test::external_generic_struct_defs::{Foo, create_foo};

public struct LocalStruct<T: copy> has drop, copy {
    g: T,
    a: u32,
    b: Foo<T>,
}

public fun structCopy(): (Foo<u16>, Foo<u16>) {
    let foo_1 = create_foo(314);

    let foo_2 = foo_1;
    (foo_1, foo_2)
}

public fun structCopy2(): (LocalStruct<u16>, LocalStruct<u16>) {
    let ls_1 = LocalStruct {
        g: 314,
        a: 42,
        b: create_foo(314),
    };

    let ls_2 = ls_1;
    (ls_1, ls_2)
}
