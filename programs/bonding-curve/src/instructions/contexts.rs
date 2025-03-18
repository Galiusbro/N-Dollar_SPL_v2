use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::BondingCurve;

/// Структура для инициализации бондинговой кривой
#[derive(Accounts)]
pub struct InitializeBondingCurve<'info> {
    /// Создатель бондинговой кривой, оплачивает rent
    #[account(mut)]
    pub creator: Signer<'info>,
    
    /// PDA бондинговой кривой
    #[account(
        init,
        payer = creator,
        space = 8 + BondingCurve::SPACE,
        seeds = [b"bonding_curve", coin_mint.key().as_ref()],
        bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    /// Mint токена N-Dollar
    pub ndollar_mint: Account<'info, Mint>,
    
    /// Mint монеты, для которой создается бондинговая кривая
    /// Передается как аргумент, для возможности повторного использования
    pub coin_mint: Account<'info, Mint>,
    
    /// Токен аккаунт для хранения ликвидности
    #[account(
        constraint = liquidity_pool.mint == ndollar_mint.key(),
        constraint = liquidity_pool.owner == bonding_curve.key(),
    )]
    pub liquidity_pool: Account<'info, TokenAccount>,
    
    /// Аккаунт конфигурации из admin_control
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова к программе admin_control
    pub admin_config: AccountInfo<'info>,
    
    /// Программа admin_control
    /// CHECK: Проверяем ID программы admin_control внутри инструкции
    pub admin_control_program: AccountInfo<'info>,
    
    /// Системная программа
    pub system_program: Program<'info, System>,
}