use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::pubkey::Pubkey;

declare_id!("Cf8bAqCzB76XVY7Dmj1M6JTuQP7Z7ziSmVnsNy7Sb7Nh");

#[program]
pub mod referral_system {
    use super::*;

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
            b"user_data".as_ref(),
            user_key.as_ref(),
            coin_mint_ref.as_ref(),
        ];
        
        let user_data_key = ctx.accounts.user_data.key();
        let (user_pda, _) = Pubkey::find_program_address(user_pda_seeds, ctx.program_id);
        
        // Проверяем, что аккаунт на самом деле создается впервые
        require!(
            user_pda == user_data_key,
            ReferralError::AlreadyRegistered
        );
        
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
        require!(
            ctx.accounts.reward_source.mint == user_data.coin_mint &&
            ctx.accounts.user_token_account.mint == user_data.coin_mint &&
            ctx.accounts.referrer_token_account.mint == user_data.coin_mint,
            ReferralError::InvalidTokenAccount
        );
        
        // Проверка, что токен-аккаунты принадлежат соответствующим владельцам
        require!(
            ctx.accounts.user_token_account.owner == user_data.user,
            ReferralError::InvalidTokenAccountOwner
        );
        
        require!(
            ctx.accounts.referrer_token_account.owner == user_data.referrer,
            ReferralError::InvalidTokenAccountOwner
        );
        
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
        let referrer_reward = amount / 10;
        
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
}

#[derive(Accounts)]
pub struct InitializeReferralSystem<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"referral_system".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + ReferralSystem::SPACE
    )]
    pub referral_system: Account<'info, ReferralSystem>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RegisterReferral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"referral_data".as_ref(), referral_data.coin_mint.as_ref()],
        bump = referral_data.bump
    )]
    pub referral_data: Account<'info, ReferralData>,
    
    #[account(
        init,
        payer = user,
        seeds = [b"user_data".as_ref(), user.key().as_ref(), referral_data.coin_mint.as_ref()],
        bump,
        space = 8 + UserData::SPACE
    )]
    pub user_data: Account<'info, UserData>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RewardReferral<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"referral_data".as_ref(), referral_data.coin_mint.as_ref()],
        bump = referral_data.bump
    )]
    pub referral_data: Account<'info, ReferralData>,
    
    #[account(
        mut,
        seeds = [b"user_data".as_ref(), user_data.user.as_ref(), user_data.coin_mint.as_ref()],
        bump = user_data.bump
    )]
    pub user_data: Account<'info, UserData>,
    
    #[account(
        mut,
        constraint = reward_source.owner == authority.key()
    )]
    pub reward_source: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub referrer_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckReferralStatus<'info> {
    #[account(
        seeds = [b"referral_data".as_ref(), referral_data.coin_mint.as_ref()],
        bump = referral_data.bump
    )]
    pub referral_data: Account<'info, ReferralData>,
}

#[account]
pub struct ReferralSystem {
    pub authority: Pubkey,
    pub coin_mint: Pubkey,
    pub creation_time: i64,
    pub total_referrals: u64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl ReferralSystem {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct ReferralData {
    pub coin_mint: Pubkey,
    pub creator: Pubkey,
    pub creation_time: i64,
    pub referred_users: u64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl ReferralData {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct UserData {
    pub user: Pubkey,
    pub referrer: Pubkey,
    pub coin_mint: Pubkey,
    pub registration_time: i64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl UserData {
    pub const SPACE: usize = 32 + 32 + 32 + 8 + 8 + 1;
}

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
