use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::BondingCurve;

/// Структура для расчета текущей цены и моделирования торговых операций
#[derive(Accounts)]
pub struct CalculatePrice<'info> {
    /// PDA бондинговой кривой
    pub bonding_curve: Account<'info, BondingCurve>,
    
    /// Mint токена монеты
    #[account(constraint = coin_mint.key() == bonding_curve.coin_mint)]
    pub coin_mint: Account<'info, Mint>,
}