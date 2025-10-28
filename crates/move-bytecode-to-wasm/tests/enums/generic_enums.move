module 0x00::generic_enums;

public enum Foo<T, U> has drop {
    A(T),
    B(U),
    C(T, U),
}

fun pack_foo<T: drop, U: drop>(i: u8, t: T, u: U): Foo<T, U> {
    match (i) {
        0 => Foo<T, U>::A(t),
        1 => Foo<T, U>::B(u),
        2 => Foo<T, U>::C(t, u),
        _ => abort(1),
    }
}

fun pack_foo_wrapper<U: drop>(i: u8, t: u64, u: U): Foo<u64, U> {
    pack_foo(i, t, u)
}

entry fun pack_unpack_foo(variant_index: u8, t: u64, u: u32): (u64, u32) {
    let enum_ = pack_foo(variant_index, t, u);
    match (enum_) {
        Foo::A(z) => (z, u),
        Foo::B(z) => (t, z),
        Foo::C(x, y) => (x, y),
    }
}

entry fun pack_unpack_foo_via_wrapper(i: u8, t: u64, u: u32): (u64, u32) {
    let enum_ = pack_foo_wrapper(i, t, u);
    match (enum_) {
        Foo::A(z) => (z, u),
        Foo::B(z) => (t, z),
        Foo::C(x, y) => (x, y),
    }
}

fun pack_foo_wrapper_2<U: drop + copy>(i: u8, t: u64, u: U): Foo<u64, vector<U>> {
    let v = vector[u, u, u];
    pack_foo(i, t, v)
}

entry fun pack_unpack_foo_via_wrapper_2(i: u8, t: u64, u: u32): vector<u32> {
    let enum_ = pack_foo_wrapper_2(i, t, u);
    match (enum_) {
        Foo::A(_) => vector[],
        Foo::B(x) => x,
        Foo::C(_, y) => y,
    }
}

public enum Bar<T> has drop, copy {
    A(T),
    B(T),
    C{x: T},
}

public enum Baz<T> has drop {
    X(Bar<T>),
    Y(vector<Bar<T>>),
    Z{x: Bar<T>, y: Bar<T>},
}

fun pack_baz<T: drop + copy>(i: u8, t: T): Baz<T> {
    match (i) {
        0 => Baz<T>::X(Bar<T>::A(t)),
        1 => Baz<T>::Y(vector[Bar<T>::A(t), Bar<T>::B(t), Bar<T>::C{x: t}]),
        2 => Baz<T>::Z{x: Bar<T>::A(t), y: Bar<T>::B(t)},
        _ => abort(1),
    }
}

fun unpack_bar<T: drop + copy>(b: Bar<T>): T {
    match (b) {
        Bar<T>::A(t) => t,
        Bar<T>::B(t) => t,
        Bar<T>::C{x} => x,
    }
}

entry fun pack_unpack_baz(i: u8, t: u32): u32 {
    let baz = pack_baz(i, t);
    match (baz) {
        Baz<u32>::X(bar) => {
            unpack_bar(bar)
        },
        Baz<u32>::Y(bar_vec) => {
            unpack_bar(bar_vec[0]) + unpack_bar(bar_vec[1]) + unpack_bar(bar_vec[2])
        },
        Baz<u32>::Z{x, y} => {
            unpack_bar(x) + unpack_bar(y)
        },
    }
}

public enum Fu<T, U> has drop {
    A {
        t: T,
        u: U,
    },
    B(T, U),
    C{t: T, u: U},
}

fun pack_fu<T: drop, U: drop>(i: u8, t: T, u: U): Fu<T, U> {
    match (i) {
        0 => Fu<T, U>::A { t, u },
        1 => Fu<T, U>::B(t, u),
        2 => Fu<T, U>::C{t, u},
        _ => abort(1),
    }
}

entry fun pack_mutate_unpack_fu(i: u8, t: u64, u: u32): (u64, u32) {
    let fu = &mut pack_fu(i, t, u);
    match (fu) {
        Fu<u64, u32>::A { t, u } => {
            *t = *t + 1;
            *u = *u + 1;
        },
        Fu<u64, u32>::B(t, u) => {
            *t = *t + 1;
            *u = *u + 1;
        },
        Fu<u64, u32>::C{t, u} => {
            *t = *t + 1;
            *u = *u + 1;
        },
    };
    match (fu) {
        Fu<u64, u32>::A { t, u } => (*t, *u),
        Fu<u64, u32>::B(t, u) => (*t, *u),
        Fu<u64, u32>::C{t, u} => (*t, *u),
    }
}