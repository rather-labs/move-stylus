module 0x01::vec_struct;

public struct Foo has drop, copy {
    q: address,
    r: vector<u32>,
    s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    bar: Bar,
    baz: Baz,
}

// Static abi sub-struct
public struct Bar has drop, copy {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
public struct Baz has drop, copy {
    a: u16,
    b: vector<u256>,
}

// Forces the compiler to store literals on locals
public fun get_literal(): vector<Foo> {
  vector[
      Foo {
        q: @0x1deadbeef,
        r : vector[1, 3, 0, 3, 4, 5, 6],
        s : vector[1, 5, 4, 3, 0, 3, 0],
        t : true,
        u : 41,
        v : 14242,
        w : 1424242,
        x : 142424242,
        y : 14242424242,
        z : 1424242424242,
        bar: Bar { a: 142, b: 14242 },
        baz: Baz { a: 14242, b: vector[1] },
      },
      Foo {
        q: @0x2deadbeef,
        r : vector[2, 3, 0, 3, 4, 5, 6],
        s : vector[2, 5, 4, 3, 0, 3, 0],
        t : true,
        u : 42,
        v : 24242,
        w : 2424242,
        x : 242424242,
        y : 24242424242,
        z : 2424242424242,
        bar: Bar { a: 242, b: 24242 },
        baz: Baz { a: 24242, b: vector[2] },
      },
      Foo {
        q: @0x3deadbeef,
        r : vector[3, 3, 0, 3, 4, 5, 6],
        s : vector[3, 5, 4, 3, 0, 3, 0],
        t : true,
        u : 43,
        v : 34242,
        w : 3424242,
        x : 342424242,
        y : 34242424242,
        z : 3424242424242,
        bar: Bar { a: 342, b: 34242 },
        baz: Baz { a: 34242, b: vector[3] },
      },
  ]
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): vector<Foo> {
  let x: vector<Foo> = vector[
      Foo {
        q: @0x1deadbeef,
        r : vector[1, 3, 0, 3, 4, 5, 6],
        s : vector[1, 5, 4, 3, 0, 3, 0],
        t : true,
        u : 41,
        v : 14242,
        w : 1424242,
        x : 142424242,
        y : 14242424242,
        z : 1424242424242,
        bar: Bar { a: 142, b: 14242 },
        baz: Baz { a: 14242, b: vector[1] },
      },
      Foo {
        q: @0x2deadbeef,
        r : vector[2, 3, 0, 3, 4, 5, 6],
        s : vector[2, 5, 4, 3, 0, 3, 0],
        t : true,
        u : 42,
        v : 24242,
        w : 2424242,
        x : 242424242,
        y : 24242424242,
        z : 2424242424242,
        bar: Bar { a: 242, b: 24242 },
        baz: Baz { a: 24242, b: vector[2] },
      },
      Foo {
        q: @0x3deadbeef,
        r : vector[3, 3, 0, 3, 4, 5, 6],
        s : vector[3, 5, 4, 3, 0, 3, 0],
        t : true,
        u : 43,
        v : 34242,
        w : 3424242,
        x : 342424242,
        y : 34242424242,
        z : 3424242424242,
        bar: Bar { a: 342, b: 34242 },
        baz: Baz { a: 34242, b: vector[3] },
      },
  ];
  let y = x;
  let _z = x;
  y
}

public fun echo(x: vector<Foo>): vector<Foo> {
  x
}

public fun vec_from_struct(x: Foo, y: Foo): vector<Foo> {
  let z = vector[x, y, x];
  z
}

public fun vec_from_vec(x: vector<Foo>, y: vector<Foo>): vector<vector<Foo>> {
  let z = vector[x, y];
  z
}

public fun vec_from_vec_and_struct(x: vector<Foo>, y: Foo): vector<vector<Foo>> {
  let z = vector[x, vector[y, y]];
  z
}

public fun vec_len(x: vector<Foo>): u64 {
  x.length()
}

public fun vec_pop_back(x: vector<Foo>): vector<Foo> {
  let mut y = x;
  y.pop_back();
  y.pop_back();
  y
}

public fun vec_swap(x: vector<Foo>, id1: u64, id2: u64): vector<Foo> {
  let mut y = x;
  y.swap(id1, id2);
  y
}

public fun vec_push_back(x: vector<Foo>, y: Foo): vector<Foo> {
  let mut z = x;
  z.push_back(y);
  z.push_back(y);
  z
}

public fun vec_push_and_pop_back(x: vector<Foo>, y: Foo): vector<Foo> {
  let mut z = x;
  z.push_back(y);
  z.pop_back();
  z
}
