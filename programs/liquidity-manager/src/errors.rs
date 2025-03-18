use anchor_lang::prelude::*;

#[error_code]
pub enum LiquidityError {
    #[msg("Недостаточно ликвидности")]
    InsufficientLiquidity,
    #[msg("Недостаточно токенов на балансе")]
    InsufficientTokenBalance,
    #[msg("Арифметическая ошибка")]
    ArithmeticError,
    #[msg("Несанкционированный доступ")]
    UnauthorizedAccess,
    #[msg("Недопустимая сумма")]
    InvalidAmount,
    #[msg("Превышен максимальный лимит")]
    ExceedsMaximumAmount,
    #[msg("Превышен лимит на размер свопа")]
    ExceedsMaximumSwapLimit,
    #[msg("Обнаружена попытка манипуляции ценой")]
    PriceManipulationDetected,
    #[msg("Слишком частые крупные транзакции")]
    TooFrequentLargeTransactions,
    #[msg("Превышен лимит проскальзывания цены")]
    SlippageExceeded,
}