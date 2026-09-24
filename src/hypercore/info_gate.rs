//! Shared `/info` slot: Hyperliquid REST is **1200 weighted req/min per IP**.
//! Most `info` types weigh **20** (`openOrders`, `userFillsByTime`, `perpDexs`, …);
//! only `clearinghouseState` / `l2Book` / `allMids` / a few others weigh **2**.
//! Bursting 10 HIP-3 `openOrders` calls (weight 200) plus the chart's
//! `candleSnapshot` from the same machine is what actually 429s — not a tight cap.

use std::sync::OnceLock;
use std::time::Duration;

use anyhow::anyhow;
use tokio::sync::Semaphore;

const MAX_INFLIGHT: usize = 2;
const MAX_429: u32 = 6;

fn info_sem() -> &'static Semaphore {
    static SEM: OnceLock<Semaphore> = OnceLock::new();
    SEM.get_or_init(|| Semaphore::new(MAX_INFLIGHT))
}

pub(crate) fn is_http_429(err: &anyhow::Error) -> bool {
    let msg = err.to_string();
    msg.contains("429") || msg.contains("Too Many Requests")
}

fn retry_after_ms(attempt: u32) -> u64 {
    (500u64 * (1u64 << attempt.min(5))).min(16_000)
}

/// Serialize `/info` POSTs: at most two in flight, 429s retried while holding the slot.
pub(crate) async fn with_info_slot<T, F, Fut>(label: &str, mut op: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut last = None;
    for attempt in 0..MAX_429 {
        let permit = info_sem()
            .acquire()
            .await
            .map_err(|_| anyhow!("{label}: info slot closed"))?;
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if is_http_429(&e) && attempt + 1 < MAX_429 => {
                drop(permit);
                let delay = Duration::from_millis(retry_after_ms(attempt));
                log::debug!("{label}: HTTP 429, retry in {}ms", delay.as_millis());
                tokio::time::sleep(delay).await;
                last = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or_else(|| anyhow!("{label}: rate limited")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hl_429_body_null() {
        let e = anyhow!("[open_orders] HTTP 429 Too Many Requests body=null");
        assert!(is_http_429(&e));
    }

    #[tokio::test]
    async fn retries_then_succeeds() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let n = AtomicU32::new(0);
        let v = with_info_slot("t", || {
            let i = n.fetch_add(1, Ordering::SeqCst);
            async move {
                if i == 0 {
                    anyhow::bail!("HTTP 429 Too Many Requests body=null");
                }
                Ok(7u32)
            }
        })
        .await
        .unwrap();
        assert_eq!(v, 7);
        assert_eq!(n.load(Ordering::SeqCst), 2);
    }
}
