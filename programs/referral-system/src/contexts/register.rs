use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct RegisterReferral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [REFERRAL_DATA_SEED, referral_data.coin_mint.as_ref()],
        bump = referral_data.bump
    )]
    pub referral_data: Account<'info, ReferralData>,
    
    #[account(
        init,
        payer = user,
        seeds = [USER_DATA_SEED, user.key().as_ref(), referral_data.coin_mint.as_ref()],
        bump,
        space = 8 + UserData::SPACE
    )]
    pub user_data: Account<'info, UserData>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
