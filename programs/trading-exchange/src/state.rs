use anchor_lang::prelude::*;

#[account]
pub struct ExchangeData {
    pub authority: Pubkey,
    pub total_volume_traded: u64,
    pub total_fees_collected: u64,
    pub last_update_time: i64,
    pub bump: u8,
}

impl ExchangeData {
    pub const SPACE: usize = 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct TradingExchange {
    pub authority: Pubkey,
    pub n_dollar_mint: Pubkey,
    pub bump: u8,
}

impl TradingExchange {
    pub const SPACE: usize = 32 + 32 + 1;
}

#[account]
pub struct ControlState {
    pub authority: Pubkey,
    pub is_active: bool,
    pub last_updated: i64,
    pub bump: u8,
}

impl ControlState {
    pub const SPACE: usize = 32 + 1 + 8 + 1;
}
