use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;

#[derive(Accounts)]
pub struct TradeToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"bonding_curve".as_ref(), coin_mint.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(
        mut,
        constraint = coin_mint.key() == bonding_curve.coin_mint
    )]
    pub coin_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = ndollar_mint.key() == bonding_curve.ndollar_mint
    )]
    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = buyer_coin_account.mint == coin_mint.key(),
        constraint = buyer_coin_account.owner == buyer.key()
    )]
    pub buyer_coin_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = buyer_ndollar_account.mint == ndollar_mint.key(),
        constraint = buyer_ndollar_account.owner == buyer.key()
    )]
    pub buyer_ndollar_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidity_pool.key() == bonding_curve.liquidity_pool
    )]
    pub liquidity_pool: Account<'info, TokenAccount>,
    
    /// Admin control аккаунт для проверки авторизации
    #[account(
        seeds = [b"admin_config".as_ref(), buyer.key().as_ref()],
        bump,
        seeds::program = admin_control_program.key()
    )]
    pub admin_config: Account<'info, admin_control::AdminConfig>,
    
    /// Программа admin_control
    pub admin_control_program: Program<'info, admin_control::program::AdminControl>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}