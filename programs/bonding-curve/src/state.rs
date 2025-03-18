use anchor_lang::prelude::*;

#[account]
pub struct BondingCurve {
    pub coin_mint: Pubkey,            // Мемкоин
    pub ndollar_mint: Pubkey,         // Резервная валюта (NDollar)
    pub creator: Pubkey,              // Создатель кривой
    pub power: u8,                    // Степенной показатель для кривой
    pub initial_price: u64,           // Начальная цена токена
    pub fee_percent: u16,             // Процент комиссии в базисных пунктах (1% = 100)
    pub liquidity_pool: Pubkey,       // Хранилище резервной валюты
    pub total_supply_in_curve: u64,   // Текущее предложение токена
    pub reserve_balance: u64,         // Баланс резервной валюты
    pub constant_product: u128,       // Константа произведения
    pub last_update_time: i64,        // Время последнего обновления
    pub admin_control_program: Pubkey, // Программа admin_control для авторизации
    pub bump: u8,                     // Bump для PDA
}

impl BondingCurve {
    pub const SPACE: usize = 32 + 32 + 32 + 1 + 8 + 2 + 32 + 8 + 8 + 16 + 8 + 32 + 1;
}