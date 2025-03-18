use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeLiquidityManager<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub n_dollar_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"liquidity_manager".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + LiquidityManager::SPACE
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}