use alloy::primitives::U256;
use rust_decimal::Decimal;

/// Converts a decimal amount to wei representation.
///
/// Wei is the smallest unit of Ethereum tokens (like satoshis for Bitcoin).
///
/// # Parameters
///
/// - `size`: The decimal amount to convert
/// - `decimals`: Number of decimal places for the token (e.g., 18 for ETH, 6 for USDC)
///
/// # Example
///
/// Convert 1.5 ETH to wei (18 decimals): `to_wei(dec!(1.5), 18)`
#[must_use]
#[inline]
pub fn to_wei(mut size: Decimal, decimals: u32) -> U256 {
    size.rescale(decimals);
    U256::from(size.mantissa())
}

/// Converts wei representation to a decimal amount.
///
/// # Parameters
///
/// - `wei`: The wei amount to convert
/// - `decimals`: Number of decimal places for the token (e.g., 18 for ETH, 6 for USDC)
///
/// # Example
///
/// Convert wei back to decimal: `from_wei(wei, 18)`
#[must_use]
#[inline]
pub fn from_wei(wei: U256, decimals: u32) -> Decimal {
    Decimal::from_i128_with_scale(wei.to::<i128>(), decimals)
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;

    #[test]
    fn test_from_wei() {
        let test_values = [
            (
                U256::from(72305406316320073300i128),
                18,
                dec!(72.305406316320073300),
            ),
            (U256::from(98996405), 6, dec!(98.996405)),
        ];
        for (index, (got, decimals, expect)) in test_values.into_iter().enumerate() {
            assert_eq!(from_wei(got, decimals), expect, "failed at {index}");
        }
    }

    #[test]
    fn test_to_wei() {
        let test_values = [
            (
                dec!(72.305406316320073386),
                18,
                U256::from(72305406316320073386i128),
            ),
            (dec!(98.996405), 6, U256::from(98996405)),
            (dec!(69), 6, U256::from(69000000)),
        ];
        for (index, (got, decimals, expect)) in test_values.into_iter().enumerate() {
            assert_eq!(to_wei(got, decimals), expect, "failed at {index}");
        }
    }
}
