// This contract was taken from
// https://stylus-saturdays.com/i/167568457/move-on-stylus-an-implementation-overview

module hello_world::dog_walker;

use stylus::event::emit;
use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::tx_context as tx_context;

public struct IWalkTheDog has copy, drop { n: u32 }

public struct CanWalkDogCap has key { id: UID, n: u32 }

public fun create(ctx: &mut TxContext) {
    transfer::transfer(
        CanWalkDogCap { id: object::new(ctx), n: 42 },
        tx_context::sender(ctx)
    );
}

public fun walk_the_dog(d: &CanWalkDogCap) {
    emit(IWalkTheDog { n: d.n });
}

