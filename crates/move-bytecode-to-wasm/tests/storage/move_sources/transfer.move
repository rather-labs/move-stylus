module test::transfer;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

// ============================================================================
// STRUCT DEFINITIONS
// ============================================================================

// Simple struct with a single value field
public struct Foo has key, store {
    id: UID,
    value: u64
}

// Struct with a vector field
public struct Bar has key, store {
    id: UID,
    a: u64,
    c: vector<u64>
}

// Simple value struct (no key)
public struct Qux has store, drop {
    a: u64,
    b: u128,
    c: u128
}

// Struct with nested field struct
public struct Baz has key {
    id: UID,
    a: u64,
    c: Qux
}

// Complex struct with nested vectors
public struct Bez has key {
    id: UID,
    a: u64,
    c: vector<Qux>,
    d: vector<vector<u128>>,
    e: u8
}

// Generic value struct
public struct Quz<T> has store, drop {
    a: T,
    b: u128,
    c: u128
}

// Generic struct with key and nested field struct
public struct Biz<T: copy> has key {
    id: UID,
    a: T,
    b: Quz<T>,
    c: vector<Quz<T>>,
}

public struct Var has key {
    id: UID,
    a: Bar,
}

public struct Vaz has key {
    id: UID,
    a: u32,
    b: Bar,
    c: u64,
    d: Bar
}

public struct EpicVar has key {
    id: UID,
    a: u32,
    b: Bar,
    c: u64,
    d: vector<Bar>,
}

// ============================================================================
// FOO FUNCTIONS
// ============================================================================

entry fun create_shared(ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        value: 101,
    };
    transfer::share_object(foo);
}

entry fun create_owned(recipient: address, ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        value: 101,
    };
    transfer::transfer(foo, recipient);
}

entry fun create_frozen(ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        value: 101,
    };
    transfer::freeze_object(foo);
}

entry fun read_value(foo: &Foo): u64 {
    foo.value
}

entry fun set_value(foo: &mut Foo, value: u64) {
    foo.value = value;
}

entry fun increment_value(foo: &mut Foo) {
    foo.value = foo.value + 1;
}

entry fun get_foo(foo: &Foo): &Foo {
    foo
}

entry fun delete_obj(foo: Foo) {
    let Foo { id, value: _ } = foo;
    id.delete();
}

entry fun delete_obj_2(foo: Foo, foo2: Foo) {
    let Foo { id: id1, value: _ } = foo;
    let Foo { id: id2, value: _ } = foo2;
    id1.delete();
    id2.delete();
}

entry fun freeze_obj(foo: Foo) {
    transfer::freeze_object(foo);
}

entry fun call_indirect_freeze_obj(foo: Foo) {
    freeze_obj(foo);
}

entry fun share_obj(foo: Foo) {
    transfer::share_object(foo);
}

entry fun transfer_obj(foo: Foo, recipient: address) {
    transfer::transfer(foo, recipient);
}

// ============================================================================
// BAR FUNCTIONS
// ============================================================================

entry fun create_bar(ctx: &mut TxContext) {
    let bar = Bar {
        id: object::new(ctx),
        a: 101,
        c: vector[1, 2, 3, 4, 5, 6, 7, 8, 9],
    };
    transfer::share_object(bar);
}

entry fun get_bar(bar: &Bar): &Bar {
    bar
}

entry fun delete_bar(bar: Bar) {
    let Bar { id, a: _, c: _ } = bar;
    id.delete();
}

// ============================================================================
// BAZ FUNCTIONS
// ============================================================================

