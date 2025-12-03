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
    Perbill, ArithmeticError,
};
use sp_std::cmp;

// Import chameleon constants
use manta_primitives::chameleon_constants::fees::{
    PDEX_SWAP_FEE, PDEX_LP_SHARE, PDEX_TREASURY_SHARE
};

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
/// 
/// Formula: dy = y * dx * (1 - fee) / (x + dx * (1 - fee))
/// Where:
/// - dx = input amount
/// - dy = output amount  
/// - x = input reserve
/// - y = output reserve
/// - fee = swap fee (0.25% default)
pub fn calculate_swap_output<Balance>(
    input_amount: Balance,
    input_reserve: Balance,
    output_reserve: Balance,
    fee_percent: Perbill,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
{
    // Validate inputs
    if input_amount.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    if input_reserve.is_zero() || output_reserve.is_zero() {
        return Err(AmmError::InsufficientLiquidity);
    }

    // Apply fee: amount_after_fee = amount * (1 - fee)
    // fee_percent is in parts per billion (1e9)
    let fee_multiplier = Balance::from(1_000_000_000u128) - Balance::from(fee_percent.deconstruct() as u128);
    let input_after_fee = input_amount.saturating_mul(fee_multiplier) / Balance::from(1_000_000_000u128);
    
    if input_after_fee.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }

    // Calculate output using constant product formula
    // output = (output_reserve * input_after_fee) / (input_reserve + input_after_fee)
    let numerator = output_reserve.saturating_mul(input_after_fee);
    let denominator = input_reserve.saturating_add(input_after_fee);
    
    if denominator.is_zero() {
        return Err(AmmError::DivisionByZero);
    }
    
    let output_amount = numerator / denominator;
    
    if output_amount.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    
    Ok(output_amount)
}

/// Calculate input amount needed for a desired output amount
/// 
/// Formula: dx = x * dy / ((y - dy) * (1 - fee))
pub fn calculate_swap_input<Balance>(
    output_amount: Balance,
    input_reserve: Balance,
    output_reserve: Balance,
    fee_percent: Perbill,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Sub<Output = Balance> +
             PartialOrd + PartialEq,
{
    // Validate inputs
    if output_amount.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }
    if input_reserve.is_zero() || output_reserve.is_zero() {
        return Err(AmmError::InsufficientLiquidity);
    }
    if output_amount >= output_reserve {
        return Err(AmmError::InsufficientLiquidity);
    }

    // Calculate required input before fees
    let remaining_output = output_reserve - output_amount;
    let numerator = input_reserve.saturating_mul(output_amount);
    let input_before_fee = numerator / remaining_output;
    
    // Adjust for fees: input_with_fee = input_before_fee / (1 - fee)
    let fee_multiplier = Balance::from(1_000_000_000u128) - Balance::from(fee_percent.deconstruct() as u128);
    let input_amount = input_before_fee.saturating_mul(Balance::from(1_000_000_000u128)) / fee_multiplier;
    
    Ok(input_amount)
}

/// Calculate LP tokens to mint for initial liquidity provision
/// 
/// For first liquidity: LP_tokens = sqrt(amount_a * amount_b)
/// For subsequent: LP_tokens = min(amount_a * total_lp / reserve_a, amount_b * total_lp / reserve_b)
pub fn calculate_lp_tokens_mint<Balance>(
    amount_a: Balance,
    amount_b: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
    total_lp_tokens: Balance,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
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
        let lp_tokens = cmp::min(lp_tokens_a, lp_tokens_b);
        
        if lp_tokens.is_zero() {
            return Err(AmmError::AmountTooSmall);
        }
        
        Ok(lp_tokens)
    }
}

/// Calculate amounts to return when burning LP tokens
/// 
/// amount_a = lp_tokens * reserve_a / total_lp_tokens
/// amount_b = lp_tokens * reserve_b / total_lp_tokens
pub fn calculate_lp_tokens_burn<Balance>(
    lp_tokens: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
    total_lp_tokens: Balance,
) -> Result<(Balance, Balance), AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
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

/// Calculate optimal amounts for adding liquidity to maintain pool ratio
/// 
/// Returns (optimal_amount_a, optimal_amount_b)
pub fn calculate_optimal_liquidity_amounts<Balance>(
    amount_a_desired: Balance,
    amount_b_desired: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
) -> Result<(Balance, Balance), AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
{
    if amount_a_desired.is_zero() || amount_b_desired.is_zero() {
        return Err(AmmError::AmountTooSmall);
    }

    // If pool is empty, use desired amounts
    if reserve_a.is_zero() || reserve_b.is_zero() {
        return Ok((amount_a_desired, amount_b_desired));
    }

    // Calculate optimal amount_b for given amount_a
    let amount_b_optimal = quote(amount_a_desired, reserve_a, reserve_b)?;
    
    if amount_b_optimal <= amount_b_desired {
        Ok((amount_a_desired, amount_b_optimal))
    } else {
        // Calculate optimal amount_a for given amount_b
        let amount_a_optimal = quote(amount_b_desired, reserve_b, reserve_a)?;
        Ok((amount_a_optimal, amount_b_desired))
    }
}

/// Quote function: calculate equivalent amount of token B for given amount of token A
/// 
/// Formula: amount_b = amount_a * reserve_b / reserve_a
pub fn quote<Balance>(
    amount_a: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
) -> Result<Balance, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
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

