//! PRJX / Uniswap V3 swap execution helpers.

use alloy::primitives::{B256, U160, U256, aliases::U24};
use anyhow::{Context, Result};
use rust_decimal::Decimal;

use crate::hyperevm::{
    Address, Provider, from_wei, to_wei,
    uniswap::contracts::{
        IQuoterV2::{self, IQuoterV2Instance},
        ISwapRouter::{self, ISwapRouterInstance},
    },
};

/// Parameters for a single-hop exact-input swap on PRJX.
#[derive(Clone, Debug)]
pub struct ExactInputSingleParams {
    pub token_in: Address,
    pub token_out: Address,
    pub fee_bps: u32,
    pub recipient: Address,
    pub amount_in: Decimal,
    pub amount_in_decimals: u32,
    pub amount_out_minimum: Decimal,
    pub amount_out_decimals: u32,
    pub deadline_unix: u64,
}

/// Swap client wrapping an Uniswap V3 router instance.
pub struct SwapClient<P: Provider> {
    router: ISwapRouterInstance<P>,
}

impl<P: Provider> SwapClient<P> {
    #[must_use]
    pub fn new(router: ISwapRouterInstance<P>) -> Self {
        Self { router }
    }

    /// Quotes minimum output via router quoter is not wired here; caller supplies slippage floor.
    pub async fn exact_input_single(&self, params: ExactInputSingleParams) -> Result<U256> {
        let amount_in = to_wei(params.amount_in, params.amount_in_decimals);
        let amount_out_min = to_wei(params.amount_out_minimum, params.amount_out_decimals);
        let fee = U24::from(params.fee_bps);
        let sqrt_price_limit = U160::ZERO;

        let call = self
            .router
            .exactInputSingle(ISwapRouter::ExactInputSingleParams {
                tokenIn: params.token_in,
                tokenOut: params.token_out,
                fee,
                recipient: params.recipient,
                deadline: U256::from(params.deadline_unix),
                amountIn: amount_in,
                amountOutMinimum: amount_out_min,
                sqrtPriceLimitX96: sqrt_price_limit,
            });

        call.call()
            .await
            .context("exactInputSingle simulation failed")
    }

    /// Submits a single-hop exact-input swap transaction.
    pub async fn send_exact_input_single(&self, params: ExactInputSingleParams) -> Result<B256> {
        let amount_in = to_wei(params.amount_in, params.amount_in_decimals);
        let amount_out_min = to_wei(params.amount_out_minimum, params.amount_out_decimals);
        let fee = U24::from(params.fee_bps);
        let sqrt_price_limit = U160::ZERO;

        let pending = self
            .router
            .exactInputSingle(ISwapRouter::ExactInputSingleParams {
                tokenIn: params.token_in,
                tokenOut: params.token_out,
                fee,
                recipient: params.recipient,
                deadline: U256::from(params.deadline_unix),
                amountIn: amount_in,
                amountOutMinimum: amount_out_min,
                sqrtPriceLimitX96: sqrt_price_limit,
            })
            .send()
            .await
            .context("exactInputSingle send failed")?;

        let tx_hash = *pending.tx_hash();
        let receipt = pending.get_receipt().await.context("swap receipt failed")?;
        anyhow::ensure!(receipt.status(), "swap transaction reverted");
        Ok(tx_hash)
    }
}

/// Quotes a single-hop exact-input swap via QuoterV2.
pub async fn quote_exact_input_single<P: Provider>(
    quoter: &IQuoterV2Instance<P>,
    token_in: Address,
    token_out: Address,
    fee_bps: u32,
    amount_in: Decimal,
    amount_in_decimals: u32,
    amount_out_decimals: u32,
) -> Result<Decimal> {
    let amount_in_wei = to_wei(amount_in, amount_in_decimals);
    let result = quoter
        .quoteExactInputSingle(IQuoterV2::QuoteExactInputSingleParams {
            tokenIn: token_in,
            tokenOut: token_out,
            amountIn: amount_in_wei,
            fee: U24::from(fee_bps),
            sqrtPriceLimitX96: U160::ZERO,
        })
        .call()
        .await
        .context("quoteExactInputSingle failed")?;
    Ok(from_wei(result.amountOut, amount_out_decimals))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_input_params_are_constructible() {
        let token_in: Address = "0x5555555555555555555555555555555555555555"
            .parse()
            .unwrap();
        let token_out: Address = "0x9fdbda0a5e284c32744d2f17ee5c74b284993463"
            .parse()
            .unwrap();
        let recipient: Address = "0x0000000000000000000000000000000000000001"
            .parse()
            .unwrap();
        let _ = ExactInputSingleParams {
            token_in,
            token_out,
            fee_bps: 3000,
            recipient,
            amount_in: Decimal::ONE,
            amount_in_decimals: 18,
            amount_out_minimum: Decimal::ZERO,
            amount_out_decimals: 8,
            deadline_unix: 4_000_000_000,
        };
    }
}
