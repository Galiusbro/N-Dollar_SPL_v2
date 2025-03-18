use anchor_lang::prelude::*;

#[account]
pub struct CoinData {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub creation_time: i64,
    pub total_supply: u64,
    pub referral_link_active: bool,
    pub admin: Pubkey,
    pub bump: u8,
}

impl CoinData {
    // Размер для хранения строковых полей (название и символ) может быть динамическим
    // но мы выделим немного больше чтобы быть уверенными
    pub const SPACE: usize = 32 + 32 + 50 + 10 + 8 + 8 + 1 + 32 + 1;
}

#[account]
pub struct ReferralData {
    pub coin_mint: Pubkey,
    pub creator: Pubkey,
    pub creation_time: i64,
    pub referred_users: u64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl ReferralData {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 1;
}