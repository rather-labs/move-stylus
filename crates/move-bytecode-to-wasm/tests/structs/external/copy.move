module test::external_struct_copy;

use test::external_struct_defs::{Foo, create_foo};

public fun structCopy(): (Foo, Foo) {
    let foo_1 = create_foo();

    let foo_2 = foo_1;
    (foo_1, foo_2)
}
