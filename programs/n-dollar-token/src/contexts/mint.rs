use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct MintSupply<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = mint.key() == admin_account.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
    #[account(
        mut,
        constraint = token_account.mint == mint.key()
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// Опциональный admin_config аккаунт из программы admin_control для проверки авторизации
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова
    pub admin_config: Option<AccountInfo<'info>>,
    
    /// Опциональная программа admin_control для CPI вызовов
    /// CHECK: ID программы admin_control
    pub admin_control_program: Option<AccountInfo<'info>>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
