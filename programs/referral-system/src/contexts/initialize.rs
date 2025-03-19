use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeReferralSystem<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [REFERRAL_SYSTEM_SEED, authority.key().as_ref()],
        bump,
        space = 8 + ReferralSystem::SPACE
    )]
    pub referral_system: Account<'info, ReferralSystem>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
