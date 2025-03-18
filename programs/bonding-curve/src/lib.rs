use anchor_lang::prelude::*;

declare_id!("HgiiaxwngpLK7jS3hC5EYXz8JkgSpMcA1xdaRc7tCqTL");

pub mod constants;
pub mod errors;
pub mod state;
pub mod contexts;
pub mod instructions;
pub mod math;

use contexts::*;

#[program]
pub mod bonding_curve {
    use super::*;

    /// Инициализация бондинговой кривой для нового мемкоина
    pub fn initialize_bonding_curve(
        ctx: Context<InitializeBondingCurve>,
        coin_mint: Pubkey,
        initial_price: u64,
        power_opt: Option<u8>,
        fee_percent_opt: Option<u16>,
    ) -> Result<()> {
        instructions::initialize::initialize_bonding_curve(
            ctx, 
            coin_mint, 
            initial_price, 
            power_opt, 
            fee_percent_opt
        )
    }

    /// Покупка токенов через бондинговую кривую, оплата в N-Dollar
    pub fn buy_token(
        ctx: Context<TradeToken>,
        ndollar_amount: u64,
    ) -> Result<()> {
        instructions::trade::buy_token(ctx, ndollar_amount)
    }

    /// Продажа токенов через бондинговую кривую, получение N-Dollar
    pub fn sell_token(
        ctx: Context<TradeToken>,
        token_amount: u64,
    ) -> Result<()> {
        instructions::trade::sell_token(ctx, token_amount)
    }

    /// Рассчитывает текущую цену токена и отправляет результат в логи
    pub fn calculate_price(ctx: Context<CalculatePrice>) -> Result<()> {
        instructions::price::calculate_price(ctx)
    }

    /// Симулирует покупку токенов, вычисляя примерное количество получаемых токенов
    pub fn simulate_buy(
        ctx: Context<CalculatePrice>, 
        ndollar_amount: u64
    ) -> Result<()> {
        instructions::price::simulate_buy(ctx, ndollar_amount)
    }
}