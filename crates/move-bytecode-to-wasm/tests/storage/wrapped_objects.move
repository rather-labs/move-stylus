module test::wrapped_objects;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

// ============================================================================
// STRUCT DEFINITIONS
// ============================================================================

// Simple struct with a single value field
public struct Alpha has key, store {
    id: UID,
    value: u64
}

// Struct with a wrapped object field
public struct Beta has key, store {
    id: UID,
    a: Alpha,
}

// Struct with a wrapped object field, which in turn has a wrapped object field
public struct Gamma has key {
    id: UID,
    a: Beta,
}

// Struct with a vector of wrapped object fields
public struct Delta has key, store {
    id: UID,
    a: vector<Alpha>,
}

// Struct with a vector of wrapped objects, which in turn have a vector of wrapped objects as field
public struct Epsilon has key {
    id: UID,
    a: vector<Delta>,
}

// ============================================================================
// FUNCTION DEFINITIONS
// ============================================================================

public fun create_alpha(value: u64, ctx: &mut TxContext) {
    let alpha = Alpha { id: object::new(ctx), value };
    transfer::transfer(alpha, ctx.sender());
}

// Creating the object to wrap inside the function
public fun create_beta(ctx: &mut TxContext) {
    let alpha = Alpha { id: object::new(ctx), value: 101 };
    let beta = Beta { id: object::new(ctx), a: alpha };
    transfer::transfer(beta, ctx.sender());
}

public fun create_gamma(ctx: &mut TxContext) {
    let alpha = Alpha { id: object::new(ctx), value: 101 };
    let beta = Beta { id: object::new(ctx), a: alpha };
    let gamma = Gamma { id: object::new(ctx), a: beta };
    transfer::transfer(gamma, ctx.sender());
}

public fun create_delta(ctx: &mut TxContext) {
    let alpha_1 = Alpha { id: object::new(ctx), value: 101 };
    let alpha_2 = Alpha { id: object::new(ctx), value: 102 };
    let delta = Delta { id: object::new(ctx), a: vector[alpha_1, alpha_2] };
    transfer::transfer(delta, ctx.sender());
}

public fun create_empty_delta(ctx: &mut TxContext) {
    let delta = Delta { id: object::new(ctx), a: vector[] };
    transfer::transfer(delta, ctx.sender());
}

public fun create_epsilon(ctx: &mut TxContext) {
    let delta_1 = Delta { id: object::new(ctx), a: vector[Alpha { id: object::new(ctx), value: 101 }, Alpha { id: object::new(ctx), value: 102 }] };
    let delta_2 = Delta { id: object::new(ctx), a: vector[Alpha { id: object::new(ctx), value: 103 }, Alpha { id: object::new(ctx), value: 104 }] };
    let epsilon = Epsilon { id: object::new(ctx), a: vector[delta_1, delta_2] };
    transfer::transfer(epsilon, ctx.sender());
}

// Receiving the object to wrap by argument
public fun create_beta_tto(alpha: Alpha, ctx: &mut TxContext) {
    let beta = Beta { id: object::new(ctx), a: alpha };
    transfer::transfer(beta, ctx.sender());
}

public fun create_gamma_tto(beta: Beta, ctx: &mut TxContext) {
    let gamma = Gamma { id: object::new(ctx), a: beta };
    transfer::transfer(gamma, ctx.sender());
}

public fun create_delta_tto(alpha_1: Alpha, alpha_2: Alpha, ctx: &mut TxContext) {
    let delta = Delta { id: object::new(ctx), a: vector[alpha_1, alpha_2] };
    transfer::transfer(delta, ctx.sender());
}

public fun create_epsilon_tto(delta_1: Delta, delta_2: Delta, ctx: &mut TxContext) {
    let epsilon = Epsilon { id: object::new(ctx), a: vector[delta_1, delta_2] };
    transfer::transfer(epsilon, ctx.sender());
}

// Reading structs
public fun read_alpha(alpha: &Alpha): &Alpha {
    alpha
}

public fun read_beta(beta: &Beta): &Beta {
    beta
}

