use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::AdminConfig;
use crate::ErrorCode;

/// Контекст для инициализации Admin Control
#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + AdminConfig::SPACE
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    pub system_program: Program<'info, System>,
}

/// Контекст для инициализации N-Dollar
#[derive(Accounts)]
pub struct InitializeNDollar<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
    
    /// Mint аккаунт токена N-Dollar
    pub ndollar_mint: Account<'info, Mint>,
} 