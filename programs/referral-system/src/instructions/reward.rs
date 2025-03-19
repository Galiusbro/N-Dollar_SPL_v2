use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::contexts::RewardReferral;
use crate::errors::ReferralError;
use crate::instructions::utils::{verify_token_accounts, verify_token_account_owners, calculate_referrer_reward};

/// Начисление вознаграждений обоим пользователям
pub fn reward_referral(
    ctx: Context<RewardReferral>,
    amount: u64,
) -> Result<()> {
    let user_data = &mut ctx.accounts.user_data;
    let referral_data = &mut ctx.accounts.referral_data;
    
    // Проверка, что вызывающий имеет права на начисление вознаграждений
    require!(
        ctx.accounts.authority.key() == referral_data.creator,
        ReferralError::UnauthorizedAccess
    );
    
    // Проверка, что пользователь действительно был приглашен по этой реферальной ссылке
    require!(
        user_data.referrer == referral_data.creator,
        ReferralError::InvalidReferralRelationship
    );
    
    // Проверка, что все токен-аккаунты соответствуют нужному mint
    verify_token_accounts(
        &user_data.coin_mint,
        &[
            &ctx.accounts.reward_source,
            &ctx.accounts.user_token_account,
            &ctx.accounts.referrer_token_account
        ]
    )?;
    
    // Проверка, что токен-аккаунты принадлежат соответствующим владельцам
    verify_token_account_owners(&ctx.accounts.user_token_account, &user_data.user)?;
    verify_token_account_owners(&ctx.accounts.referrer_token_account, &user_data.referrer)?;
    
    // Начисляем вознаграждение пользователю
    let transfer_to_user_instruction = Transfer {
        from: ctx.accounts.reward_source.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_to_user_instruction,
    );
    
    token::transfer(cpi_ctx, amount)?;
    
    // Начисляем вознаграждение реферреру (например, 10% от суммы)
    let referrer_reward = calculate_referrer_reward(amount);
    
    let transfer_to_referrer_instruction = Transfer {
        from: ctx.accounts.reward_source.to_account_info(),
        to: ctx.accounts.referrer_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_to_referrer_instruction,
    );
    
    token::transfer(cpi_ctx, referrer_reward)?;
    
    // Обновляем статистику
    user_data.total_rewards += amount;
    referral_data.total_rewards += amount + referrer_reward;
    
    msg!("Вознаграждения успешно начислены");
    Ok(())
}
