use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use bonding_curve::state::BondingCurve;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct SwapTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [EXCHANGE_DATA_SEED, exchange_data.authority.as_ref()],
        bump = exchange_data.bump
    )]
    pub exchange_data: Account<'info, ExchangeData>,
    
    #[account(
        seeds = [CONTROL_STATE_SEED, exchange_data.authority.as_ref()],
        bump,
        constraint = control_state.is_active == true
    )]
    pub control_state: Account<'info, ControlState>,
    
    pub from_mint: Account<'info, Mint>,
    pub to_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_from_account.mint == from_mint.key(),
        constraint = user_from_account.owner == user.key()
    )]
    pub user_from_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_to_account.mint == to_mint.key(),
        constraint = user_to_account.owner == user.key()
    )]
    pub user_to_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidity_from_account.mint == from_mint.key()
    )]
    pub liquidity_from_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidity_to_account.mint == to_mint.key()
    )]
    pub liquidity_to_account: Account<'info, TokenAccount>,
    
    /// Аккаунт бондинговой кривой для входящего токена
    #[account(
        seeds = [BONDING_CURVE_SEED, from_mint.key().as_ref()],
        bump,
        seeds::program = bonding_curve_program.key()
    )]
    pub from_bonding_curve: Account<'info, BondingCurve>,
    
    /// Аккаунт бондинговой кривой для исходящего токена
    #[account(
        seeds = [BONDING_CURVE_SEED, to_mint.key().as_ref()],
        bump,
        seeds::program = bonding_curve_program.key()
    )]
    pub to_bonding_curve: Account<'info, BondingCurve>,
    
    /// Опциональный admin_config аккаунт из программы admin_control для проверки авторизации
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова
    pub admin_config: Option<AccountInfo<'info>>,
    
    /// Опциональная программа admin_control для CPI вызовов
    /// CHECK: ID программы admin_control
    pub admin_control_program: Option<AccountInfo<'info>>,
    
    /// Программа бондинговой кривой
    /// CHECK: Используем AccountInfo вместо Program, чтобы обойти проблему с типами
    pub bonding_curve_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SwapNDollarToSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [TRADING_EXCHANGE_SEED, trading_exchange.authority.as_ref()],
        bump = trading_exchange.bump
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    #[account(
        seeds = [CONTROL_STATE_SEED, trading_exchange.authority.as_ref()],
        bump,
        constraint = control_state.is_active == true
    )]
    pub control_state: Account<'info, ControlState>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == trading_exchange.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: AccountInfo<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(sol_amount: u64)]
pub struct SwapSolToNDollar<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [TRADING_EXCHANGE_SEED, trading_exchange.authority.as_ref()],
        bump = trading_exchange.bump
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    #[account(
        seeds = [CONTROL_STATE_SEED, trading_exchange.authority.as_ref()],
        bump,
        constraint = control_state.is_active == true
    )]
    pub control_state: Account<'info, ControlState>,
    
    #[account(
        mut,
        constraint = user.lamports() >= sol_amount + MIN_SOL_FOR_FEES, // +0.001 SOL для комиссии
        constraint = user_ndollar_account.mint == trading_exchange.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: AccountInfo<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
