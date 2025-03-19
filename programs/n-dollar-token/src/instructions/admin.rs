use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, FreezeAccount, ThawAccount};
use crate::contexts::{AdminFunction, AdminFunctionWithMultisig};
use crate::errors::NDollarError;
use crate::instructions::utils::verify_multisig;

/// Сжигание токенов (административная функция)
pub fn burn_tokens(ctx: Context<AdminFunction>, amount: u64) -> Result<()> {
    let admin_account = &ctx.accounts.admin_account;
    
    // Только авторизованный пользователь может сжигать токены
    require!(
        admin_account.authority == ctx.accounts.authority.key(),
        NDollarError::UnauthorizedAccess
    );

    // Сжигаем токены
    let cpi_accounts = Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::burn(cpi_ctx, amount)?;
    
    msg!("Токены успешно сожжены, количество: {}", amount);
    Ok(())
}

/// Заморозка аккаунта (административная функция)
pub fn freeze_account(ctx: Context<AdminFunctionWithMultisig>) -> Result<()> {
    // Проверка мультиподписи
    verify_multisig(
        &ctx.accounts.admin_account,
        &ctx.accounts.authority,
        &ctx.accounts.additional_signer1,
        &ctx.accounts.additional_signer2
    )?;

    // Замораживаем аккаунт
    let cpi_accounts = FreezeAccount {
        account: ctx.accounts.token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::freeze_account(cpi_ctx)?;
    
    msg!("Аккаунт заморожен успешно");
    Ok(())
}

/// Разморозка аккаунта (административная функция)
pub fn thaw_account(ctx: Context<AdminFunctionWithMultisig>) -> Result<()> {
    // Проверка мультиподписи
    verify_multisig(
        &ctx.accounts.admin_account,
        &ctx.accounts.authority,
        &ctx.accounts.additional_signer1,
        &ctx.accounts.additional_signer2
    )?;

    // Размораживаем аккаунт
    let cpi_accounts = ThawAccount {
        account: ctx.accounts.token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::thaw_account(cpi_ctx)?;
    
    msg!("Аккаунт разморожен успешно");
    Ok(())
}
