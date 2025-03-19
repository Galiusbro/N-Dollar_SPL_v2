use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct RewardReferral<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [REFERRAL_DATA_SEED, referral_data.coin_mint.as_ref()],
        bump = referral_data.bump
    )]
    pub referral_data: Account<'info, ReferralData>,
    
    #[account(
        mut,
        seeds = [USER_DATA_SEED, user_data.user.as_ref(), user_data.coin_mint.as_ref()],
        bump = user_data.bump
    )]
    pub user_data: Account<'info, UserData>,
    
    #[account(
        mut,
        constraint = reward_source.owner == authority.key()
    )]
    pub reward_source: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub referrer_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
