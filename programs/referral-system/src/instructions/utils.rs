use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::errors::ReferralError;
use crate::constants::REFERRER_REWARD_PERCENTAGE;

/// Проверяет соответствие токен-аккаунтов используемой монете
pub fn verify_token_accounts(
    coin_mint: &Pubkey,
    token_accounts: &[&Account<TokenAccount>],
) -> Result<()> {
    for account in token_accounts {
        require!(
            account.mint == *coin_mint,
            ReferralError::InvalidTokenAccount
        );
    }
    
    Ok(())
}

/// Проверяет владельцев токен-аккаунтов
pub fn verify_token_account_owners(
    token_account: &Account<TokenAccount>,
    expected_owner: &Pubkey,
) -> Result<()> {
    require!(
        token_account.owner == *expected_owner,
        ReferralError::InvalidTokenAccountOwner
    );
    
    Ok(())
}

/// Рассчитывает вознаграждение реферрера
pub fn calculate_referrer_reward(amount: u64) -> u64 {
    amount * REFERRER_REWARD_PERCENTAGE / 100
}

/// Проверяет, что пользователь может быть зарегистрирован
pub fn verify_registration_eligibility(
    user_pda: Pubkey,
    user_data_key: Pubkey,
) -> Result<()> {
    require!(
        user_pda == user_data_key,
        ReferralError::AlreadyRegistered
    );
    
    Ok(())
}
