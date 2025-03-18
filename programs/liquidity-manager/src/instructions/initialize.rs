use anchor_lang::prelude::*;
use crate::contexts::InitializeLiquidityManager;
use crate::constants::*;

/// Инициализация менеджера ликвидности
pub fn initialize_liquidity_manager(
    ctx: Context<InitializeLiquidityManager>,
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    liquidity_manager.authority = ctx.accounts.authority.key();
    liquidity_manager.n_dollar_mint = ctx.accounts.n_dollar_mint.key();
    liquidity_manager.total_liquidity = 0;
    liquidity_manager.total_users = 0;
    liquidity_manager.current_price = INITIAL_PRICE; // 1 SOL = 1000 N-Dollar (с учетом децималов)
    liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
    liquidity_manager.last_large_swap_time = 0;
    liquidity_manager.last_large_swap_amount = 0;
    liquidity_manager.last_large_swap_direction = true;
    liquidity_manager.price_impact_cooldown = PRICE_STABILITY_WINDOW as u64;
    liquidity_manager.bump = ctx.bumps.liquidity_manager;
    
    msg!("Менеджер ликвидности успешно инициализирован");
    Ok(())
}