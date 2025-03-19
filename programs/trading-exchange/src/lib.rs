use anchor_lang::prelude::*;

declare_id!("7i7EpxhmCxmDhBvcTNFVXqq2SRQNt7HG98ANxRdcF6Dh");

pub mod constants;
pub mod errors;
pub mod state;
pub mod contexts;
pub mod instructions;
pub mod math;

use contexts::*;

#[program]
pub mod trading_exchange {
    use super::*;

    /// Инициализация данных обмена
    pub fn initialize_exchange(
        ctx: Context<InitializeExchange>,
    ) -> Result<()> {
        instructions::initialize::initialize_exchange(ctx)
    }

    /// Инициализация торговой биржи
    pub fn initialize_trading_exchange(
        ctx: Context<InitializeTradingExchange>,
        n_dollar_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize::initialize_trading_exchange(ctx, n_dollar_mint)
    }

    /// Создание контрольных аккаунтов программы через PDA
    pub fn initialize_control_accounts(
        ctx: Context<InitializeControlAccounts>,
    ) -> Result<()> {
        instructions::initialize::initialize_control_accounts(ctx)
    }

    /// Свап между различными токенами по их курсу относительно N-Dollar
    pub fn swap_tokens(
        ctx: Context<SwapTokens>,
        amount_in: u64,
    ) -> Result<()> {
        instructions::swap::swap_tokens(ctx, amount_in)
    }

    /// Обмен N-Dollar на SOL через Liquidity Manager
    pub fn swap_ndollar_to_sol(
        ctx: Context<SwapNDollarToSol>,
        ndollar_amount: u64,
    ) -> Result<()> {
        instructions::swap::swap_ndollar_to_sol(ctx, ndollar_amount)
    }

    /// Обмен SOL на N-Dollar через Liquidity Manager
    pub fn swap_sol_to_ndollar(
        ctx: Context<SwapSolToNDollar>,
        sol_amount: u64,
    ) -> Result<()> {
        instructions::swap::swap_sol_to_ndollar(ctx, sol_amount)
    }
}