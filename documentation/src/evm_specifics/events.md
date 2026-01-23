# Events

Events enable data to be recorded publicly on the blockchain. Each log entry includes the contract’s address, up to four topics, and binary data of arbitrary length.

In Move, events are compatible with the [EVM event model](https://docs.soliditylang.org/en/develop/abi-spec.html#events), allowing seamless interaction between Move contracts and EVM-based tools and libraries.

## Declaring Events

Events are common structs annotated with the `#[ext(event(..)]` attribute. This attribute indicates that the struct is intended to be used as an event.

```move
#[ext(event(indexes = 2))]
public struct Event has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}
```

Events must have the copy and drop abilities. In the example above, the `Event` struct has four fields: `a`, `b`, `c`, and `d`. The `indexes = 2` parameter specifies that the first two fields (`a` and `b`) will be indexed topics in the event log.

You can also declare events as anonymous by annoating `anonymous` inside the event attribute:

```move
#[ext(event(indexes = 2, anonymous))]
public struct Event has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}
```

An anonymous event does not include the event signature as the first topic in the log entry.

> [!NOTE]
> If an event struct has more indexed fields than allowed (maximum of 4 for anonymous events, maximum of 3 for non-anonymous events), the Move compiler will raise a compilation error.


> [!WARNING]
> If an event struct is not annotated with the `#[ext(event(..)]` attribute, it will not be recognized as an event, and attempting to emit it will result in a compilation error.
