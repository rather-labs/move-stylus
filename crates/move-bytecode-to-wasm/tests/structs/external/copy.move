module test::external_struct_copy;

use test::external_struct_defs::{Foo, create_foo};

public struct LocalStruct has drop, copy {
    a: u32,
    b: Foo,
}

public fun structCopy(): (Foo, Foo) {
    let foo_1 = create_foo();

    let foo_2 = foo_1;
    (foo_1, foo_2)
}

public fun structCopy2(): (LocalStruct, LocalStruct) {
    let ls_1 = LocalStruct {
        a: 42,
        b: create_foo(),
    };

    let ls_2 = ls_1;
    (ls_1, ls_2)
}
