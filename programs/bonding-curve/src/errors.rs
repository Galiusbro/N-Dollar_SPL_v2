use anchor_lang::prelude::*;

#[error_code]
pub enum BondingCurveError {
    #[msg("Недостаточно средств для покупки")]
    InsufficientFunds,
    #[msg("Недостаточно токенов для продажи")]
    InsufficientTokens,
    #[msg("Недостаточно ликвидности в пуле")]
    InsufficientLiquidity,
    #[msg("Арифметическая ошибка при расчете")]
    ArithmeticError,
    #[msg("Деление на ноль при расчетах")]
    ZeroDivision,
    #[msg("Количество токенов должно быть больше нуля")]
    ZeroAmount,
    #[msg("Рассчитано нулевое количество токенов")]
    ZeroOutput,
    #[msg("Превышен максимальный размер транзакции")]
    TransactionTooLarge,
    #[msg("Некорректный параметр")]
    InvalidParameter,
    #[msg("Ошибка при выполнении транзакции токена")]
    TokenTransferError,
    #[msg("Слишком маленькое количество токенов для операции")]
    AmountTooSmall,
    #[msg("Слишком большое количество токенов для операции")]
    AmountTooLarge,
    #[msg("Отсутствует необходимый аккаунт")]
    MissingAccount,
    #[msg("Неавторизованный доступ")]
    UnauthorizedAccess,
}