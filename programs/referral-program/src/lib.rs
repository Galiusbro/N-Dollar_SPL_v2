// referral_program/src/lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("DMQh8Evpe3y4DzAWxx1rhLuGpnZGDvFSPLJvD9deQQfX");

const REWARD_AMOUNT: u64 = 1_000_000_000; // 1 токен с 9 децималами

#[program]
pub mod referral_program {
    use super::*;

    pub fn process_referral(ctx: Context<ProcessReferral>) -> Result<()> {
        msg!("Processing referral...");

        // Проверяем, достаточно ли средств в казне реферальной программы
        let required_amount = REWARD_AMOUNT.checked_mul(2).ok_or(ErrorCode::CalculationOverflow)?;
        require!(
            ctx.accounts.referral_treasury_token_account.amount >= required_amount,
            ErrorCode::InsufficientTreasuryBalance
        );

        // Находим PDA казны этой программы и бамп
        let mint_key = ctx.accounts.mint.key();
        // Сиды для ПОДПИСИ CPI (принадлежат ЭТОЙ программе)
        let seeds = &[
            b"referral_treasury".as_ref(), // <-- Новый сид
            mint_key.as_ref(),
            &[ctx.bumps.referral_treasury_authority], // Используем бамп для этого PDA
        ];
        let signer_seeds = &[&seeds[..]];

        // Перевод награды первому рефералу
        msg!(
            "Transferring {} tokens to referee 1: {}",
            REWARD_AMOUNT,
            ctx.accounts.referee1_token_account.key()
        );
        let cpi_accounts_ref1 = Transfer {
            from: ctx.accounts.referral_treasury_token_account.to_account_info(), // <-- Из казны рефералки
            to: ctx.accounts.referee1_token_account.to_account_info(),
            authority: ctx.accounts.referral_treasury_authority.to_account_info(), // <-- PDA рефералки
        };
        let cpi_program_ref1 = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new(cpi_program_ref1, cpi_accounts_ref1)
                .with_signer(signer_seeds), // <-- Подписываем своим PDA
            REWARD_AMOUNT,
        )?;

        // Перевод награды второму рефералу
        msg!(
            "Transferring {} tokens to referee 2: {}",
            REWARD_AMOUNT,
            ctx.accounts.referee2_token_account.key()
        );
        let cpi_accounts_ref2 = Transfer {
            from: ctx.accounts.referral_treasury_token_account.to_account_info(), // <-- Из казны рефералки
            to: ctx.accounts.referee2_token_account.to_account_info(),
            authority: ctx.accounts.referral_treasury_authority.to_account_info(), // <-- PDA рефералки
        };
        let cpi_program_ref2 = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new(cpi_program_ref2, cpi_accounts_ref2)
                .with_signer(signer_seeds), // <-- Подписываем своим PDA
            REWARD_AMOUNT,
        )?;

        msg!("Referral processed successfully.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ProcessReferral<'info> {
    pub mint: Account<'info, Mint>, // Mint токена для награды

    /// CHECK: PDA казны, принадлежащий ЭТОЙ программе.
    /// Сиды: [b"referral_treasury", mint.key().as_ref()]
    /// Бамп проверяется Anchor относительно ID этой программы.
    #[account(
        seeds = [b"referral_treasury".as_ref(), mint.key().as_ref()], // <-- Новый сид
        bump, // Anchor найдет бамп для ЭТОЙ программы
        // seeds::program больше не нужен
    )]
    pub referral_treasury_authority: AccountInfo<'info>,

    #[account(
        mut, // Баланс казны будет уменьшаться
        associated_token::mint = mint,
        associated_token::authority = referral_treasury_authority, // Казна принадлежит PDA ЭТОЙ программы
    )]
    pub referral_treasury_token_account: Account<'info, TokenAccount>,

    // Аккаунт, инициирующий реферальную транзакцию
    #[account(mut)]
    pub referrer: Signer<'info>,

    // Токен-аккаунт первого реферала
    #[account(
        mut,
        constraint = referee1_token_account.mint == mint.key() @ ErrorCode::IncorrectMint,
    )]
    pub referee1_token_account: Account<'info, TokenAccount>,

    // Токен-аккаунт второго реферала
     #[account(
        mut,
        constraint = referee2_token_account.mint == mint.key() @ ErrorCode::IncorrectMint,
    )]
    pub referee2_token_account: Account<'info, TokenAccount>,

    // --- Необходимые программы ---
    pub token_program: Program<'info, Token>,
    // SystemProgram может понадобиться, если бы мы создавали аккаунты рефералов
    // pub system_program: Program<'info, System>,
    // AssociatedTokenProgram не нужен, т.к. мы не создаем ATA здесь
    // pub associated_token_program: Program<'info, AssociatedToken>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Referral already processed for these participants and mint.")]
    AlreadyProcessed,
    #[msg("Insufficient balance in referral treasury.")] // Обновил сообщение
    InsufficientTreasuryBalance,
    #[msg("Calculation overflow.")]
    CalculationOverflow,
    #[msg("Referrer cannot be the same as the referee.")]
    ReferrerCannotBeReferee,
    #[msg("Incorrect mint for referee token account.")] // Обновил сообщение
    IncorrectMint,
}
