use anchor_lang::prelude::*;

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