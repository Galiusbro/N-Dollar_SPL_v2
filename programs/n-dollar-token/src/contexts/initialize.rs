use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
// use anchor_spl::metadata::Metadata;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeNDollar<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = admin_account,
    )]
    pub mint: Account<'info, Mint>,
    
    /* Временно отключено для тестирования
    /// CHECK: Аккаунт метаданных, который будет инициализирован через CPI
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    */
    
    #[account(
        init,
        payer = authority,
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + AdminAccount::SPACE
    )]
    pub admin_account: Account<'info, AdminAccount>,

    /// Токен-аккаунт для администратора, куда будет выполнен минт резервной части токенов
    #[account(
        init_if_needed,
        payer = authority,
        token::mint = mint,
        token::authority = authority,
    )]
    pub admin_token_account: Account<'info, TokenAccount>,
    
    /// Опциональный токен-аккаунт пула ликвидности, куда будет направлена основная часть минта
    /// Если не предоставлен, все токены будут минтированы на admin_token_account
    #[account(
        mut,
    )]
    pub liquidity_pool_account: Option<Account<'info, TokenAccount>>,
    
    /// Опциональный admin_config аккаунт из программы admin_control
    /// Используется для регистрации минта N-Dollar в admin_control
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова
    #[account(mut)]
    pub admin_config: Option<AccountInfo<'info>>,
    
    /// Опциональная программа admin_control для CPI вызовов
    /// CHECK: ID программы admin_control
    pub admin_control_program: Option<AccountInfo<'info>>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /* Временно отключено для тестирования
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    */
    pub rent: Sysvar<'info, Rent>,
}
