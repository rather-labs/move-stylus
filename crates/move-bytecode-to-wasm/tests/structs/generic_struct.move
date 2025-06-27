module 0x00::generic_structs;

public struct Foo<T> has drop {
    x: T,
    y: u128,
}

/*
public fun echo_bool(a: bool): Foo<bool> {
    let foo = Foo {
        x: a,
        y: vector[1, 2, 3],
    };

    foo
}

public fun echo_bool(a: u8): Foo<u8> {
    let foo = Foo {
        x: a,
        y: vector[1, 2, 3],
    };

    foo
}

*/
public fun test<T>(a: T): Foo<T> {

     Foo {
        x: a,
        y: vector[1, 2, 3],
    };

}
