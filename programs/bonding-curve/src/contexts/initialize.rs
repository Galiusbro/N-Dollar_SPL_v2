use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeBondingCurve<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        seeds = [b"bonding_curve".as_ref(), coin_mint.key().as_ref()],
        bump,
        space = 8 + BondingCurve::SPACE
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(mut)]
    pub coin_mint: Account<'info, Mint>,
    
    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        token::mint = ndollar_mint
    )]
    pub liquidity_pool: Account<'info, TokenAccount>,
    
    /// Admin control аккаунт
    /// Этот аккаунт хранит информацию об авторизованных программах и настройках
    #[account(
        seeds = [b"admin_config".as_ref(), creator.key().as_ref()],
        bump,
        seeds::program = admin_control_program.key()
    )]
    pub admin_config: Account<'info, admin_control::AdminConfig>,
    
    /// Программа admin_control
    pub admin_control_program: Program<'info, admin_control::program::AdminControl>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}