use anchor_lang::prelude::*;
use crate::contexts::InitializeReferralSystem;

/// Инициализация реферальной системы для монеты
pub fn initialize_referral_system(
    ctx: Context<InitializeReferralSystem>,
    coin_mint: Pubkey,
) -> Result<()> {
    let referral_system = &mut ctx.accounts.referral_system;
    referral_system.authority = ctx.accounts.authority.key();
    referral_system.coin_mint = coin_mint;
    referral_system.creation_time = Clock::get()?.unix_timestamp;
    referral_system.total_referrals = 0;
    referral_system.total_rewards = 0;
    referral_system.bump = ctx.bumps.referral_system;

    msg!("Реферальная система успешно инициализирована для монеты: {}", coin_mint);
    Ok(())
}
