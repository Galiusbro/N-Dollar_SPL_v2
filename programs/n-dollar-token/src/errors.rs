use anchor_lang::prelude::*;

#[error_code]
pub enum NDollarError {
    #[msg("Несанкционированный доступ")]
    UnauthorizedAccess,
    #[msg("Слишком рано для минтинга, должна пройти неделя между минтами")]
    TooEarlyToMint,
    #[msg("Обнаружена атака на время, несоответствие в данных блока")]
    TimeManipulationDetected,
    #[msg("Недостаточное количество подтверждений для критической операции")]
    InsufficientSigners,
    #[msg("Недопустимая сумма для минтинга")]
    InvalidMintAmount,
    #[msg("Арифметическая ошибка при вычислениях")]
    ArithmeticError,
    #[msg("Не удалось создать токен")]
    TokenCreationFailed,
    #[msg("Не удалось создать метаданные")]
    MetadataCreationFailed,
    #[msg("Не удалось создать токен-счет")]
    TokenAccountCreationFailed,
}
