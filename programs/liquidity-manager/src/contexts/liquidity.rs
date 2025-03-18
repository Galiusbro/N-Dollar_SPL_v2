use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct ManageLiquidity<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"liquidity_manager".as_ref(), authority.key().as_ref()],
        bump = liquidity_manager.bump,
        constraint = liquidity_manager.authority == authority.key()
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    #[account(
        mut,
        constraint = authority_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = authority_ndollar_account.owner == authority.key()
    )]
    pub authority_ndollar_account: Account<'info, TokenAccount>,
    
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