module hello_world::hello_world;

use hello_world::external_generic_struct_defs::{Foo, create_foo};

public struct LocalStruct<T: copy> has drop, copy {
    g: T,
    a: u32,
    b: Foo<T>,
}

public fun structCopy(): Foo<u16> {
    create_foo(314)
}