public fun read_gamma(gamma: &Gamma): &Gamma {
    gamma
}

public fun read_delta(delta: &Delta): &Delta {
    delta
}

public fun read_epsilon(epsilon: &Epsilon): &Epsilon {
    epsilon
}

public fun delete_alpha(alpha: Alpha) {
    let Alpha { id, value: _ } = alpha;
    id.delete();
}

// Deleting structs
public fun delete_beta(beta: Beta) {
    let Beta { id, a: alpha } = beta;
    id.delete();
    let Alpha { id, value: _ } = alpha;
    id.delete();
}

public fun delete_gamma(gamma: Gamma) {
    let Gamma { id, a: beta } = gamma;
    id.delete();
    let Beta { id, a: alpha } = beta;
    id.delete();
    let Alpha { id, value: _ } = alpha;
    id.delete();
}

public fun delete_delta(delta: Delta) {
    let Delta { id, a: mut vector_alpha } = delta;
    id.delete();
    while (!vector::is_empty(&vector_alpha)) {
        let alpha = vector::pop_back(&mut vector_alpha);
        let Alpha { id, value: _ } = alpha;
        id.delete();
    };
    vector::destroy_empty(vector_alpha);
}

public fun delete_epsilon(epsilon: Epsilon) {
    let Epsilon { id, a: mut vector_delta } = epsilon;
    id.delete();
    while (!vector::is_empty(&vector_delta)) {
        let delta = vector::pop_back(&mut vector_delta);
        let Delta { id, a: mut vector_alpha } = delta;
        id.delete();
        while (!vector::is_empty(&vector_alpha)) {
            let alpha = vector::pop_back(&mut vector_alpha);
            let Alpha { id, value: _ } = alpha;
            id.delete();
        };
        vector::destroy_empty(vector_alpha);
    };
    vector::destroy_empty(vector_delta);
}

// Transferring structs
public fun transfer_beta(beta: Beta, recipient: address) {
    transfer::transfer(beta, recipient);
}

public fun transfer_gamma(gamma: Gamma, recipient: address) {
    transfer::transfer(gamma, recipient);
}

public fun transfer_delta(delta: Delta, recipient: address) {
    transfer::transfer(delta, recipient);
}

// Miscellaneous operations on structs

// Destructs gamma and wraps beta in a new gamma
public fun rebuild_gamma(gamma: Gamma, recipient: address, ctx: &mut TxContext) {
    let Gamma { id, a: beta } = gamma;
    id.delete();
    let new_gamma = Gamma { id: object::new(ctx), a: beta };
    transfer::transfer(new_gamma, recipient);
}

// Destructs delta and wraps each alpha in a beta
public fun destruct_delta_to_beta(delta: Delta, ctx: &mut TxContext) {
    let Delta { id, a: mut vector_alpha } = delta;
    id.delete();
    while (!vector::is_empty(&vector_alpha)) {
        let alpha = vector::pop_back(&mut vector_alpha);
        let beta = Beta { id: object::new(ctx), a: alpha };
        transfer::share_object(beta);
    };
    vector::destroy_empty(vector_alpha);
}

// Pushing Alpha to Delta
public fun push_alpha_to_delta(delta: &mut Delta, alpha: Alpha) {
    delta.a.push_back(alpha);
}

// Popping Alpha from Delta
public fun pop_alpha_from_delta(delta: &mut Delta) {
    let alpha = delta.a.pop_back();
    transfer::share_object(alpha);
}

public fun destruct_epsilon(epsilon: Epsilon, alpha: Alpha, ctx: &mut TxContext) {
    let Epsilon { id, a: mut vector_delta } = epsilon;
    id.delete();

    if (!vector::is_empty(&vector_delta)) {
        let mut delta = vector::pop_back(&mut vector_delta);
        delta.a.push_back(alpha);
        transfer::transfer(delta, ctx.sender());
    } else {
        let Alpha { id, value: _ } = alpha;
        id.delete();
    };

    let new_epsilon = Epsilon { id: object::new(ctx), a: vector_delta };
    transfer::share_object(new_epsilon);
}
