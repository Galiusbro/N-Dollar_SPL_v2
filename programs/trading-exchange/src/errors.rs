use anchor_lang::prelude::*;

#[error_code]
pub enum TradingError {
    #[msg("Недостаточно токенов на балансе")]
    InsufficientTokenBalance,
    #[msg("Недостаточно ликвидности в пуле")]
    InsufficientLiquidity,
    #[msg("Арифметическая ошибка при расчете")]
    ArithmeticError,
    #[msg("Неверный идентификатор программы Liquidity Manager")]
    InvalidLiquidityManagerProgram,
    #[msg("Недостаточность баланса")]
    InsufficientBalance,
    #[msg("Неверная бондинговая кривая для токена")]
    InvalidBondingCurve,
    #[msg("Неверная пара токенов для обмена")]
    InvalidTokenPair,
    #[msg("Неверные параметры для расчета")]
    InvalidParameters,
    #[msg("Результат расчета равен нулю")]
    ZeroOutput,
    #[msg("Программа не активна")]
    ProgramNotActive,
    #[msg("Неавторизованный доступ")]
    UnauthorizedAccess,
}