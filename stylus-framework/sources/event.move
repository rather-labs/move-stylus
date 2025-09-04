/// Events module.
///
/// Defines the functions to publicly log data in the blockchain.
///
/// For more information:
/// https://docs.soliditylang.org/en/v0.8.19/abi-spec.html#events
/// https://docs.arbitrum.io/stylus-by-example/basic_examples/events
module stylus::event;

/// Emits an event in the topic 0.
///
/// This function It ensures that an event will be logged in a Solidity ABI-compatible format.
public native fun emit<T: copy + drop>(event: T);
