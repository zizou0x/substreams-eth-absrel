## Substreams Ethereum - Absolute/Relative Problem

It happens that some contracts specifies events to report delta values and on other event using absolute values to represent a single entity.

Uniswap V3 Pool contract is an example of this. The `Mint` and `Burn` events are emitting relative values while `Swap` event emits absolute values for the same entity, one of the pool's token.

