use anchor_lang::prelude::*;

#[account]
pub struct ReferralSystem {
    pub authority: Pubkey,
    pub coin_mint: Pubkey,
    pub creation_time: i64,
    pub total_referrals: u64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl ReferralSystem {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 1;
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

#[account]
pub struct UserData {
    pub user: Pubkey,
    pub referrer: Pubkey,
    pub coin_mint: Pubkey,
    pub registration_time: i64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl UserData {
    pub const SPACE: usize = 32 + 32 + 32 + 8 + 8 + 1;
}
