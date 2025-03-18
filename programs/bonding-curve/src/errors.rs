use anchor_lang::prelude::*;

#[error_code]
pub enum BondingCurveError {
    #[msg("Неавторизованный доступ к программе")]
    UnauthorizedAccess,
    
    #[msg("Недостаточно средств")]
    InsufficientFunds,
    
    #[msg("Недостаточно ликвидности")]
    InsufficientLiquidity,
    
    #[msg("Недостаточно токенов")]
    InsufficientTokens,
    
    #[msg("Нулевое количество")]
    ZeroAmount,
    
    #[msg("Нулевой результат операции")]
    ZeroOutput,
    
    #[msg("Сумма слишком мала")]
    AmountTooSmall,
    
    #[msg("Транзакция слишком большая")]
    TransactionTooLarge,
    
    #[msg("Деление на ноль")]
    ZeroDivision,
    
    #[msg("Арифметическая ошибка")]
    ArithmeticError,
    
    #[msg("Неверный параметр")]
    InvalidParameter,
} 