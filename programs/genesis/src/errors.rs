use anchor_lang::prelude::*;

#[error_code]
pub enum GenesisError {
    #[msg("Вы не являетесь создателем этой монеты")]
    NotCoinCreator,
    #[msg("Превышена максимальная аллокация для основателя (10%)")]
    ExceedsFounderAllocation,
    #[msg("Реферальная ссылка уже активна")]
    ReferralLinkAlreadyActive,
    #[msg("Название токена слишком короткое (минимум 3 символа)")]
    NameTooShort,
    #[msg("Название токена слишком длинное (максимум 40 символов)")]
    NameTooLong,
    #[msg("Символ токена слишком короткий (минимум 2 символа)")]
    SymbolTooShort,
    #[msg("Символ токена слишком длинный (максимум 8 символов)")]
    SymbolTooLong,
    #[msg("Название или символ содержат недопустимые символы")]
    InvalidCharacters,
    #[msg("Вы не являетесь администратором этой монеты")]
    NotCoinAdmin,
}