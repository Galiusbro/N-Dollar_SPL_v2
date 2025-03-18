use anchor_lang::prelude::*;
use crate::contexts::{InitializeAdmin, InitializeNDollar};
use crate::state::AdminConfig;

/// Инициализация базовой конфигурации Admin Control для экосистемы N-Dollar
pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.version = AdminConfig::CURRENT_VERSION;
    admin_config.authority = ctx.accounts.authority.key();
    admin_config.bump = ctx.bumps.admin_config;
    admin_config.initialized_modules = 0;
    admin_config.fee_basis_points = 30; // 0.3% комиссия по умолчанию
    
    // Инициализация других полей пустыми значениями
    admin_config.ndollar_mint = Pubkey::default();
    admin_config.bonding_curve_program = Pubkey::default();
    admin_config.genesis_program = Pubkey::default();
    admin_config.referral_system_program = Pubkey::default();
    admin_config.trading_exchange_program = Pubkey::default();
    admin_config.liquidity_manager_program = Pubkey::default();
    admin_config.authorized_programs = [Pubkey::default(); 10];
    
    msg!("Admin Control initialized with authority: {} (version: {})", 
            admin_config.authority, admin_config.version);
    Ok(())
}

/// Инициализация токена N-Dollar
pub fn initialize_ndollar(ctx: Context<InitializeNDollar>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.ndollar_mint = ctx.accounts.ndollar_mint.key();
    
    // Устанавливаем бит, что N-Dollar инициализирован
    admin_config.initialized_modules |= 1; // 00000001
    
    msg!("N-Dollar initialized with mint: {}", admin_config.ndollar_mint);
    Ok(())
} 