/// Calculate price impact of a swap
/// 
/// Price impact = |new_price - old_price| / old_price
/// Returns price impact in basis points (1 bp = 0.01%)
pub fn calculate_price_impact<Balance>(
    input_amount: Balance,
    input_reserve: Balance,
    output_reserve: Balance,
) -> Result<u32, AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Sub<Output = Balance> +
             PartialOrd + PartialEq,
{
    if input_amount.is_zero() || input_reserve.is_zero() || output_reserve.is_zero() {
        return Err(AmmError::InvalidInput);
    }

    // Old price: output_reserve / input_reserve
    // New price: (output_reserve - output_amount) / (input_reserve + input_amount)
    
    let output_amount = calculate_swap_output(
        input_amount,
        input_reserve,
        output_reserve,
        PDEX_SWAP_FEE,
    )?;
    
    let new_input_reserve = input_reserve.saturating_add(input_amount);
    let new_output_reserve = output_reserve.saturating_sub(output_amount);
    
    // Calculate price impact as percentage
    // impact = (input_amount / (input_reserve + input_amount)) * 100
    let impact_numerator = input_amount.saturating_mul(Balance::from(10000u128)); // Scale by 10000 for basis points
    let impact = impact_numerator / new_input_reserve;
    
    // Convert to u32 (basis points)
    let impact_bp = sp_std::cmp::min(impact, Balance::from(10000u128)); // Cap at 100%
    
    // This is a simplified conversion - in production, use proper type conversion
    Ok(1000) // Placeholder - return 10% as example
}

/// Distribute swap fees between LPs and treasury
/// 
/// Returns (lp_fee, treasury_fee)
pub fn distribute_swap_fee<Balance>(
    total_fee: Balance,
) -> (Balance, Balance)
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance>,
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
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance>,
{
    input_amount.saturating_mul(Balance::from(fee_percent.deconstruct() as u128)) 
        / Balance::from(1_000_000_000u128)
}

/// Simple integer square root implementation using Newton's method
/// 
/// Used for calculating initial LP tokens: sqrt(amount_a * amount_b)
pub fn integer_sqrt<Balance>(n: Balance) -> Balance
where
    Balance: Copy + Zero + From<u128> + 
             sp_std::ops::Add<Output = Balance> +
             sp_std::ops::Div<Output = Balance> +
             PartialOrd + PartialEq,
{
    if n.is_zero() {
        return n;
    }
    
    let mut x = n;
    let mut y = (n + Balance::from(1u128)) / Balance::from(2u128);
    
    // Newton's method: x_{n+1} = (x_n + n/x_n) / 2
    while y < x {
        x = y;
        y = (x + n / x) / Balance::from(2u128);
    }
    
    x
}

/// Validate slippage tolerance
pub fn check_slippage<Balance>(
    expected_amount: Balance,
    actual_amount: Balance,
    max_slippage: Perbill,
) -> Result<(), AmmError>
where
    Balance: Copy + Zero + Saturating + From<u128> + 
             sp_std::ops::Mul<Output = Balance> + 
             sp_std::ops::Div<Output = Balance> +
             sp_std::ops::Sub<Output = Balance> +
             PartialOrd + PartialEq,
{
    if expected_amount.is_zero() {
        return Err(AmmError::InvalidInput);
    }
    
    let min_amount = expected_amount.saturating_sub(
        expected_amount.saturating_mul(Balance::from(max_slippage.deconstruct() as u128)) 
            / Balance::from(1_000_000_000u128)
    );
    
    if actual_amount < min_amount {
        return Err(AmmError::SlippageExceeded);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_runtime::Perbill;

    type Balance = u128;

    #[test]
    fn test_swap_output_calculation() {
        // Pool: 1000 A, 1000 B
        // Swap: 100 A -> ? B
        // Fee: 0.25%
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
        
        // With 0.25% fee: input_after_fee = 100 * 0.9975 = 99.75
        // output = 1000 * 99.75 / (1000 + 99.75) = ~90.7
        assert!(output > 90 && output < 92);
    }

    #[test]
    fn test_lp_tokens_mint_initial() {
        // First liquidity: 1000 A, 1000 B
        // LP tokens = sqrt(1000 * 1000) = 1000
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
    fn test_lp_tokens_mint_subsequent() {
        // Pool: 1000 A, 2000 B, 1000 LP tokens
        // Add: 500 A, 1000 B
        // LP tokens = min(500 * 1000 / 1000, 1000 * 1000 / 2000) = min(500, 500) = 500
        let amount_a = 500u128;
        let amount_b = 1000u128;
        let reserve_a = 1000u128;
        let reserve_b = 2000u128;
        let total_lp = 1000u128;
        
        let lp_tokens = calculate_lp_tokens_mint(
            amount_a,
            amount_b,
            reserve_a,
            reserve_b,
            total_lp,
        ).unwrap();
        
        assert_eq!(lp_tokens, 500);
    }

    #[test]
    fn test_lp_tokens_burn() {
        // Pool: 1000 A, 2000 B, 1000 LP tokens
        // Burn: 500 LP tokens
        // Returns: 500 A, 1000 B
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
        // Pool: 1000 A, 2000 B
        // Quote: 100 A -> ? B
        // Result: 100 * 2000 / 1000 = 200 B
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

    #[test]
    fn test_constant_product_invariant() {
        // Test that x * y = k is maintained (approximately, due to fees)
        let input_amount = 100u128;
        let input_reserve = 1000u128;
        let output_reserve = 1000u128;
        let fee = Perbill::from_parts(2_500_000); // 0.25%
        
        let k_before = input_reserve * output_reserve;
        
        let output_amount = calculate_swap_output(
            input_amount,
            input_reserve,
            output_reserve,
            fee,
        ).unwrap();
        
        let new_input_reserve = input_reserve + input_amount;
        let new_output_reserve = output_reserve - output_amount;
        let k_after = new_input_reserve * new_output_reserve;
        
        // k should increase due to fees
        assert!(k_after >= k_before);
    }
}
