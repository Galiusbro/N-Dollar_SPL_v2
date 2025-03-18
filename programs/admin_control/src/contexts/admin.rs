use anchor_lang::prelude::*;
use crate::state::AdminConfig;
use crate::ErrorCode;

/// Контекст для обновления комиссий
#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Контекст для авторизации программы
#[derive(Accounts)]
pub struct AuthorizeProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Контекст для отзыва авторизации программы
#[derive(Accounts)]
pub struct RevokeProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
}

/// Контекст для обновления версии AdminConfig
#[derive(Accounts)]
pub struct UpgradeAdminConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"admin_config".as_ref(), authority.key().as_ref()],
        bump = admin_config.bump,
        constraint = admin_config.authority == authority.key() @ ErrorCode::UnauthorizedAccess
    )]
    pub admin_config: Account<'info, AdminConfig>,
} 