entry fun create_baz(recipient: address, share: bool, ctx: &mut TxContext) {
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

entry fun get_baz(baz: &Baz): &Baz {
    baz
}

entry fun delete_baz(baz: Baz) {
    let Baz { id, a: _, c: _ } = baz;
    id.delete();
}

// ============================================================================
// BEZ FUNCTIONS
// ============================================================================

entry fun create_bez(ctx: &mut TxContext) {
    let bez = Bez {
        id: object::new(ctx),
        a: 101,
        c: vector[
            Qux { a: 42, b: 55, c: 66 },
            Qux { a: 43, b: 56, c: 67 },
            Qux { a: 44, b: 57, c: 68 }
        ],
        d: vector[
            vector[1, 2, 3],
            vector[4],
            vector[],
            vector[5, 6]
        ],
        e: 17,
    };
    transfer::share_object(bez);
}

entry fun get_bez(bez: &Bez): &Bez {
    bez
}

entry fun delete_bez(bez: Bez) {
    let Bez { id, a: _, c: _, d: _, e: _ } = bez;
    id.delete();
}

// ============================================================================
// BIZ FUNCTIONS
// ============================================================================

entry fun create_biz(ctx: &mut TxContext) {
    let biz = Biz<u64> {
        id: object::new(ctx),
        a: 101,
        b: Quz<u64> { a: 42, b: 55, c: 66 },
        c: vector[
            Quz<u64>{ a: 42, b: 55, c: 66 },
            Quz<u64>{ a: 43, b: 56, c: 67 },
            Quz<u64>{ a: 44, b: 57, c: 68 },
        ],
    };
    transfer::share_object(biz);
}

entry fun get_biz(biz: &Biz<u64>): &Biz<u64> {
    biz
}

entry fun delete_biz(biz: Biz<u64>) {
    let Biz { id, a: _, b: _, c: _ } = biz;
    id.delete();
}

// ============================================================================
// VAR FUNCTIONS
// ============================================================================

entry fun create_var(recipient: address, ctx: &mut TxContext) {
    let var = Var {
        id: object::new(ctx),
        a: Bar { id: object::new(ctx), a: 42, c: vector[1, 2, 3] },
    };
    transfer::transfer(var, recipient);
}

entry fun create_var_shared(ctx: &mut TxContext) {
    let var = Var {
        id: object::new(ctx),
        a: Bar { id: object::new(ctx), a: 42, c: vector[1, 2, 3] },
    };
    transfer::share_object(var);
}

entry fun share_var(var: Var) {
    transfer::share_object(var);
}

entry fun freeze_var(var: Var) {
    transfer::freeze_object(var);
}

entry fun get_var(var: &Var): &Var {
    var
}

entry fun delete_var(var: Var) {
    let Var { id, a: bar } = var;
    let Bar { id: bar_id, a: _, c: _ } = bar;
    bar_id.delete();
    id.delete();
}

entry fun delete_var_and_transfer_bar(var: Var) {
    let Var { id, a: bar } = var;
    id.delete();
    transfer::share_object(bar);
}

// ============================================================================
// VAZ FUNCTIONS
// ============================================================================

entry fun create_vaz(ctx: &mut TxContext) {
    let vaz = Vaz {
        id: object::new(ctx),
        a: 101,
        b: Bar { id: object::new(ctx), a: 42, c: vector[1, 2, 3] },
        c: 102,
        d: Bar { id: object::new(ctx), a: 43, c: vector[4, 5, 6] },
    };
    transfer::share_object(vaz);
}

entry fun get_vaz(vaz: &Vaz): &Vaz {
    vaz
}

entry fun delete_vaz(vaz: Vaz) {
    let Vaz { id, a: _, b: bar1, c: _ , d: bar2} = vaz;
    let Bar { id: bar_id1, a: _, c: _ } = bar1;
    let Bar { id: bar_id2, a: _, c: _ } = bar2;
    bar_id1.delete();
    bar_id2.delete();
    id.delete();
}

// ============================================================================
// EPIC VAR FUNCTIONS
// ============================================================================

entry fun create_epic_var(ctx: &mut TxContext) {
    let epic_var = EpicVar {
        id: object::new(ctx),
        a: 101,
        b: Bar { id: object::new(ctx), a: 41, c: vector[1, 2, 3] },
        c: 102,
        d: vector[Bar { id: object::new(ctx), a: 42, c: vector[42, 43] }, Bar { id: object::new(ctx), a: 43, c: vector[44, 45, 46] }],
    };
    transfer::share_object(epic_var);
}

entry fun get_epic_var(epic_var: &EpicVar): &EpicVar {
    epic_var
}

entry fun delete_epic_var(epic_var: EpicVar) {
    let EpicVar { id, a: _, b: bar, c: _, d: mut vector_bar } = epic_var;
    id.delete();
    let Bar { id, a: _, c: _ } = bar;
    id.delete();
    
    // Iterate through the vector and delete each Bar
    while (!vector::is_empty(&vector_bar)) {
        let bar = vector::pop_back(&mut vector_bar);
        let Bar { id, a: _, c: _ } = bar;
        id.delete();
    };
    
    // Consume the empty vector
    vector::destroy_empty(vector_bar);
}