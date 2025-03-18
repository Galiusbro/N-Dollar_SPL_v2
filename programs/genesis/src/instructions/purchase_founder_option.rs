use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};

use crate::errors::GenesisError;
use crate::instructions::contexts::PurchaseFounderOption;
use crate::constants::*;
use crate::utils::*;

pub fn handler(
    ctx: Context<PurchaseFounderOption>,
    amount: u64,
    ndollar_payment: u64,
) -> Result<()> {
    let coin_data = &ctx.accounts.coin_data;
    let admin = &ctx.accounts.admin;
    let creator = ctx.accounts.creator.key();
    
    // Проверка, что вызывающий является администратором монеты
    require!(
        coin_data.admin == admin.key(),
        GenesisError::NotCoinAdmin
    );
    
    // Проверка, что токены покупаются для создателя
    require!(
        coin_data.creator == creator,
        GenesisError::NotCoinCreator
    );
    
    // Проверка, что запрашиваемая сумма не превышает 10% от максимального предложения
    let max_founder_allocation = MAX_SUPPLY / INITIAL_SUPPLY_PERCENTAGE;
    let current_allocation = coin_data.total_supply;
    
    require!(
        current_allocation + amount <= max_founder_allocation,
        GenesisError::ExceedsFounderAllocation
    );
    
    // Переводим N-Dollar в качестве оплаты
    let transfer_instruction = anchor_spl::token::Transfer {
        from: ctx.accounts.ndollar_token_account.to_account_info(),
        to: ctx.accounts.fees_account.to_account_info(),
        authority: admin.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );
    
    token::transfer(cpi_ctx, ndollar_payment)?;
    
    // Минтим дополнительные токены создателю
    let mint = ctx.accounts.mint.key();
    let bump = &coin_data.bump;
    let seeds = get_coin_data_seeds(&mint, bump);
    let signer = &[&seeds[..]];
    
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.creator_token_account.to_account_info(),
        authority: coin_data.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::mint_to(cpi_ctx, amount)?;
    
    // Обновляем информацию о токенах в coin_data
    let coin_data = &mut ctx.accounts.coin_data;
    coin_data.total_supply += amount;
    
    msg!("Основатель приобрел дополнительные токены: {}", amount);
    Ok(())
}