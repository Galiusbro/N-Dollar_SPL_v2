use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;
pub mod constants;
pub mod utils;

use instructions::*;

declare_id!("9FtGUvKzUZ9MFKYWnj4hsNLomFVopjGtznp52pMEWY8D");

#[program]
pub mod genesis {
    use super::*;

    /// Создание нового мемкоина
    pub fn create_coin(
        ctx: Context<CreateCoin>,
        name: String,
        symbol: String,
        uri: String,
        ndollar_payment: u64,
        admin: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_coin::handler(ctx, name, symbol, uri, ndollar_payment, admin)
    }

    /// Опция для основателя купить дополнительные токены (до 10% от общего предложения)
    pub fn purchase_founder_option(
        ctx: Context<PurchaseFounderOption>,
        amount: u64,
        ndollar_payment: u64,
    ) -> Result<()> {
        instructions::purchase_founder_option::handler(ctx, amount, ndollar_payment)
    }

    /// Генерация уникальной реферальной ссылки
    pub fn generate_referral_link(
        ctx: Context<GenerateReferralLink>,
    ) -> Result<()> {
        instructions::generate_referral_link::handler(ctx)
    }

    /// Передача административных прав другому пользователю
    pub fn transfer_admin_rights(
        ctx: Context<TransferAdminRights>,
        new_admin: Pubkey,
    ) -> Result<()> {
        instructions::transfer_admin_rights::handler(ctx, new_admin)
    }
}
