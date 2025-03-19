use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct CheckReferralStatus<'info> {
    #[account(
        seeds = [REFERRAL_DATA_SEED, referral_data.coin_mint.as_ref()],
        bump = referral_data.bump
    )]
    pub referral_data: Account<'info, ReferralData>,
}
