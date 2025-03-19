use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};
use crate::contexts::MintSupply;
use crate::errors::NDollarError;
use crate::constants::WEEK_IN_SECONDS;
use crate::instructions::utils::{verify_admin_control_authorization, verify_time_manipulation};

/// Минтинг токенов согласно расписанию
pub fn mint_supply(ctx: Context<MintSupply>, amount: u64) -> Result<()> {
    // Проверка авторизации через admin_control, если admin_config передан
    verify_admin_control_authorization(
        &ctx.accounts.admin_config,
        &ctx.accounts.admin_control_program
    )?;
    
    let admin_account = &mut ctx.accounts.admin_account;
    let current_time = Clock::get()?.unix_timestamp;
    let current_slot = Clock::get()?.slot;
    
    // Проверка прошла ли неделя с последнего минта
    let time_since_last_mint = current_time - admin_account.last_mint_time;
    require!(time_since_last_mint >= WEEK_IN_SECONDS, NDollarError::TooEarlyToMint);
    
    // Защита от атак на время
    verify_time_manipulation(admin_account, current_time, current_slot)?;
    
    // Только авторизованный пользователь может минтить
    require!(
        admin_account.authority == ctx.accounts.authority.key(),
        NDollarError::UnauthorizedAccess
    );

    // Минтим токены
    let seeds = &[
        b"admin_account".as_ref(),
        &admin_account.mint.to_bytes(),
        &[admin_account.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: admin_account.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::mint_to(cpi_ctx, amount)?;
    
    // Обновляем информацию о последнем минте
    admin_account.last_mint_time = current_time;
    admin_account.last_block_time = current_time;
    admin_account.last_block_height = current_slot;
    admin_account.total_supply += amount;
    
    msg!("Минт выполнен успешно, добавлено: {}", amount);
    Ok(())
}