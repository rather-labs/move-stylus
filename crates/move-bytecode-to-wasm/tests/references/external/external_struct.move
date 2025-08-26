module test::external_struct;

use test::external_struct_def::{Foo, Bar, get_foo};

public fun deref_struct(x: Foo): Foo {
  let y = &x;
  *y
}

public fun deref_struct_ref(y: &Foo): Foo {
  *y
}

public fun identity_struct_ref(x: &Foo): &Foo {
    x
}

public fun identity_static_struct_ref(x: &Bar): &Bar {
    x
}
public fun call_deref_struct_ref(x: Foo): Foo {
    deref_struct_ref(&x)
}

public fun deref_nested_struct(x: Foo): Foo {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut Foo): Foo {
    *x
}

public fun freeze_ref(y: Foo): Foo {
    let mut x = get_foo();
    let x_mut_ref: &mut Foo = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &Foo = freeze(x_mut_ref);
    *x_frozen_ref
}

public fun write_ref(x: &mut Foo, y: Foo): &mut Foo {
    *x = y;
    x
}
