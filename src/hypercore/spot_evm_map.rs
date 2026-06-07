//! Map HyperEVM contract addresses to spot token names and perp symbols.

use std::collections::HashMap;
use std::str::FromStr;

use alloy::primitives::Address;

use crate::hypercore::SpotToken;

/// Lookup table built from HyperCore `spotMeta` tokens.
#[derive(Clone, Debug, Default)]
pub struct SpotEvmMap {
    by_evm_contract: HashMap<Address, SpotToken>,
}

impl SpotEvmMap {
    /// Builds a map keyed by normalized `evm_contract` address.
    #[must_use]
    pub fn from_spot_tokens(tokens: &[SpotToken]) -> Self {
        let mut by_evm_contract = HashMap::new();
        for token in tokens {
            if let Some(addr) = token.evm_contract {
                by_evm_contract.insert(addr, token.clone());
            }
        }
        Self { by_evm_contract }
    }

    /// Resolves a spot token by EVM contract address (case-insensitive hex).
    #[must_use]
    pub fn spot_token_by_evm_contract(&self, contract: &str) -> Option<&SpotToken> {
        Address::from_str(contract.trim())
            .ok()
            .and_then(|addr| self.by_evm_contract.get(&addr))
    }

    /// Heuristic perp symbol from a spot token name (e.g. `UBTC` → `BTC`, `HYPE` → `HYPE`).
    #[must_use]
    pub fn perp_for_spot_name(spot_name: &str) -> String {
        let upper = spot_name.trim().to_uppercase();
        if upper.len() > 1 && upper.starts_with('U') && upper != "USDC" && upper != "USDT" {
            upper[1..].to_string()
        } else {
            upper
        }
    }

    /// Maps an EVM contract to a perp coin symbol when spot linkage exists.
    #[must_use]
    pub fn perp_for_evm_contract(&self, contract: &str) -> Option<String> {
        self.spot_token_by_evm_contract(contract)
            .map(|t| Self::perp_for_spot_name(&t.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypercore::SpotToken;
    use alloy::primitives::Address;

    fn token(name: &str, evm: Option<Address>) -> SpotToken {
        SpotToken {
            name: name.into(),
            index: 0,
            token_id: Default::default(),
            evm_contract: evm,
            cross_chain_address: None,
            sz_decimals: 8,
            wei_decimals: 8,
            evm_extra_decimals: 0,
        }
    }

    #[test]
    fn maps_evm_contract_to_perp() {
        let addr: Address = "0x9fdbda0a5e284c32744d2f17ee5c74b284993463"
            .parse()
            .unwrap();
        let map = SpotEvmMap::from_spot_tokens(&[token("UBTC", Some(addr))]);
        let perp = map
            .perp_for_evm_contract("0x9fdbda0a5e284c32744d2f17ee5c74b284993463")
            .unwrap();
        assert_eq!(perp, "BTC");
    }

    #[test]
    fn perp_for_spot_name_strips_u_prefix() {
        assert_eq!(SpotEvmMap::perp_for_spot_name("UBTC"), "BTC");
        assert_eq!(SpotEvmMap::perp_for_spot_name("USDC"), "USDC");
        assert_eq!(SpotEvmMap::perp_for_spot_name("HYPE"), "HYPE");
    }
}
