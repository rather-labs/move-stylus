module test::wrapped_objects;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

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

// Struct wrapping another struct without the key ability
// but which in turn has a vector of wrapped objects as field
public struct Zeta has key {
    id: UID,
    b: Astra
}

// Struct wrapping another struct without the key ability
// containing only vectors of primitive types
public struct Eta has key {
    id: UID,
    a: Bora
}

// This struct does not have the key ability, even though it has a vector of wrapped objects
public struct Astra has store {
    a: vector<Alpha>
}

// Struct containing only vectors of primitive types
public struct Bora has store {
    a: vector<u64>,
    b: vector<vector<u64>>
}

// ============================================================================
// FUNCTION DEFINITIONS
// ============================================================================

entry fun create_alpha(value: u64, ctx: &mut TxContext) {
    let alpha = Alpha { id: object::new(ctx), value };
    transfer::transfer(alpha, ctx.sender());
}

// Creating the object to wrap inside the function
entry fun create_beta(ctx: &mut TxContext) {
    let alpha = Alpha { id: object::new(ctx), value: 101 };
    let beta = Beta { id: object::new(ctx), a: alpha };
    transfer::transfer(beta, ctx.sender());
}

entry fun create_gamma(ctx: &mut TxContext) {
    let alpha = Alpha { id: object::new(ctx), value: 101 };
    let beta = Beta { id: object::new(ctx), a: alpha };
    let gamma = Gamma { id: object::new(ctx), a: beta };
    transfer::transfer(gamma, ctx.sender());
}

entry fun create_delta(ctx: &mut TxContext) {
    let alpha_1 = Alpha { id: object::new(ctx), value: 101 };
    let alpha_2 = Alpha { id: object::new(ctx), value: 102 };
    let delta = Delta { id: object::new(ctx), a: vector[alpha_1, alpha_2] };
    transfer::transfer(delta, ctx.sender());
}

entry fun create_empty_delta(ctx: &mut TxContext) {
    let delta = Delta { id: object::new(ctx), a: vector[] };
    transfer::transfer(delta, ctx.sender());
}

entry fun create_epsilon(ctx: &mut TxContext) {
    let delta_1 = Delta { id: object::new(ctx), a: vector[Alpha { id: object::new(ctx), value: 101 }, Alpha { id: object::new(ctx), value: 102 }] };
    let delta_2 = Delta { id: object::new(ctx), a: vector[Alpha { id: object::new(ctx), value: 103 }, Alpha { id: object::new(ctx), value: 104 }] };
    let epsilon = Epsilon { id: object::new(ctx), a: vector[delta_1, delta_2] };
    transfer::transfer(epsilon, ctx.sender());
}

// Receiving the object to wrap by argument
entry fun create_beta_tto(alpha: Alpha, ctx: &mut TxContext) {
    let beta = Beta { id: object::new(ctx), a: alpha };
    transfer::transfer(beta, ctx.sender());
}

entry fun create_gamma_tto(beta: Beta, ctx: &mut TxContext) {
    let gamma = Gamma { id: object::new(ctx), a: beta };
    transfer::transfer(gamma, ctx.sender());
}

entry fun create_delta_tto(alpha_1: Alpha, alpha_2: Alpha, ctx: &mut TxContext) {
    let delta = Delta { id: object::new(ctx), a: vector[alpha_1, alpha_2] };
    transfer::transfer(delta, ctx.sender());
}

entry fun create_epsilon_tto(delta_1: Delta, delta_2: Delta, ctx: &mut TxContext) {
    let epsilon = Epsilon { id: object::new(ctx), a: vector[delta_1, delta_2] };
    transfer::transfer(epsilon, ctx.sender());
}

entry fun create_empty_zeta(ctx: &mut TxContext) {
    let astra = Astra { a: vector[] };
    let zeta = Zeta { id: object::new(ctx), b: astra };
    transfer::transfer(zeta, ctx.sender());
}

entry fun create_eta(ctx: &mut TxContext) {
    let bora = Bora { a: vector[], b: vector[] };
    let eta = Eta { id: object::new(ctx), a: bora };
    transfer::transfer(eta, ctx.sender());
}

