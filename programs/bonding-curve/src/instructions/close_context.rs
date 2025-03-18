use anchor_lang::prelude::*;
use crate::state::BondingCurve;

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