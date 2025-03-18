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

/// Структура для покупки/продажи токенов через бондинговую кривую
#[derive(Accounts)]
pub struct TradeToken<'info> {
    /// Покупатель/продавец токенов
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// PDA бондинговой кривой
    #[account(mut)]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    /// Mint токена монеты
    #[account(
        mut,
        constraint = coin_mint.key() == bonding_curve.coin_mint
    )]
    pub coin_mint: Account<'info, Mint>,
    
    /// Mint токена N-Dollar
    #[account(
        constraint = ndollar_mint.key() == bonding_curve.ndollar_mint
    )]
    pub ndollar_mint: Account<'info, Mint>,
    
    /// Токен аккаунт покупателя для монеты
    #[account(
        mut,
        constraint = buyer_coin_account.mint == coin_mint.key(),
        constraint = buyer_coin_account.owner == buyer.key()
    )]
    pub buyer_coin_account: Account<'info, TokenAccount>,
    
    /// Токен аккаунт покупателя для N-Dollar
    #[account(
        mut,
        constraint = buyer_ndollar_account.mint == ndollar_mint.key(),
        constraint = buyer_ndollar_account.owner == buyer.key()
    )]
    pub buyer_ndollar_account: Account<'info, TokenAccount>,
    
    /// Токен аккаунт для хранения ликвидности
    #[account(
        mut,
        constraint = liquidity_pool.mint == ndollar_mint.key(),
        constraint = liquidity_pool.owner == bonding_curve.key(),
        constraint = liquidity_pool.key() == bonding_curve.liquidity_pool
    )]
    pub liquidity_pool: Account<'info, TokenAccount>,
    
    /// Аккаунт конфигурации из admin_control
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова к программе admin_control
    pub admin_config: AccountInfo<'info>,
    
    /// Программа admin_control
    /// CHECK: Проверяем ID программы admin_control внутри инструкции
    pub admin_control_program: AccountInfo<'info>,
    
    /// Программа токенов
    pub token_program: Program<'info, Token>,
}

/// Структура для расчета текущей цены и моделирования торговых операций
#[derive(Accounts)]
pub struct CalculatePrice<'info> {
    /// PDA бондинговой кривой
    pub bonding_curve: Account<'info, BondingCurve>,
    
    /// Mint токена монеты
    #[account(constraint = coin_mint.key() == bonding_curve.coin_mint)]
    pub coin_mint: Account<'info, Mint>,
}

/// Структура для закрытия бондинговой кривой
#[derive(Accounts)]
pub struct CloseBondingCurve<'info> {
    /// Создатель бондинговой кривой, получит рент
    #[account(mut)]
    pub creator: Signer<'info>,
    
    /// PDA бондинговой кривой
    #[account(
        mut,
        close = creator,
        constraint = bonding_curve.creator == creator.key(),
        constraint = bonding_curve.total_supply_in_curve == 0,
        constraint = bonding_curve.reserve_balance == 0,
        seeds = [b"bonding_curve", bonding_curve.coin_mint.as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    /// Аккаунт конфигурации из admin_control
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова к программе admin_control
    pub admin_config: AccountInfo<'info>,
    
    /// Программа admin_control
    /// CHECK: Проверяем ID программы admin_control внутри инструкции
    pub admin_control_program: AccountInfo<'info>,
} 