// Reading structs
entry fun read_alpha(alpha: &Alpha): &Alpha {
    alpha
}

entry fun read_beta(beta: &Beta): &Beta {
    beta
}

entry fun read_gamma(gamma: &Gamma): &Gamma {
    gamma
}

entry fun read_delta(delta: &Delta): &Delta {
    delta
}

entry fun read_epsilon(epsilon: &Epsilon): &Epsilon {
    epsilon
}

entry fun read_zeta(zeta: &Zeta): &Zeta {
    zeta
}

entry fun read_eta(eta: &Eta): &Eta {
    eta
}

entry fun delete_alpha(alpha: Alpha) {
    let Alpha { id, value: _ } = alpha;
    id.delete();
}

// Deleting structs
entry fun delete_beta(beta: Beta) {
    let Beta { id, a: alpha } = beta;
    id.delete();
    let Alpha { id, value: _ } = alpha;
    id.delete();
}

entry fun delete_gamma(gamma: Gamma) {
    let Gamma { id, a: beta } = gamma;
    id.delete();
    let Beta { id, a: alpha } = beta;
    id.delete();
    let Alpha { id, value: _ } = alpha;
    id.delete();
}

entry fun delete_delta(delta: Delta) {
    let Delta { id, a: mut vector_alpha } = delta;
    id.delete();
    while (!vector::is_empty(&vector_alpha)) {
        let alpha = vector::pop_back(&mut vector_alpha);
        let Alpha { id, value: _ } = alpha;
        id.delete();
    };
    vector::destroy_empty(vector_alpha);
}

entry fun delete_epsilon(epsilon: Epsilon) {
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

entry fun delete_zeta(zeta: Zeta) {
    let Zeta { id, b: astra } = zeta;
    id.delete();
    let Astra { a: mut vector_alpha } = astra;
    while (!vector::is_empty(&vector_alpha)) {
        let alpha = vector::pop_back(&mut vector_alpha);
        let Alpha { id, value: _ } = alpha;
        id.delete();
    };
    vector::destroy_empty(vector_alpha);
}

// Transferring structs
entry fun transfer_beta(beta: Beta, recipient: address) {
    transfer::transfer(beta, recipient);
}

entry fun transfer_gamma(gamma: Gamma, recipient: address) {
    transfer::transfer(gamma, recipient);
}

entry fun transfer_delta(delta: Delta, recipient: address) {
    transfer::transfer(delta, recipient);
}

entry fun transfer_zeta(zeta: Zeta, recipient: address) {
    transfer::transfer(zeta, recipient);
}

// Miscellaneous operations on structs

// Destructs gamma and wraps beta in a new gamma
entry fun rebuild_gamma(gamma: Gamma, recipient: address, ctx: &mut TxContext) {
    let Gamma { id, a: beta } = gamma;
    id.delete();
    let new_gamma = Gamma { id: object::new(ctx), a: beta };
    transfer::transfer(new_gamma, recipient);
}

// Destructs delta and wraps each alpha in a beta
entry fun destruct_delta_to_beta(delta: Delta, ctx: &mut TxContext) {
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
entry fun push_alpha_to_delta(delta: &mut Delta, alpha: Alpha) {
    delta.a.push_back(alpha);
}

// Popping Alpha from Delta
entry fun pop_alpha_from_delta(delta: &mut Delta) {
    let alpha = delta.a.pop_back();
    transfer::share_object(alpha);
}

entry fun destruct_epsilon(epsilon: Epsilon, alpha: Alpha, ctx: &mut TxContext) {
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

entry fun push_alpha_to_zeta(zeta: &mut Zeta, alpha: Alpha) {
    zeta.b.a.push_back(alpha);
}

entry fun pop_alpha_from_zeta(zeta: &mut Zeta) {
    let alpha = zeta.b.a.pop_back();
    transfer::share_object(alpha);
}

entry fun push_to_bora(eta: &mut Eta, value: u64) {
    eta.a.a.push_back(value);
    let v = vector[value, value + 1, value + 2];
    eta.a.b.push_back(v);
}

entry fun pop_from_bora(eta: &mut Eta): (u64, vector<u64>) {
    (eta.a.a.pop_back(), eta.a.b.pop_back())
}
