use anchor_lang::prelude::*;

declare_id!("2MiMzfH55kTC9tNM8LQFLzC8GbduTMTe5SyTadBi33cL");

pub mod constants;
pub mod errors;
pub mod state;
pub mod contexts;
pub mod instructions;

use contexts::*;

#[program]
pub mod n_dollar_token {
    use super::*;

    /// Инициализирует токен N-Dollar
    pub fn initialize_n_dollar(
        ctx: Context<InitializeNDollar>,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
    ) -> Result<()> {
        instructions::initialize::initialize_n_dollar(ctx, name, symbol, uri, decimals)
    }

    /// Минтинг токенов согласно расписанию
    pub fn mint_supply(ctx: Context<MintSupply>, amount: u64) -> Result<()> {
        instructions::mint::mint_supply(ctx, amount)
    }

    /// Сжигание токенов (административная функция)
    pub fn burn_tokens(ctx: Context<AdminFunction>, amount: u64) -> Result<()> {
        instructions::admin::burn_tokens(ctx, amount)
    }

    /// Заморозка аккаунта (административная функция)
    pub fn freeze_account(ctx: Context<AdminFunctionWithMultisig>) -> Result<()> {
        instructions::admin::freeze_account(ctx)
    }

    /// Разморозка аккаунта (административная функция)
    pub fn thaw_account(ctx: Context<AdminFunctionWithMultisig>) -> Result<()> {
        instructions::admin::thaw_account(ctx)
    }

    /// Обновление метаданных токена (административная функция)
    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::metadata::update_metadata(ctx, name, symbol, uri)
    }

    /// Добавление авторизованного подписанта
    pub fn add_authorized_signer(ctx: Context<AdminFunction>, new_signer: Pubkey) -> Result<()> {
        instructions::auth::add_authorized_signer(ctx, new_signer)
    }

    /// Удаление авторизованного подписанта
    pub fn remove_authorized_signer(ctx: Context<AdminFunction>, signer_to_remove: Pubkey) -> Result<()> {
        instructions::auth::remove_authorized_signer(ctx, signer_to_remove)
    }

    /// Установка минимального количества подписантов
    pub fn set_min_required_signers(ctx: Context<AdminFunction>, min_signers: u8) -> Result<()> {
        instructions::auth::set_min_required_signers(ctx, min_signers)
    }
}
