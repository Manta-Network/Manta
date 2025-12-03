// Copyright 2020-2024 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

//! AMM (Automated Market Maker) mathematics for Chameleon pDEX
//!
//! Implements the constant product formula: x × y = k
//! Based on Uniswap v2 with Chameleon-specific optimizations

use sp_runtime::{
    traits::{Zero, Saturating},
    Perbill,
};

// Import chameleon constants
use manta_primitives::chameleon_constants::fees::PDEX_LP_SHARE;

/// AMM calculation errors
#[derive(Debug, PartialEq, Eq)]
pub enum AmmError {
    /// Insufficient liquidity in the pool
    InsufficientLiquidity,
    /// Amount is too small to process
    AmountTooSmall,
    /// Mathematical overflow occurred
    Overflow,
    /// Division by zero
    DivisionByZero,
    /// Invalid input parameters
    InvalidInput,
    /// Slippage tolerance exceeded
    SlippageExceeded,
}

/// Calculate swap output using constant product formula with fees
/// Uses basis points for fee (25 = 0.25%)
pub fn calculate_swap_output<Balance>(
    input_amount: Balance,
    input_reserve: Balance,
    output_reserve: Balance,
    fee_bps: u32,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance>,
{
    // Validate inputs
    if input_amount.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    if input_reserve.is_zero() || output_reserve.is_zero() {
        return Err(AmmError::InsufficientLiquidity);
    }

    // Apply fee: input_with_fee = input * (10000 - fee_bps) / 10000
    let input_with_fee = input_amount
        .saturating_mul(Balance::from(10000u128.saturating_sub(fee_bps as u128)))
        / Balance::from(10000u128);
    
    if input_with_fee.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }

    // Calculate output: dy = y * dx / (x + dx)
    let numerator = output_reserve.saturating_mul(input_with_fee);
    let denominator = input_reserve.saturating_add(input_with_fee);
    
    if denominator.is_zero() {
        return Err(AmmError::DivisionByZero);
    }
    
    let output_amount = numerator / denominator;
    
    if output_amount.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    
    Ok(output_amount)
}

/// Calculate LP tokens to mint for initial liquidity provision
pub fn calculate_lp_tokens_mint<Balance>(
    amount_a: Balance,
    amount_b: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
    total_lp_tokens: Balance,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + PartialOrd +
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance>,
{
    if amount_a.is_zero() || amount_b.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }

    if total_lp_tokens.is_zero() {
        // First liquidity provision: LP tokens = sqrt(amount_a * amount_b)
        let product = amount_a.saturating_mul(amount_b);
        let lp_tokens = integer_sqrt(product);
        
        if lp_tokens.is_zero() {
            return Err(AmmError::AmountTooSmall);
        }
        
        Ok(lp_tokens)
    } else {
        // Subsequent liquidity provision: maintain ratio
        if reserve_a.is_zero() || reserve_b.is_zero() {
            return Err(AmmError::InsufficientLiquidity);
        }
        
        let lp_tokens_a = amount_a.saturating_mul(total_lp_tokens) / reserve_a;
        let lp_tokens_b = amount_b.saturating_mul(total_lp_tokens) / reserve_b;
        
        // Take minimum to maintain pool ratio
        let lp_tokens = if lp_tokens_a < lp_tokens_b { lp_tokens_a } else { lp_tokens_b };
        
        if lp_tokens.is_zero() {
            return Err(AmmError::AmountTooSmall);
        }
        
        Ok(lp_tokens)
    }
}

/// Calculate amounts to return when burning LP tokens
pub fn calculate_lp_tokens_burn<Balance>(
    lp_tokens: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
    total_lp_tokens: Balance,
) -> Result<(Balance, Balance), AmmError>
where
    Balance: Copy + Zero + Saturating + PartialOrd +
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance>,
{
    if lp_tokens.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    if total_lp_tokens.is_zero() {
        return Err(AmmError::DivisionByZero);
    }
    if lp_tokens > total_lp_tokens {
        return Err(AmmError::InvalidInput);
    }

    let amount_a = lp_tokens.saturating_mul(reserve_a) / total_lp_tokens;
    let amount_b = lp_tokens.saturating_mul(reserve_b) / total_lp_tokens;
    
    Ok((amount_a, amount_b))
}

