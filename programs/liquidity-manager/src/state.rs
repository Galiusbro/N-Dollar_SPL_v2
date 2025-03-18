use anchor_lang::prelude::*;

#[account]
pub struct LiquidityManager {
    pub authority: Pubkey,
    pub n_dollar_mint: Pubkey,
    pub total_liquidity: u64,
    pub total_users: u64,
    pub current_price: u64,  // Цена в N-Dollar за 1 SOL
    pub last_update_time: i64,
    pub last_large_swap_time: i64, // Время последнего крупного свопа
    pub last_large_swap_amount: u64, // Размер последнего крупного свопа
    pub last_large_swap_direction: bool, // true для SOL->N-Dollar, false для N-Dollar->SOL
    pub price_impact_cooldown: u64, // Кулдаун между крупными свопами, меняющими цену
    pub bump: u8,
}

impl LiquidityManager {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 1;
}