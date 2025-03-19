use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use anchor_spl::metadata::Metadata;
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
    
    /// CHECK: Аккаунт метаданных, который будет инициализирован через CPI
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + AdminAccount::SPACE
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
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
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}
