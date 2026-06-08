//! Multi-DEX perp universe helpers.

use super::http::Client;
use super::types::MetaAndAssetCtxsResponse;

/// Returns every perp DEX name from `perpDexs`, preserving `null` for the native DEX slot.
pub async fn all_perp_dex_names(client: &Client) -> anyhow::Result<Vec<Option<String>>> {
    let (url, http) = client.request_parts();
    super::perp_dex_name_list(url, http).await
}

/// Fetches `metaAndAssetCtxs` for each DEX in `dexes` (parallel when multiple).
pub async fn meta_and_asset_ctxs_for_dexes(
    client: &Client,
    dexes: &[Option<String>],
) -> anyhow::Result<Vec<(Option<String>, MetaAndAssetCtxsResponse)>> {
    let mut out = Vec::with_capacity(dexes.len());
    for dex in dexes {
        let response = client.meta_and_asset_ctxs(dex.clone()).await?;
        out.push((dex.clone(), response));
    }
    Ok(out)
}
