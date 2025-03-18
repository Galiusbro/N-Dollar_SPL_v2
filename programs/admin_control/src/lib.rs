use anchor_lang::prelude::*;
use borsh::BorshDeserialize;

// Импортируем модуль ошибок
mod error;
pub use error::ErrorCode;

// Импортируем модуль состояния
mod state;
pub use state::*;

// Импортируем модуль контекстов
mod contexts;
pub use contexts::*;

// Импортируем модуль инструкций
mod instructions;

declare_id!("EH6KMczMwjBNHUougk6oUU2PTF7aUxzJF1hbwyEdRnJd");

// Модуль CPI для использования в других программах
pub mod admin_cpi;

#[program]
pub mod admin_control {
    use super::*;

    /// Инициализация базовой конфигурации Admin Control для экосистемы N-Dollar
    pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
        instructions::initialize_admin(ctx)
    }

    /// Инициализация токена N-Dollar
    pub fn initialize_ndollar(ctx: Context<InitializeNDollar>) -> Result<()> {
        instructions::initialize_ndollar(ctx)
    }

    /// Инициализация Bonding Curve модуля
    pub fn initialize_bonding_curve(ctx: Context<InitializeBondingCurve>) -> Result<()> {
        instructions::initialize_bonding_curve(ctx)
    }

    /// Инициализация Genesis модуля
    pub fn initialize_genesis(ctx: Context<InitializeGenesis>) -> Result<()> {
        instructions::initialize_genesis(ctx)
    }

    /// Инициализация Referral System модуля
    pub fn initialize_referral_system(ctx: Context<InitializeReferralSystem>) -> Result<()> {
        instructions::initialize_referral_system(ctx)
    }

    /// Инициализация Trading Exchange модуля
    pub fn initialize_trading_exchange(ctx: Context<InitializeTradingExchange>) -> Result<()> {
        instructions::initialize_trading_exchange(ctx)
    }

    /// Инициализация Liquidity Manager модуля
    pub fn initialize_liquidity_manager(ctx: Context<InitializeLiquidityManager>) -> Result<()> {
        instructions::initialize_liquidity_manager(ctx)
    }

    /// Обновление комиссионных ставок в админской конфигурации
    pub fn update_fees(ctx: Context<UpdateFees>, fee_basis_points: u16) -> Result<()> {
        instructions::update_fees(ctx, fee_basis_points)
    }

    /// Авторизация новой программы для взаимодействия с экосистемой
    pub fn authorize_program(ctx: Context<AuthorizeProgram>, program_id: Pubkey) -> Result<()> {
        instructions::authorize_program(ctx, program_id)
    }

    /// Отзыв авторизации у программы
    pub fn revoke_program_authorization(ctx: Context<RevokeProgram>, program_id: Pubkey) -> Result<()> {
        instructions::revoke_program_authorization(ctx, program_id)
    }

    /// Обновление версии структуры AdminConfig при необходимости
    pub fn upgrade_admin_config(ctx: Context<UpgradeAdminConfig>) -> Result<()> {
        instructions::upgrade_admin_config(ctx)
    }
}
