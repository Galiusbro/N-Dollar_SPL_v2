use anchor_lang::prelude::*;
use crate::state::AdminConfig;
use crate::ErrorCode;

/// Контекст для инициализации Bonding Curve
#[derive(Accounts)]
pub struct InitializeBondingCurve<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Программа Bonding Curve
    /// CHECK: Это идентификатор программы Bonding Curve, который записывается в конфигурацию
    pub bonding_curve_program: AccountInfo<'info>,
}

/// Контекст для инициализации Genesis модуля
#[derive(Accounts)]
pub struct InitializeGenesis<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Программа Genesis
    /// CHECK: Это идентификатор программы Genesis, который записывается в конфигурацию
    pub genesis_program: AccountInfo<'info>,
}

/// Контекст для инициализации Referral System модуля
#[derive(Accounts)]
pub struct InitializeReferralSystem<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Программа Referral System
    /// CHECK: Это идентификатор программы Referral System, который записывается в конфигурацию
    pub referral_system_program: AccountInfo<'info>,
}

/// Контекст для инициализации Trading Exchange модуля
#[derive(Accounts)]
pub struct InitializeTradingExchange<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Программа Trading Exchange
    /// CHECK: Это идентификатор программы Trading Exchange, который записывается в конфигурацию
    pub trading_exchange_program: AccountInfo<'info>,
}

/// Контекст для инициализации Liquidity Manager модуля
#[derive(Accounts)]
pub struct InitializeLiquidityManager<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Программа Liquidity Manager
    /// CHECK: Это идентификатор программы Liquidity Manager, который записывается в конфигурацию
    pub liquidity_manager_program: AccountInfo<'info>,
} 