/// Quote function: calculate equivalent amount of token B for given amount of token A
pub fn quote<Balance>(
    amount_a: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating +
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance>,
{
    if amount_a.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    if reserve_a.is_zero() || reserve_b.is_zero() {
        return Err(AmmError::InsufficientLiquidity);
    }

    let amount_b = amount_a.saturating_mul(reserve_b) / reserve_a;
    Ok(amount_b)
}

/// Distribute swap fees between LPs and treasury
pub fn distribute_swap_fee<Balance>(
    total_fee: Balance,
) -> (Balance, Balance)
where
    Balance: Copy + Zero + Saturating + From<u128> +
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance>,
{
    let lp_fee = total_fee.saturating_mul(Balance::from(PDEX_LP_SHARE.deconstruct() as u128))
        / Balance::from(1_000_000_000u128);
    let treasury_fee = total_fee.saturating_sub(lp_fee);
    
    (lp_fee, treasury_fee)
}

/// Calculate the fee amount for a given swap
pub fn calculate_swap_fee<Balance>(
    input_amount: Balance,
    fee_percent: Perbill,
) -> Balance
where
    Balance: Copy + Zero + Saturating + From<u128> +
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance>,
{
    input_amount.saturating_mul(Balance::from(fee_percent.deconstruct() as u128))
        / Balance::from(1_000_000_000u128)
}

/// Simple integer square root implementation using Newton's method
pub fn integer_sqrt<Balance>(n: Balance) -> Balance
where
    Balance: Copy + Zero + From<u128> + PartialOrd + Saturating +
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Mul<Output = Balance> +
             sp_std::ops::Add<Output = Balance>,
{
    if n.is_zero() {
        return n;
    }
    
    let mut x = n;
    let mut y = (n + Balance::from(1u128)) / Balance::from(2u128);
    
    // Newton's method: x_{n+1} = (x_n + n/x_n) / 2
    let mut iterations = 0;
    while y < x && iterations < 100 { // Prevent infinite loops
        x = y;
        y = (x + n / x) / Balance::from(2u128);
        iterations += 1;
    }
    
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_runtime::Perbill;

    type Balance = u128;

    #[test]
    fn test_swap_output_calculation() {
        let input_amount = 100u128;
        let input_reserve = 1000u128;
        let output_reserve = 1000u128;
        let fee = Perbill::from_parts(2_500_000); // 0.25%
        
        let output = calculate_swap_output(
            input_amount,
            input_reserve,
            output_reserve,
            fee,
        ).unwrap();
        
        // Should get approximately 90-92 tokens out
        assert!(output > 90 && output < 92);
    }

    #[test]
    fn test_lp_tokens_mint_initial() {
        let amount_a = 1000u128;
        let amount_b = 1000u128;
        let reserve_a = 0u128;
        let reserve_b = 0u128;
        let total_lp = 0u128;
        
        let lp_tokens = calculate_lp_tokens_mint(
            amount_a,
            amount_b,
            reserve_a,
            reserve_b,
            total_lp,
        ).unwrap();
        
        assert_eq!(lp_tokens, 1000);
    }

    #[test]
    fn test_lp_tokens_burn() {
        let lp_tokens = 500u128;
        let reserve_a = 1000u128;
        let reserve_b = 2000u128;
        let total_lp = 1000u128;
        
        let (amount_a, amount_b) = calculate_lp_tokens_burn(
            lp_tokens,
            reserve_a,
            reserve_b,
            total_lp,
        ).unwrap();
        
        assert_eq!(amount_a, 500);
        assert_eq!(amount_b, 1000);
    }

    #[test]
    fn test_quote() {
        let amount_a = 100u128;
        let reserve_a = 1000u128;
        let reserve_b = 2000u128;
        
        let amount_b = quote(amount_a, reserve_a, reserve_b).unwrap();
        assert_eq!(amount_b, 200);
    }

    #[test]
    fn test_fee_distribution() {
        let total_fee = 1000u128;
        let (lp_fee, treasury_fee) = distribute_swap_fee(total_fee);
        
        // 90% to LPs, 10% to treasury
        assert_eq!(lp_fee, 900);
        assert_eq!(treasury_fee, 100);
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0u128), 0);
        assert_eq!(integer_sqrt(1u128), 1);
        assert_eq!(integer_sqrt(4u128), 2);
        assert_eq!(integer_sqrt(9u128), 3);
        assert_eq!(integer_sqrt(16u128), 4);
        assert_eq!(integer_sqrt(1000000u128), 1000);
    }
}
