use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::*;

#[derive(Accounts)]
pub struct CalculatePrice<'info> {
    #[account(
        seeds = [b"bonding_curve".as_ref(), coin_mint.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(
        constraint = coin_mint.key() == bonding_curve.coin_mint
    )]
    pub coin_mint: Account<'info, Mint>,
}