/// Константы для бондинговой кривой
pub mod bonding_curve {
    // Степенной показатель кривой по умолчанию (2 = квадратичная кривая)
    pub const DEFAULT_POWER: u8 = 2;
    
    // Комиссия по умолчанию в базисных пунктах (50 = 0.5%)
    pub const DEFAULT_FEE_PERCENT: u16 = 50;
    
    // Минимальная начальная цена
    pub const MIN_INITIAL_PRICE: u64 = 1;
    
    // Максимальный размер транзакции в единицах токена 
    pub const MAX_TOKEN_TRANSACTION: u64 = 1_000_000_000;
    
    // Максимальный размер транзакции в N-Dollar (1000 N-Dollar, с 9 знаками после запятой)
    pub const MAX_NDOLLAR_TRANSACTION: u64 = 1_000_000_000 * 10u64.pow(9);
    
    // Максимальное значение fee_percent (10% = 1000 базисных пунктов)
    pub const MAX_FEE_PERCENT: u16 = 1000;
    
    // Минимальное количество токенов, которое можно купить
    pub const MIN_TOKEN_AMOUNT: u64 = 1;
    
    // Минимальные значения для предотвращения числовых ошибок
    pub const MIN_SAFE_SUPPLY: u64 = 10;
}