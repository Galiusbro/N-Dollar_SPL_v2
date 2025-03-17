use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use borsh::BorshDeserialize;

declare_id!("EH6KMczMwjBNHUougk6oUU2PTF7aUxzJF1hbwyEdRnJd");

// Модуль CPI для использования в других программах
pub mod admin_cpi;

#[program]
pub mod admin_control {
    use super::*;

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

    /// Обновление комиссионных ставок в админской конфигурации
    pub fn update_fees(ctx: Context<UpdateFees>, fee_basis_points: u16) -> Result<()> {
        // Проверка, что комиссия не превышает 10%
        require!(
            fee_basis_points <= 1000, 
            ErrorCode::FeeTooHigh
        );
        
        let admin_config = &mut ctx.accounts.admin_config;
        admin_config.fee_basis_points = fee_basis_points;
        
        msg!("Fee updated to {} basis points", fee_basis_points);
        Ok(())
    }

    /// Авторизация новой программы для взаимодействия с экосистемой
    pub fn authorize_program(ctx: Context<AuthorizeProgram>, program_id: Pubkey) -> Result<()> {
        let admin_config = &mut ctx.accounts.admin_config;
        
        // Проверяем, не добавлена ли уже эта программа
        for i in 0..admin_config.authorized_programs.len() {
            if admin_config.authorized_programs[i] == program_id {
                return Err(ErrorCode::ProgramAlreadyAuthorized.into());
            }
            
            // Если нашли пустой слот, добавляем новую программу
            if admin_config.authorized_programs[i] == Pubkey::default() {
                admin_config.authorized_programs[i] = program_id;
                msg!("Program {} is now authorized", program_id);
                return Ok(());
            }
        }
        
        // Если не нашли свободных слотов
        Err(ErrorCode::TooManyAuthorizedPrograms.into())
    }

    /// Отзыв авторизации у программы
    pub fn revoke_program_authorization(ctx: Context<RevokeProgram>, program_id: Pubkey) -> Result<()> {
        let admin_config = &mut ctx.accounts.admin_config;
        
        for i in 0..admin_config.authorized_programs.len() {
            if admin_config.authorized_programs[i] == program_id {
                admin_config.authorized_programs[i] = Pubkey::default();
                msg!("Program {} authorization revoked", program_id);
                return Ok(());
            }
        }
        
        Err(ErrorCode::ProgramNotAuthorized.into())
    }

    /// Обновление версии структуры AdminConfig при необходимости
    pub fn upgrade_admin_config(ctx: Context<UpgradeAdminConfig>) -> Result<()> {
        let admin_config = &mut ctx.accounts.admin_config;
        
        // Проверяем, что текущая версия ниже чем последняя
        require!(
            admin_config.version < AdminConfig::CURRENT_VERSION,
            ErrorCode::AlreadyUpgraded
        );
        
        // Обновляем версию
        admin_config.version = AdminConfig::CURRENT_VERSION;
        
        // Здесь можно добавить логику миграции данных между версиями
        
        msg!("Admin Config upgraded to version {}", admin_config.version);
        Ok(())
    }
}

/// Контекст для инициализации Admin Control
#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + AdminConfig::SPACE
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    pub system_program: Program<'info, System>,
}

/// Контекст для инициализации N-Dollar
#[derive(Accounts)]
pub struct InitializeNDollar<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Mint аккаунт токена N-Dollar
    pub ndollar_mint: Account<'info, Mint>,
}

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

/// Контекст для обновления комиссий
#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Контекст для авторизации программы
#[derive(Accounts)]
pub struct AuthorizeProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Контекст для отзыва авторизации программы
#[derive(Accounts)]
pub struct RevokeProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Контекст для обновления версии AdminConfig
#[derive(Accounts)]
pub struct UpgradeAdminConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Структура для хранения конфигурации админа
#[account]
pub struct AdminConfig {
    /// Версия структуры данных
    pub version: u8,
    
    /// Адрес владельца и админа
    pub authority: Pubkey,
    
    /// Mint-аккаунт токена N-Dollar
    pub ndollar_mint: Pubkey,
    
    /// Программа Bonding Curve
    pub bonding_curve_program: Pubkey,
    
    /// Программа Genesis
    pub genesis_program: Pubkey,
    
    /// Программа Referral System
    pub referral_system_program: Pubkey,
    
    /// Программа Trading Exchange
    pub trading_exchange_program: Pubkey,
    
    /// Программа Liquidity Manager
    pub liquidity_manager_program: Pubkey,
    
    /// Список авторизованных программ, которые могут взаимодействовать с экосистемой
    pub authorized_programs: [Pubkey; 10],
    
    /// Биты инициализации модулей (1 = N-Dollar, 2 = Bonding Curve, 4 = Genesis, и т.д.)
    pub initialized_modules: u8,
    
    /// Комиссия в базисных пунктах (1/100 процента) - например, 30 = 0.3%
    pub fee_basis_points: u16,
    
    /// Bump для PDA admin_config
    pub bump: u8,
}

impl AdminConfig {
    // Текущая версия структуры
    pub const CURRENT_VERSION: u8 = 1;
    
    // Размер структуры для размещения в хранилище
    pub const SPACE: usize = 1 + // version
                              32 + // authority
                              32 + // ndollar_mint
                              32 + // bonding_curve_program
                              32 + // genesis_program
                              32 + // referral_system_program
                              32 + // trading_exchange_program
                              32 + // liquidity_manager_program
                              (32 * 10) + // authorized_programs
                              1 + // initialized_modules
                              2 + // fee_basis_points
                              1; // bump
}

/// Коды ошибок программы
#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
    
    #[msg("Program is already authorized")]
    ProgramAlreadyAuthorized,
    
    #[msg("Too many authorized programs")]
    TooManyAuthorizedPrograms,
    
    #[msg("Program is not authorized")]
    ProgramNotAuthorized,
    
    #[msg("Fee is too high, max is 10% (1000 basis points)")]
    FeeTooHigh,
    
    #[msg("Admin Config is already upgraded")]
    AlreadyUpgraded,
}
