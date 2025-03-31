// referral_program/src/lib.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

// !!! ЗАМЕНИТЕ ЭТО НА ВАШ РЕАЛЬНЫЙ PROGRAM ID ПОСЛЕ ПЕРВОГО ДЕПЛОЯ !!!
declare_id!("DMQh8Evpe3y4DzAWxx1rhLuGpnZGDvFSPLJvD9deQQfX");

#[program]
pub mod referral_program {
    use super::*;

    pub fn process_referral(ctx: Context<ProcessReferral>) -> Result<()> {
        msg!("Processing referral...");

        // Проверяем, не был ли этот реферал уже обработан
        let referral_status = &ctx.accounts.referral_status;
        require!(!referral_status.processed, ErrorCode::AlreadyProcessed);

        // Получаем децималы токена для расчета награды
        let token_decimals = ctx.accounts.mint.decimals;
        let reward_amount = 1 * 10u64.pow(token_decimals as u32); // 1 токен с децималами
        msg!("Reward amount (lamports): {}", reward_amount);

        // Проверяем, достаточно ли средств в казне
        let required_total_reward = reward_amount
            .checked_mul(2) // Награда для реферера и рефери
            .ok_or(ErrorCode::CalculationOverflow)?;
        require!(
            ctx.accounts.referral_treasury.amount >= required_total_reward,
            ErrorCode::InsufficientTreasuryBalance
        );

        // Получаем bump для PDA-авторитета казны
        let mint_key = ctx.accounts.mint.key();
        let (_authority_pda, authority_bump) = Pubkey::find_program_address(
            &[b"referral".as_ref(), mint_key.as_ref()],
            ctx.program_id,
        );

        // Создаем signer seeds
        let authority_seeds = &[
            b"referral".as_ref(),
            mint_key.as_ref(),
            &[authority_bump],
        ];
        let signer = &[&authority_seeds[..]];

        // 1. Выплачиваем награду рефереру
        msg!(
            "Paying reward to referrer: {}",
            ctx.accounts.referrer_token_account.key()
        );
        let cpi_accounts_referrer = Transfer {
            from: ctx.accounts.referral_treasury.to_account_info(),
            to: ctx.accounts.referrer_token_account.to_account_info(),
            authority: ctx.accounts.referral_authority.to_account_info(), // PDA подписывает
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_referrer =
            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts_referrer, signer);
        token::transfer(cpi_ctx_referrer, reward_amount)?;

        // 2. Выплачиваем награду рефери
        msg!(
            "Paying reward to referee: {}",
            ctx.accounts.referee_token_account.key()
        );
        let cpi_accounts_referee = Transfer {
            from: ctx.accounts.referral_treasury.to_account_info(),
            to: ctx.accounts.referee_token_account.to_account_info(),
            authority: ctx.accounts.referral_authority.to_account_info(), // PDA подписывает
        };
        let cpi_ctx_referee =
            CpiContext::new_with_signer(cpi_program, cpi_accounts_referee, signer);
        token::transfer(cpi_ctx_referee, reward_amount)?;

        // Помечаем реферал как обработанный
        let referral_status = &mut ctx.accounts.referral_status;
        referral_status.processed = true;
        referral_status.referrer = ctx.accounts.referrer.key();
        referral_status.referee = ctx.accounts.referee.key();
        referral_status.mint = ctx.accounts.mint.key();
        msg!(
            "Referral processed for referee: {}",
            ctx.accounts.referee.key()
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ProcessReferral<'info> {
    // --- Аккаунты для проверки и выплат ---
    pub mint: Account<'info, Mint>, // Токен, который используется для награды

    /// CHECK: PDA, который владеет казной. Проверяется через seeds + program_id.
    #[account(
        seeds = [b"referral".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub referral_authority: AccountInfo<'info>,

    #[account(
        mut, // Средства списываются отсюда
        associated_token::mint = mint,
        associated_token::authority = referral_authority, // Проверяем владельца казны
        constraint = referral_treasury.amount > 0 @ ErrorCode::InsufficientTreasuryBalance // Доп. проверка
    )]
    pub referral_treasury: Account<'info, TokenAccount>, // Казна реферальной программы

    /// CHECK: Реферер - просто адрес, проверять нечего, кроме того что он не равен рефери?
    #[account(
         constraint = referrer.key() != referee.key() @ ErrorCode::ReferrerCannotBeReferee
    )]
    pub referrer: AccountInfo<'info>, // Кто пригласил

    /// CHECK: Рефери - новый пользователь. Проверяется через referral_status.
    pub referee: AccountInfo<'info>, // Кто был приглашен

    // ATA реферера для получения награды
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = referrer, // Убедимся, что это ATA реферера
    )]
    pub referrer_token_account: Account<'info, TokenAccount>,

    // ATA рефери для получения награды
    // Может не существовать, поэтому init_if_needed
    #[account(
        init_if_needed,
        payer = payer, // Кто платит за создание ATA, если его нет
        // mut,
        associated_token::mint = mint,
        associated_token::authority = referee, // Убедимся, что это ATA рефери
    )]
    pub referee_token_account: Account<'info, TokenAccount>,

    // --- Аккаунт для отслеживания статуса ---
    #[account(
        init, // Создается при первой обработке
        payer = payer,
        space = 8 + ReferralStatus::INIT_SPACE,
        seeds = [b"status".as_ref(), mint.key().as_ref(), referee.key().as_ref()], // Уникально для mint+referee
        bump
    )]
    pub referral_status: Account<'info, ReferralStatus>,

    // --- Системные аккаунты ---
    #[account(mut)]
    pub payer: Signer<'info>, // Кто платит за создание status и referee_token_account
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Состояние для отслеживания обработанных рефералов
#[account]
#[derive(InitSpace)]
pub struct ReferralStatus {
    pub processed: bool,     // 1
    pub referrer: Pubkey,    // 32
    pub referee: Pubkey,     // 32
    pub mint: Pubkey,        // 32
}

#[error_code]
pub enum ErrorCode {
    #[msg("Referral already processed for this referee and mint.")]
    AlreadyProcessed,
    #[msg("Insufficient balance in treasury for rewards.")]
    InsufficientTreasuryBalance,
    #[msg("Calculation overflow.")]
    CalculationOverflow,
    #[msg("Referrer cannot be the same as the referee.")]
    ReferrerCannotBeReferee,
}