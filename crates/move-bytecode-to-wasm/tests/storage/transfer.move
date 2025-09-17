module test::transfer;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Foo has key {
    id: UID,
    value: u64
}

public fun create_shared(ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}

public fun create_owned(recipient: address, ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::transfer(foo, recipient);
}

public fun create_frozen(ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::freeze_object(foo);
}

public fun read_value(foo: &Foo): u64 {
    foo.value
}

public fun set_value(foo: &mut Foo, value: u64) {
    foo.value = value;
}

public fun increment_value(foo: &mut Foo) {
    foo.value = foo.value + 1;
}

// Wrappers to manipulate storage directly: delete, transfer, freeze and share object.
public fun delete_obj(foo: Foo) {
    let Foo { id, value: _ } = foo;
    id.delete();
}

public fun delete_obj_2(foo: Foo, foo2: Foo) {
    let Foo { id: id1, value: _ } = foo;
    let Foo { id: id2, value: _ } = foo2;
    id1.delete();
    id2.delete();
}

public fun freeze_obj(foo: Foo) {
  transfer::freeze_object(foo);
}

public fun share_obj(foo: Foo) {
  transfer::share_object(foo);
}

public fun transfer_obj(foo: Foo, recipient: address) {
  transfer::transfer(foo, recipient);
}

public fun get_foo(foo: &Foo): &Foo {
    foo
}

public struct Bar has key {
    id: UID,
    a: u64,
    c: vector<u64>
}

public struct Qux has store {
    a: u64,
    b: u128,
    c: u128
}

public struct Baz has key {
    id: UID,
    a: u64,
    c: Qux
}

public struct Bez has key {
    id: UID,
    a: u64,
    c: vector<Qux>,
    d: vector<vector<u128>>,
    e: u8
}

public struct Quz<T> has store {
    a: T,
    b: u128,
    c: u128
}

public struct Biz<T: copy> has key {
    id: UID,
    a: T,
    b: Quz<T>,
    c: vector<Quz<T>>,
}

public fun create_bar(ctx: &mut TxContext) {
  let bar = Bar {
    id: object::new(ctx),
    a: 101,
    c: vector[1, 2, 3, 4, 5, 6, 7, 8, 9],
  };

  transfer::share_object(bar);
}

public fun get_bar(bar: &Bar): &Bar {
    bar
}

public fun delete_bar(bar: Bar) {
  object::delete(bar);
}

public fun create_baz(recipient: address, share: bool, ctx: &mut TxContext) {
  let baz = Baz {
    id: object::new(ctx),
    a: 101,
    c: Qux { a: 42, b: 55, c: 66 },
  };

  if (share) {
    transfer::share_object(baz);
  } else {
    transfer::transfer(baz, recipient);
  }
}

public fun get_baz(baz: &Baz): &Baz {
    baz
}

public fun delete_baz(baz: Baz) {
  object::delete(baz);
}

public fun create_bez(ctx: &mut TxContext) {
  let bez = Bez {
    id: object::new(ctx),
    a: 101,
    c: vector[Qux { a: 42, b: 55, c: 66 }, Qux { a: 43, b: 56, c: 67 }, Qux { a: 44, b: 57, c: 68 }],
    d: vector[vector[1,2,3], vector[4], vector[], vector[5,6]],
    e: 17,
  };

  transfer::share_object(bez);
}

public fun get_bez(bez: &Bez): &Bez {
    bez
}

public fun delete_bez(bez: Bez) {
  object::delete(bez);
}

public fun create_biz(ctx: &mut TxContext) {
  let biz = Biz<u64> {
    id: object::new(ctx),
    a: 101,
    b: Quz<u64> { a: 42, b: 55, c: 66 },
    c: vector[Quz<u64> { a: 42, b: 55, c: 66 }, Quz<u64> { a: 43, b: 56, c: 67 }, Quz<u64> { a: 44, b: 57, c: 68 }],
  };

  transfer::share_object(biz);
}

public fun get_biz(biz: &Biz<u64>): &Biz<u64> {
    biz
}

public fun delete_biz(biz: Biz<u64>) {
  object::delete(biz);
}
