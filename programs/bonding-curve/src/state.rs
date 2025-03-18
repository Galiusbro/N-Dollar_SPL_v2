use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;

/// Счетчик бондинговых кривых в системе
#[account]
pub struct BondingCurveCounter {
    pub count: u64,
    pub bump: u8,
}

/// Основная структура для хранения данных бондинговой кривой
#[account]
pub struct BondingCurve {
    /// Монета, которая торгуется через бондинговую кривую
    pub coin_mint: Pubkey,
    
    /// Mint NDollar, используемый для покупки/продажи токенов
    pub ndollar_mint: Pubkey,
    
    /// Создатель бондинговой кривой
    pub creator: Pubkey,
    
    /// Степень кривой (1=линейная, 2=квадратичная и т.д.)
    pub power: u8,
    
    /// Начальная цена токена
    pub initial_price: u64,
    
    /// Комиссия в базисных пунктах (1/100 процента)
    pub fee_percent: u16,
    
    /// Аккаунт пула ликвидности, хранящий NDollar
    pub liquidity_pool: Pubkey,
    
    /// Общее количество токенов в обращении через кривую
    pub total_supply_in_curve: u64,
    
    /// Текущий баланс резерва (NDollar в пуле ликвидности)
    pub reserve_balance: u64,
    
    /// Константа product для кривой (используется для проверки)
    pub constant_product: u128,
    
    /// Время последнего обновления кривой
    pub last_update_time: i64,
    
    /// Bump для PDA
    pub bump: u8,
    
    /// Программа admin_control
    pub admin_control_program: Pubkey,
}

impl BondingCurve {
    // Размер структуры для выделения памяти
    pub const SPACE: usize = 32 + 32 + 32 + 1 + 8 + 2 + 32 + 8 + 8 + 16 + 8 + 1 + 32;
} 