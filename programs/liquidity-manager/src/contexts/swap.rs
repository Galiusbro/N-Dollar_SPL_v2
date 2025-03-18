use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct SwapSolToNDollar<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"liquidity_manager".as_ref(), liquidity_manager.authority.as_ref()],
        bump = liquidity_manager.bump
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Это аккаунт для хранения SOL, принадлежащий пулу ликвидности
    #[account(
        mut,
        seeds = [b"pool_sol".as_ref(), liquidity_manager.key().as_ref()],
        bump,
    )]
    pub pool_sol_account: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = pool_ndollar_account.mint == liquidity_manager.n_dollar_mint
    )]
    pub pool_ndollar_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SwapNDollarToSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"liquidity_manager".as_ref(), liquidity_manager.authority.as_ref()],
        bump = liquidity_manager.bump
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Это аккаунт для хранения SOL, принадлежащий пулу ликвидности
    #[account(
        mut,
        seeds = [b"pool_sol".as_ref(), liquidity_manager.key().as_ref()],
        bump,
    )]
    pub pool_sol_account: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = pool_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = pool_ndollar_account.owner == liquidity_manager.key()
    )]
    pub pool_ndollar_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}