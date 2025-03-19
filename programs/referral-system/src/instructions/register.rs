use anchor_lang::prelude::*;
use crate::contexts::RegisterReferral;
use crate::errors::ReferralError;
use crate::constants::USER_DATA_SEED;
use crate::instructions::utils::verify_registration_eligibility;

/// Регистрация нового пользователя по реферальной ссылке
pub fn register_referral(
    ctx: Context<RegisterReferral>,
    coin_mint: Pubkey,
) -> Result<()> {
    let referral_data = &mut ctx.accounts.referral_data;
    
    // Проверка, что реферальная ссылка действительна
    require!(
        referral_data.coin_mint == coin_mint,
        ReferralError::InvalidReferralLink
    );
    
    // Проверка на двойную регистрацию (дополнительная проверка)
    let user_key = ctx.accounts.user.key();
    let coin_mint_ref = referral_data.coin_mint;
    
    let user_pda_seeds = &[
        USER_DATA_SEED,
        user_key.as_ref(),
        coin_mint_ref.as_ref(),
    ];
    
    let user_data_key = ctx.accounts.user_data.key();
    let (user_pda, _) = Pubkey::find_program_address(user_pda_seeds, ctx.program_id);
    
    // Проверяем, что аккаунт на самом деле создается впервые
    verify_registration_eligibility(user_pda, user_data_key)?;
    
    // Инициализация данных пользователя
    let user_data = &mut ctx.accounts.user_data;
    user_data.user = user_key;
    user_data.referrer = referral_data.creator;
    user_data.coin_mint = coin_mint;
    user_data.registration_time = Clock::get()?.unix_timestamp;
    user_data.total_rewards = 0;
    user_data.bump = ctx.bumps.user_data;
    
    // Обновляем статистику реферальной системы
    referral_data.referred_users += 1;
    
    msg!("Пользователь успешно зарегистрирован по реферальной ссылке");
    Ok(())
}
