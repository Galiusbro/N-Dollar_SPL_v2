/// Константы для проверок
pub const MIN_SOL_FOR_FEES: u64 = 1_000_000; // 0.001 SOL для комиссий

/// Константы для расчетов комиссий
pub const DEFAULT_FEE_PERCENTAGE: u64 = 1; // 1% по умолчанию

/// Константы для расчета ликвидности
pub const SMALL_POOL_PERCENTAGE: u64 = 1; // 0.1% от пула считается малой суммой
pub const LARGE_POOL_PERCENTAGE: u64 = 10; // 10% от пула считается крупной суммой
pub const SLIPPAGE_ADJUSTMENT_LARGE: u64 = 85; // 85% от линейной оценки для крупных сумм
pub const SLIPPAGE_ADJUSTMENT_SMALL: u64 = 90; // 90% от линейной оценки для средних сумм

/// Семена для аккаунтов
pub const EXCHANGE_DATA_SEED: &[u8] = b"exchange_data";
pub const TRADING_EXCHANGE_SEED: &[u8] = b"trading_exchange";
pub const CONTROL_STATE_SEED: &[u8] = b"control_state";
pub const BONDING_CURVE_SEED: &[u8] = b"bonding_curve";

/// Дискриминаторы для CPI вызовов
pub const SWAP_NDOLLAR_TO_SOL_DISCRIMINATOR: &[u8] = b"global:swap_ndollar_to_sol";
pub const SWAP_SOL_TO_NDOLLAR_DISCRIMINATOR: &[u8] = b"global:swap_sol_to_ndollar";
