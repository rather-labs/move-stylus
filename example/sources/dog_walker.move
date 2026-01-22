// This contract was taken from
// https://stylus-saturdays.com/i/167568457/move-on-stylus-an-implementation-overview

module hello_world::dog_walker;

use stylus::{
    event::emit, 
    transfer::{Self}, 
    object::{Self, UID}, 
    tx_context::{Self, TxContext}
};

#[ext(event)]
public struct IWalkTheDog has copy, drop { }

public struct CanWalkDogCap has key { id: UID }

// We replaced the constructor with a create function so we can use it more than once.
entry fun create(ctx: &mut TxContext) {
    transfer::transfer(
        CanWalkDogCap { id: object::new(ctx) },
        tx_context::sender(ctx)
    );
}

entry fun walk_the_dog(_: &CanWalkDogCap) {
    emit(IWalkTheDog { });
}

