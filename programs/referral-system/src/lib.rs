use anchor_lang::prelude::*;

declare_id!("Cf8bAqCzB76XVY7Dmj1M6JTuQP7Z7ziSmVnsNy7Sb7Nh");

pub mod constants;
pub mod errors;
pub mod state;
pub mod contexts;
pub mod instructions;

use contexts::*;

#[program]
pub mod referral_system {
    use super::*;

    /// Инициализация реферальной системы для монеты
    pub fn initialize_referral_system(
        ctx: Context<InitializeReferralSystem>,
        coin_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize::initialize_referral_system(ctx, coin_mint)
    }

    /// Регистрация нового пользователя по реферальной ссылке
    pub fn register_referral(
        ctx: Context<RegisterReferral>,
        coin_mint: Pubkey,
    ) -> Result<()> {
        instructions::register::register_referral(ctx, coin_mint)
    }

    /// Начисление вознаграждений обоим пользователям
    pub fn reward_referral(
        ctx: Context<RewardReferral>,
        amount: u64,
    ) -> Result<()> {
        instructions::reward::reward_referral(ctx, amount)
    }

    /// Проверка статуса реферальной ссылки
    pub fn check_referral_status(
        ctx: Context<CheckReferralStatus>,
    ) -> Result<()> {
        instructions::status::check_referral_status(ctx)
    }
}
