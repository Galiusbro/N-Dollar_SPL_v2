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

/// Контекст для минтинга N-Dollar токенов с автоматическим направлением в пул ликвидности
#[derive(Accounts)]
pub struct MintToLiquidity<'info> {
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
    
    /// Аккаунт для резервного хранения части токенов администратором
    #[account(
        mut,
        constraint = admin_token_account.mint == mint.key(),
        constraint = admin_token_account.owner == authority.key()
    )]
    pub admin_token_account: Account<'info, TokenAccount>,
    
    /// Аккаунт пула ликвидности для хранения N-Dollar
    #[account(
        mut,
        constraint = liquidity_pool_account.mint == mint.key()
    )]
    pub liquidity_pool_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// Опциональная программа liquidity-manager для CPI вызовов
    /// CHECK: ID программы liquidity-manager
    pub liquidity_manager_program: Option<AccountInfo<'info>>,
    
    /// Опциональный admin_config аккаунт из программы admin_control для проверки авторизации
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова
    pub admin_config: Option<AccountInfo<'info>>,
    
    /// Опциональная программа admin_control для CPI вызовов
    /// CHECK: ID программы admin_control
    pub admin_control_program: Option<AccountInfo<'info>>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
