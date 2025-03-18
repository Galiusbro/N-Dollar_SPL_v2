use anchor_lang::prelude::*;
use crate::contexts::{
    InitializeBondingCurve, 
    InitializeGenesis, 
    InitializeReferralSystem, 
    InitializeTradingExchange, 
    InitializeLiquidityManager
};

/// Инициализация Bonding Curve модуля
pub fn initialize_bonding_curve(ctx: Context<InitializeBondingCurve>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.bonding_curve_program = ctx.accounts.bonding_curve_program.key();
    
    // Устанавливаем бит, что Bonding Curve инициализирован
    admin_config.initialized_modules |= 2; // 00000010
    
    msg!("Bonding Curve initialized with program ID: {}", admin_config.bonding_curve_program);
    Ok(())
}

/// Инициализация Genesis модуля
pub fn initialize_genesis(ctx: Context<InitializeGenesis>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.genesis_program = ctx.accounts.genesis_program.key();
    
    // Устанавливаем бит, что Genesis инициализирован
    admin_config.initialized_modules |= 4; // 00000100
    
    msg!("Genesis initialized with program ID: {}", admin_config.genesis_program);
    Ok(())
}

/// Инициализация Referral System модуля
pub fn initialize_referral_system(ctx: Context<InitializeReferralSystem>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.referral_system_program = ctx.accounts.referral_system_program.key();
    
    // Устанавливаем бит, что Referral System инициализирован
    admin_config.initialized_modules |= 8; // 00001000
    
    msg!("Referral System initialized with program ID: {}", admin_config.referral_system_program);
    Ok(())
}

/// Инициализация Trading Exchange модуля
pub fn initialize_trading_exchange(ctx: Context<InitializeTradingExchange>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.trading_exchange_program = ctx.accounts.trading_exchange_program.key();
    
    // Устанавливаем бит, что Trading Exchange инициализирован
    admin_config.initialized_modules |= 16; // 00010000
    
    msg!("Trading Exchange initialized with program ID: {}", admin_config.trading_exchange_program);
    Ok(())
}

/// Инициализация Liquidity Manager модуля
pub fn initialize_liquidity_manager(ctx: Context<InitializeLiquidityManager>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.liquidity_manager_program = ctx.accounts.liquidity_manager_program.key();
    
    // Устанавливаем бит, что Liquidity Manager инициализирован
    admin_config.initialized_modules |= 32; // 00100000
    
    msg!("Liquidity Manager initialized with program ID: {}", admin_config.liquidity_manager_program);
    Ok(())
} 