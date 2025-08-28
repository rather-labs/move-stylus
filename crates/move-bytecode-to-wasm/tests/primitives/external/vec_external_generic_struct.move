module test::vec_external_generic_struct;

use test::external_generic_struct_def::{Foo, get_foo_1, get_foo_2, get_foo_3};

// Forces the compiler to store literals on locals
public fun get_literal(): vector<Foo<u32>> {
  vector[
      get_foo_1(1),
      get_foo_2(2),
      get_foo_3(3),
  ]
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<Foo<u32>> {
  let x: vector<Foo<u32>> = vector[
      get_foo_1(1),
      get_foo_2(2),
      get_foo_3(3),
  ];
  let y = x;
  let _z = x;
  y
}

public fun echo(x: vector<Foo<u32>>): vector<Foo<u32>> {
  x
}

public fun vec_from_struct(x: Foo<u32>, y: Foo<u32>): vector<Foo<u32>> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec(x: vector<Foo<u32>>, y: vector<Foo<u32>>): vector<vector<Foo<u32>>> {
  let z = vector[x, y];
  z
}

public fun vec_from_vec_and_struct(x: vector<Foo<u32>>, y: Foo<u32>): vector<vector<Foo<u32>>> {
  let z = vector[x, vector[y, y]];
  z
}

public fun vec_len(x: vector<Foo<u32>>): u64 {
  x.length()
}

public fun vec_pop_back(x: vector<Foo<u32>>): vector<Foo<u32>> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap(x: vector<Foo<u32>>, id1: u64, id2: u64): vector<Foo<u32>> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back(x: vector<Foo<u32>>, y: Foo<u32>): vector<Foo<u32>> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

public fun vec_push_and_pop_back(x: vector<Foo<u32>>, y: Foo<u32>): vector<Foo<u32>> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}

public fun vec_eq(x: vector<Foo<u32>>, y: vector<Foo<u32>>): bool {
    x == y
}

public fun vec_neq(x: vector<Foo<u32>>, y: vector<Foo<u32>>): bool {
    x != y
}

public fun vec_borrow(x: &vector<Foo<u32>>): &Foo<u32> {
    &x[0]
}

public fun vec_mut_borrow(x: &mut vector<Foo<u32>>): &mut Foo<u32> {
    &mut x[0]
}
