//! Chunked log fetching helpers for HyperEVM.

use alloy::{
    providers::Provider,
    rpc::types::{Filter, Log},
};
use anyhow::{Context, Result};

/// Default maximum block span per `eth_getLogs` request on HyperEVM.
pub const DEFAULT_LOG_CHUNK_BLOCKS: u64 = 10_000;

/// Fetches logs in backward chunks from `from_block` down to `to_block` (inclusive).
///
/// Each chunk spans at most `chunk_size` blocks. Returns logs in discovery order
/// (newest chunk first, within-chunk RPC order preserved).
pub async fn get_logs_chunked<P: Provider>(
    provider: &P,
    filter: Filter,
    from_block: u64,
    to_block: u64,
    chunk_size: u64,
) -> Result<Vec<Log>> {
    anyhow::ensure!(to_block <= from_block, "to_block must be <= from_block");
    anyhow::ensure!(chunk_size > 0, "chunk_size must be positive");

    let mut out = Vec::new();
    let mut high = from_block;
    while high >= to_block {
        let low = high
            .saturating_sub(chunk_size.saturating_sub(1))
            .max(to_block);
        let chunk_filter = filter.clone().from_block(low).to_block(high);
        let chunk = provider
            .get_logs(&chunk_filter)
            .await
            .with_context(|| format!("get_logs failed for blocks {low}..={high}"))?;
        out.extend(chunk);
        if low == to_block {
            break;
        }
        high = low.saturating_sub(1);
    }
    Ok(out)
}
