module hello_world::hello_world;

// use hello_world::external_generic_struct_defs::{Foo, create_foo};

public struct Foo<T: copy> has drop, copy {
    g: T,
}

public fun create_foo<T: copy>(g: T): Foo<T> {
    Foo {
        g,
    }
}

public fun structCopy(): Foo<u16> {
    create_foo(314)
}
