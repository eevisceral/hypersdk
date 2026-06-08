//! Asset name parsing and perpetual market resolution.
//!
//! Supports unified asset formats used across Hyperliquid:
//! - `"BTC"` — native perp
//! - `"PURR/USDC"` — spot pair
//! - `"xyz:BTC"` — HIP-3 perp on a named DEX

use super::PerpMarket;

/// Parsed asset specification from a unified name string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSpec<'a> {
    /// Perpetual on the native Hyperliquid DEX (e.g. `"BTC"`).
    Perp(&'a str),
    /// Spot market (e.g. `"PURR/USDC"`).
    Spot(&'a str, &'a str),
    /// Perpetual on a HIP-3 DEX (e.g. `"xyz:BTC"`).
    Hip3Perp(&'a str, &'a str),
}

/// Parse an asset name string into an [`AssetSpec`].
///
/// # Formats
///
/// - `"BTC"` → `Perp("BTC")`
/// - `"PURR/USDC"` → `Spot("PURR", "USDC")`
/// - `"xyz:BTC"` → `Hip3Perp("xyz", "BTC")`
pub fn parse_asset_spec(asset: &str) -> anyhow::Result<AssetSpec<'_>> {
    if let Some((base, quote)) = asset.split_once('/') {
        Ok(AssetSpec::Spot(base, quote))
    } else if let Some((dex, symbol)) = asset.split_once(':') {
        Ok(AssetSpec::Hip3Perp(dex, symbol))
    } else {
        Ok(AssetSpec::Perp(asset))
    }
}

/// Returns whether a perp market `name` matches `symbol` (case-insensitive).
///
/// Matches either an exact name (`"BTC"`) or the symbol portion of HIP-3 names (`"xyz:BTC"`).
#[must_use]
pub fn perp_name_matches(name: &str, symbol: &str) -> bool {
    if name.eq_ignore_ascii_case(symbol) {
        return true;
    }
    if let Some((_dex, market_symbol)) = name.split_once(':') {
        return market_symbol.eq_ignore_ascii_case(symbol);
    }
    false
}

/// Resolve a perpetual symbol to its Hyperliquid asset index within `perps`.
///
/// Returns the [`PerpMarket::index`] for the first matching market, or `None` when no match exists.
#[must_use]
pub fn resolve_perp_market_index(perps: &[PerpMarket], symbol: &str) -> Option<usize> {
    perps
        .iter()
        .find(|p| perp_name_matches(&p.name, symbol))
        .map(|p| p.index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypercore::{PriceTick, SpotToken};

    fn sample_perp(name: &str, index: usize) -> PerpMarket {
        let collateral = SpotToken {
            name: "USDC".into(),
            index: 0,
            token_id: Default::default(),
            evm_contract: None,
            cross_chain_address: None,
            sz_decimals: 6,
            wei_decimals: 6,
            evm_extra_decimals: 0,
        };
        PerpMarket {
            name: name.into(),
            index,
            sz_decimals: 5,
            collateral,
            max_leverage: 40,
            isolated_margin: false,
            margin_mode: None,
            growth_mode: false,
            aligned_quote_token: false,
            margin_table_id: 0,
            margin_tiers: Vec::new(),
            table: PriceTick::for_perp(5),
            dex: None,
        }
    }

    #[test]
    fn parse_asset_spec_formats() {
        assert_eq!(parse_asset_spec("BTC").unwrap(), AssetSpec::Perp("BTC"));
        assert_eq!(
            parse_asset_spec("PURR/USDC").unwrap(),
            AssetSpec::Spot("PURR", "USDC")
        );
        assert_eq!(
            parse_asset_spec("xyz:BTC").unwrap(),
            AssetSpec::Hip3Perp("xyz", "BTC")
        );
    }

    #[test]
    fn perp_name_matches_native_and_hip3() {
        assert!(perp_name_matches("BTC", "btc"));
        assert!(perp_name_matches("xyz:AAPL", "AAPL"));
        assert!(!perp_name_matches("xyz:AAPL", "BTC"));
    }

    #[test]
    fn resolve_perp_market_index_returns_asset_index() {
        let perps = vec![sample_perp("BTC", 0), sample_perp("xyz:AAPL", 110_042)];
        assert_eq!(resolve_perp_market_index(&perps, "BTC"), Some(0));
        assert_eq!(resolve_perp_market_index(&perps, "AAPL"), Some(110_042));
        assert_eq!(resolve_perp_market_index(&perps, "ETH"), None);
    }
}
