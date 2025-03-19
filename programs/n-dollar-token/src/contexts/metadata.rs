use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::metadata::Metadata;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        constraint = mint.key() == admin_account.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump = admin_account.bump,
        constraint = admin_account.authority == authority.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
    /// CHECK: Аккаунт метаданных, который будет обновлен через CPI
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
}
