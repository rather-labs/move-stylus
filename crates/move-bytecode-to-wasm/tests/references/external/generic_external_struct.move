module test::external_generic_struct;

use test::external_generic_struct_defs::{Foo, Bar, get_foo};

public fun deref_struct(x: Foo<u32>): Foo<u32> {
  let y = &x;
  *y
}

public fun deref_struct_ref(y: &Foo<u32>): Foo<u32> {
  *y
}

public fun identity_struct_ref(x: &Foo<u32>): &Foo<u32> {
    x
}

public fun identity_static_struct_ref(x: &Bar<u32>): &Bar<u32> {
    x
}
public fun call_deref_struct_ref(x: Foo<u32>): Foo<u32> {
    deref_struct_ref(&x)
}

public fun deref_nested_struct(x: Foo<u32>): Foo<u32> {
    let y = &x;
    let z = &*y;
    *z
}

public fun deref_mut_arg(x: &mut Foo<u32>): Foo<u32> {
    *x
}

public fun freeze_ref(y: Foo<u32>): Foo<u32> {
    let mut x = get_foo(314);
    let x_mut_ref: &mut Foo<u32> = &mut x;
    *x_mut_ref = y;
    let x_frozen_ref: &Foo<u32> = freeze(x_mut_ref);
    *x_frozen_ref
}

public fun write_ref(x: &mut Foo<u32>, y: Foo<u32>): &mut Foo<u32> {
    *x = y;
    x
}
