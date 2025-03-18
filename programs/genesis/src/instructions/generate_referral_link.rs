use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{Instruction, AccountMeta};
use anchor_lang::solana_program::program::invoke;

use crate::errors::GenesisError;
use crate::instructions::contexts::GenerateReferralLink;

pub fn handler(
    ctx: Context<GenerateReferralLink>,
) -> Result<()> {
    let coin_data = &mut ctx.accounts.coin_data;
    let authority = &ctx.accounts.authority;
    
    // Проверка, что вызывающий является администратором монеты
    require!(
        coin_data.admin == authority.key(),
        GenesisError::NotCoinAdmin
    );
    
    // Проверка, что реферальная ссылка еще не активирована
    require!(
        !coin_data.referral_link_active,
        GenesisError::ReferralLinkAlreadyActive
    );
    
    // Инициализируем реферальный аккаунт
    let referral_data = &mut ctx.accounts.referral_data;
    referral_data.coin_mint = coin_data.mint;
    referral_data.creator = coin_data.creator;
    referral_data.creation_time = Clock::get()?.unix_timestamp;
    referral_data.referred_users = 0;
    referral_data.total_rewards = 0;
    referral_data.bump = ctx.bumps.referral_data;
    
    // Отмечаем, что реферальная ссылка активирована
    coin_data.referral_link_active = true;
    
    // CPI для инициализации в модуле реферальной системы
    // Рассчитываем дискриминатор для инструкции initialize_referral_system
    let disc = anchor_lang::solana_program::hash::hash("global:initialize_referral_system".as_bytes());
    let initialize_referral_system_discriminator = disc.to_bytes()[..8].to_vec();
    
    // Готовим данные для инструкции
    let mut ix_data = initialize_referral_system_discriminator;
    ix_data.extend_from_slice(&ctx.accounts.mint.key().to_bytes()); // coin_mint
    
    // Определяем аккаунты для CPI вызова
    let ix_accounts = vec![
        AccountMeta::new(ctx.accounts.authority.key(), true), // authority (signer)
        AccountMeta::new(ctx.accounts.referral_system.key(), false), // referral_system PDA
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false), // system_program
        AccountMeta::new_readonly(ctx.accounts.rent.key(), false), // rent
    ];
    
    // Создаем инструкцию
    let ix = Instruction {
        program_id: ctx.accounts.referral_system_program.key(),
        accounts: ix_accounts,
        data: ix_data,
    };
    
    // Выполняем инструкцию
    invoke(
        &ix,
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.referral_system.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;
    
    msg!("Реферальная ссылка сгенерирована для монеты: {}", coin_data.symbol);
    Ok(())
}