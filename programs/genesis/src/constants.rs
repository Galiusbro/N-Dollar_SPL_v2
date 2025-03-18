/// Константы, используемые в программе

// Общая информация о токене
pub const DECIMALS: u8 = 9;
pub const MAX_SUPPLY: u64 = 1_000_000_000 * 10u64.pow(DECIMALS as u32); // 1 миллиард токенов
pub const INITIAL_SUPPLY_PERCENTAGE: u64 = 10; // 10% от максимального предложения

// Параметры бондинговой кривой
pub const BONDING_CURVE_POWER: u8 = 2; // Стандартный показатель степени для кривой
pub const INITIAL_PRICE: u64 = 5_000_000; // Начальная цена (0.00005 N-Dollar с учетом 9 десятичных знаков)
pub const FEE_PERCENT: u16 = 50; // 0.5% комиссия (в базисных пунктах)

// Ограничения для имени токена
pub const MIN_NAME_LENGTH: usize = 3;
pub const MAX_NAME_LENGTH: usize = 40;

// Ограничения для символа токена
pub const MIN_SYMBOL_LENGTH: usize = 2;
pub const MAX_SYMBOL_LENGTH: usize = 8;

// Допустимые символы
pub const VALID_SPECIAL_CHARS: &str = "-_.:;,?!()[]{}\"'";