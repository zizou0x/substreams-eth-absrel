## Substreams Ethereum - Absolute/Relative Problem

It happens that some contracts specifies events to report delta values and on other event using absolute values to represent a single entity.

Uniswap V3 Pool contract is an example of this. The `Mint` and `Burn` events are emitting relative values while `Swap` event emits absolute values for the same entity, one of the pool's token.

### Solution

To achieve that in Substreams, accumulate all the relative values in one store, accumulate all the "latest" value in another store. Then in a mapper/store that depends on those 2 stores, reconcile both values by adding the summed up relative values against the latest value.

- See [store_mint_burn_liquidity](./src/lib.rs#L41-L69) for accumulation of Mint/Burn relative values.
- See [store_swap_liquidity](./src/lib.rs#L72-86) for accumulation of Swap absolute values.
- See [map_output](./src/lib.rs#L89-103) for merging of relative values with absolute values to from a single store value.
