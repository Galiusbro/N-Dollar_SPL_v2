use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

/// Константы для ограничения размера свопов
pub const MAX_SOL_SWAP_AMOUNT: u64 = 25 * LAMPORTS_PER_SOL; // 25 SOL
pub const MAX_NDOLLAR_SWAP_AMOUNT: u64 = 25_000_000_000; // Эквивалент 25,000 N-Dollar (с учетом децималов)

/// Константы для защиты от манипуляций с ценой
pub const PRICE_IMPACT_THRESHOLD_PERCENTAGE: u64 = 5; // 5% максимальное влияние на цену
pub const PRICE_STABILITY_WINDOW: i64 = 60; // 60 секунд минимальное время между крупными транзакциями

/// Константы для цен
pub const INITIAL_PRICE: u64 = 1_000_000_000; // 1 SOL = 1000 N-Dollar (с учетом децималов)
pub const MIN_PRICE: u64 = 500_000_000; // 50% от начальной цены
pub const MAX_PRICE: u64 = 2_000_000_000; // 200% от начальной цены

/// Константы для комиссий
pub const FEE_PERCENTAGE: u64 = 1; // 1% комиссия при свопах

// TODO: Константа для защиты от проскальзывания цены
// pub const MAX_SLIPPAGE_PERCENTAGE: u64 = 3; // 3% максимальное проскальзывание для защиты пользователя