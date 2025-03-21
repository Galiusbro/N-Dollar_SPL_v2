use anchor_lang::prelude::*;

declare_id!("4uP4rbMsgqMf9GKSAbbiDKKLJ1a2Rp4SEYE5jdhiVYLU");

pub mod constants;
pub mod errors;
pub mod state;
pub mod contexts;
pub mod instructions;
// #[cfg(feature = "cpi")]
pub mod liquidity_cpi;

use contexts::*;

#[program]
pub mod liquidity_manager {
    use super::*;

    /// Инициализация менеджера ликвидности
    pub fn initialize_liquidity_manager(
        ctx: Context<InitializeLiquidityManager>,
    ) -> Result<()> {
        instructions::initialize::initialize_liquidity_manager(ctx)
    }
    
    /// Покупка N-Dollar за SOL
    pub fn swap_sol_to_ndollar(
        ctx: Context<SwapSolToNDollar>,
        sol_amount: u64,
    ) -> Result<()> {
        instructions::swap::swap_sol_to_ndollar(ctx, sol_amount)
    }
    
    /// Обмен N-Dollar на SOL
    pub fn swap_ndollar_to_sol(
        ctx: Context<SwapNDollarToSol>,
        ndollar_amount: u64,
    ) -> Result<()> {
        instructions::swap::swap_ndollar_to_sol(ctx, ndollar_amount)
    }
    
    /// Покупка N-Dollar за SOL с защитой от проскальзывания
    pub fn swap_sol_to_ndollar_with_slippage(
        ctx: Context<SwapSolToNDollar>,
        sol_amount: u64,
        min_ndollar_amount: u64,
    ) -> Result<()> {
        instructions::swap::swap_sol_to_ndollar_with_slippage(ctx, sol_amount, min_ndollar_amount)
    }
    
    /// Добавление ликвидности в пул (только для владельца)
    pub fn add_liquidity(
        ctx: Context<ManageLiquidity>,
        sol_amount: u64,
        ndollar_amount: u64,
    ) -> Result<()> {
        instructions::liquidity::add_liquidity(ctx, sol_amount, ndollar_amount)
    }
    
    /// Изъятие ликвидности из пула (только для владельца)
    pub fn remove_liquidity(
        ctx: Context<ManageLiquidity>,
        sol_amount: u64,
        ndollar_amount: u64,
    ) -> Result<()> {
        instructions::liquidity::remove_liquidity(ctx, sol_amount, ndollar_amount)
    }

    /// Обмен N-Dollar на SOL с защитой от проскальзывания
    pub fn swap_ndollar_to_sol_with_slippage(
        ctx: Context<SwapNDollarToSol>,
        ndollar_amount: u64,
        min_sol_amount: u64,
    ) -> Result<()> {
        instructions::swap::swap_ndollar_to_sol_with_slippage(ctx, ndollar_amount, min_sol_amount)
    }
    
    /// Обновление состояния ликвидности после прямого минта
    pub fn update_after_mint(
        ctx: Context<UpdateAfterMint>,
        ndollar_amount: u64,
    ) -> Result<()> {
        instructions::liquidity::update_after_mint(ctx, ndollar_amount)
    }
}