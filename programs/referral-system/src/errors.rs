use anchor_lang::prelude::*;

#[error_code]
pub enum ReferralError {
    #[msg("Недействительная реферальная ссылка")]
    InvalidReferralLink,
    #[msg("Несанкционированный доступ")]
    UnauthorizedAccess,
    #[msg("Недействительные отношения реферала")]
    InvalidReferralRelationship,
    #[msg("Пользователь уже зарегистрирован")]
    AlreadyRegistered,
    #[msg("Недействительный токен-аккаунт")]
    InvalidTokenAccount,
    #[msg("Недействительный владелец токен-аккаунта")]
    InvalidTokenAccountOwner,
}