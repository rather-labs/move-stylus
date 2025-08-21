// This contract was taken from
// https://stylus-saturdays.com/i/167568457/move-on-stylus-an-implementation-overview

module hello_world::dog_walker;

use stylus::event::emit;
use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::tx_context as tx_context;

public struct IWalkTheDog has copy, drop { }

public struct CanWalkDogCap has key { id: UID }

// We replaced the constructor with a create function so we can use it more than once.
public fun create(ctx: &mut TxContext) {
    transfer::transfer(
        CanWalkDogCap { id: object::new(ctx) },
        tx_context::sender(ctx)
    );
}

public fun walk_the_dog(_: &CanWalkDogCap) {
    emit(IWalkTheDog { });
}

