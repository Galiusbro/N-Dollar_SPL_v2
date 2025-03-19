use anchor_lang::prelude::*;
use crate::contexts::CheckReferralStatus;

/// Проверка статуса реферальной ссылки
pub fn check_referral_status(
    ctx: Context<CheckReferralStatus>,
) -> Result<()> {
    let referral_data = &ctx.accounts.referral_data;
    
    msg!("Статус реферальной ссылки:");
    msg!("Создатель: {}", referral_data.creator);
    msg!("Монета: {}", referral_data.coin_mint);
    msg!("Приглашенных пользователей: {}", referral_data.referred_users);
    msg!("Всего вознаграждений: {}", referral_data.total_rewards);
    
    Ok(())
}
