use std::convert::TryFrom;

use anchor_lang::{
    prelude::ProgramError,
    solana_program::clock::{DEFAULT_TICKS_PER_SECOND, DEFAULT_TICKS_PER_SLOT, SECONDS_PER_DAY},
};
use spl_math::precise_number::PreciseNumber;

use crate::errors::ErrorCode;

pub const INITIAL_COLLATERAL_RATIO: u64 = 1;

pub fn calc_reserve_to_lp(
    reserve_token_amount: u64,
    lp_token_supply: u64,
    reserve_tokens_in_vault: u64,
) -> Option<u64> {
    match reserve_tokens_in_vault {
        // Assert that lp supply is 0
        0 => Some(INITIAL_COLLATERAL_RATIO.checked_mul(reserve_token_amount)?),
        _ => {
            let reserve_token_amount = PreciseNumber::new(reserve_token_amount as u128)?;
            let lp_token_supply = PreciseNumber::new(lp_token_supply as u128)?;
            let reserve_tokens_in_vault = PreciseNumber::new(reserve_tokens_in_vault as u128)?;

            let lp_tokens_to_mint = lp_token_supply
                .checked_mul(&reserve_token_amount)?
                .checked_div(&reserve_tokens_in_vault)?
                .floor()?
                .to_imprecise()?;

            u64::try_from(lp_tokens_to_mint).ok()
        }
    }
}

pub fn calc_lp_to_reserve(
    lp_token_amount: u64,
    lp_token_supply: u64,
    reserve_tokens_in_vault: u64,
) -> Option<u64> {
    let lp_token_amount = PreciseNumber::new(lp_token_amount as u128)?;
    let lp_token_supply = PreciseNumber::new(lp_token_supply as u128)?;
    let reserve_tokens_in_vault = PreciseNumber::new(reserve_tokens_in_vault as u128)?;

    let reserve_tokens_to_transfer = lp_token_amount
        .checked_mul(&reserve_tokens_in_vault)?
        .checked_div(&lp_token_supply)?
        .floor()?
        .to_imprecise()?;

    u64::try_from(reserve_tokens_to_transfer).ok()
}

/// Number of slots per year
/// 63072000
pub const SLOTS_PER_YEAR: u64 =
    DEFAULT_TICKS_PER_SECOND / DEFAULT_TICKS_PER_SLOT * SECONDS_PER_DAY * 365;

pub const ONE_AS_BPS: u64 = 10000;

pub fn calc_carry_fees(profit: u64, fee_bps: u64) -> Result<u64, ProgramError> {
    profit
        .checked_mul(fee_bps)
        .map(|n| n / ONE_AS_BPS)
        .ok_or_else(|| ErrorCode::OverflowError.into())
}