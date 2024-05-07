mod abi;

use substreams::hex;
use substreams::scalar::BigInt;
use substreams::store::StoreAdd;
use substreams::store::StoreAddBigInt;
use substreams::store::StoreGet;
use substreams::store::StoreGetBigInt;
use substreams::store::StoreNew;
use substreams::store::StoreSet;
use substreams::store::StoreSetBigInt;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

const WETH_USDC_POOL_ADDR: [u8; 20] = hex!("88e6a0c2ddd26feeb64f039a2c41296fcb3f5640");

// Should be in it's own mapper, for simplicity, we extract from full block directly
enum PoolEvent {
    Mint {
        ordinal: u64,
        amount: BigInt,
        tick_lower: BigInt,
        tick_upper: BigInt,
        tx: String,
    },
    Burn {
        ordinal: u64,
        amount: BigInt,
        tick_lower: BigInt,
        tick_upper: BigInt,
        tx: String,
    },
    Swap {
        ordinal: u64,
        liquidity: BigInt,
        tick: BigInt,
        tx: String,
    },
    Initialize {
        ordinal: u64,
        tick: BigInt,
    },
}

#[substreams::handlers::store]
fn store_pool_current_tick(blk: Block, s: StoreSetBigInt) {
    for event in block_to_events(blk) {
        match event {
            PoolEvent::Swap { ordinal, tick, .. } => {
                s.set(ordinal, "tick", &tick);
            }
            PoolEvent::Initialize { ordinal, tick, .. } => {
                s.set(ordinal, "tick", &tick);
            }
            _ => {}
        }
    }
}

#[substreams::handlers::store]
fn store_mint_burn_liquidity(blk: Block, tick_store: StoreGetBigInt, s: StoreAddBigInt) {
    for event in block_to_events(blk) {
        match event {
            PoolEvent::Mint {
                ordinal,
                amount,
                tick_lower,
                tick_upper,
                tx,
            } => {
                substreams::log::info!("Mint at tx {}", tx);
                let current_tick = tick_store.get_last("tick").unwrap_or(BigInt::zero());
                if current_tick > tick_lower && current_tick < tick_upper {
                    s.add(ordinal, "liquidity", &amount);
                }
            }
            PoolEvent::Burn {
                ordinal,
                amount,
                tick_lower,
                tick_upper,
                tx,
            } => {
                substreams::log::info!("Burn at tx {}", tx);
                let current_tick = tick_store.get_last("tick").unwrap_or(BigInt::zero());

                if current_tick > tick_lower && current_tick < tick_upper {
                    s.add(ordinal, "liquidity", &amount.neg());
                }
            }
            _ => {}
        }
    }
}

#[substreams::handlers::store]
fn store_swap_liquidity(blk: Block, s: StoreSetBigInt) {
    for event in block_to_events(blk) {
        if let PoolEvent::Swap {
            ordinal,
            liquidity,
            tx,
            ..
        } = event
        {
            substreams::log::info!("Swap at tx {}", tx);
            s.set(ordinal, "liquidity", &liquidity);
        }
    }
}

#[substreams::handlers::map]
fn map_output(mint_burn: StoreGetBigInt, swap: StoreGetBigInt) -> Option<()> {
    let mint_burn_last = mint_burn.get_last("liquidity").unwrap_or_default();
    let swap_last = swap.get_last("liquidity").unwrap_or_default();

    substreams::log::info!(
        "Liquidity at end of block: {}",
        (swap_last + mint_burn_last),
    );

    Some(())
}

fn block_to_events(blk: Block) -> Vec<PoolEvent> {
    use abi::pool::events::{Burn, Initialize, Mint, Swap};

    let events = blk
        .logs()
        .filter_map(|log_view| {
            if log_view.address() != WETH_USDC_POOL_ADDR {
                return None;
            }

            if let Some(mint) = Mint::match_and_decode(log_view.log) {
                Some(PoolEvent::Mint {
                    ordinal: log_view.ordinal(),
                    amount: mint.amount,
                    tick_lower: mint.tick_lower,
                    tick_upper: mint.tick_upper,
                    tx: Hex(&log_view.receipt.transaction.hash).to_string(),
                })
            } else if let Some(burn) = Burn::match_and_decode(log_view.log) {
                Some(PoolEvent::Burn {
                    ordinal: log_view.ordinal(),
                    amount: burn.amount,
                    tick_lower: burn.tick_lower,
                    tick_upper: burn.tick_upper,
                    tx: Hex(&log_view.receipt.transaction.hash).to_string(),
                })
            } else if let Some(swap) = Swap::match_and_decode(log_view.log) {
                Some(PoolEvent::Swap {
                    ordinal: log_view.ordinal(),
                    liquidity: swap.liquidity,
                    tick: swap.tick,
                    tx: Hex(&log_view.receipt.transaction.hash).to_string(),
                })
            } else if let Some(init) = Initialize::match_and_decode(log_view.log) {
                Some(PoolEvent::Initialize {
                    ordinal: log_view.ordinal(),
                    tick: init.tick,
                })
            } else {
                None
            }
        })
        .collect();

    return events;
}
