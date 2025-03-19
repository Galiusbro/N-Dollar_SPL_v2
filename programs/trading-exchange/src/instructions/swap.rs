use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::contexts::{SwapTokens, SwapNDollarToSol, SwapSolToNDollar};
use crate::errors::TradingError;
use crate::instructions::utils::{verify_admin_control_authorization, get_fee_percentage, create_cpi_instruction};
use crate::math::{calculate_token_ndollar_value, calculate_ndollar_token_amount};
use crate::constants::*;

/// Свап между различными токенами по их курсу относительно N-Dollar
pub fn swap_tokens(
    ctx: Context<SwapTokens>,
    amount_in: u64,
) -> Result<()> {
    // Проверка авторизации через admin_control
    verify_admin_control_authorization(
        &ctx.accounts.admin_config,
        &ctx.accounts.admin_control_program
    )?;
    
    // Получаем информацию о токенах
    let from_mint = ctx.accounts.from_mint.key();
    let to_mint = ctx.accounts.to_mint.key();
    let user = &ctx.accounts.user;
    
    // Проверка, что у пользователя достаточно токенов для обмена
    require!(
        ctx.accounts.user_from_account.amount >= amount_in,
        TradingError::InsufficientTokenBalance
    );
    
    // Получаем бондинговые кривые для токенов
    let from_bonding_curve = &ctx.accounts.from_bonding_curve;
    let to_bonding_curve = &ctx.accounts.to_bonding_curve;
    
    // Проверяем, что бондинговые кривые соответствуют токенам
    require!(
        from_bonding_curve.coin_mint == from_mint,
        TradingError::InvalidBondingCurve
    );
    
    require!(
        to_bonding_curve.coin_mint == to_mint,
        TradingError::InvalidBondingCurve
    );
    
    // Проверяем, что оба токена используют N-Dollar как базовую валюту
    require!(
        from_bonding_curve.ndollar_mint == to_bonding_curve.ndollar_mint,
        TradingError::InvalidTokenPair
    );
    
    // 1. Расчет стоимости входящих токенов в N-Dollar
    let ndollar_amount = calculate_token_ndollar_value(
        amount_in, 
        from_bonding_curve.total_supply_in_curve,
        from_bonding_curve.reserve_balance,
        from_bonding_curve.power
    )?;
    
    // 2. Определяем комиссию за свап
    let fee_percentage = get_fee_percentage(
        &ctx.accounts.admin_config,
        &ctx.accounts.admin_control_program
    )?;
    
    let fee_amount = ndollar_amount.checked_mul(fee_percentage)
        .and_then(|v| v.checked_div(100))
        .ok_or(TradingError::ArithmeticError)?;
    
    // 3. Вычисляем чистую сумму N-Dollar после комиссии
    let net_ndollar_amount = ndollar_amount.checked_sub(fee_amount)
        .ok_or(TradingError::ArithmeticError)?;
    
    // 4. Рассчитываем, сколько токенов назначения соответствует этой сумме N-Dollar
    let amount_out = calculate_ndollar_token_amount(
        net_ndollar_amount,
        to_bonding_curve.total_supply_in_curve,
        to_bonding_curve.reserve_balance,
        to_bonding_curve.power,
        to_bonding_curve.initial_price
    )?;
    
    // Проверка, что в пуле ликвидности достаточно токенов для обмена
    require!(
        ctx.accounts.liquidity_to_account.amount >= amount_out,
        TradingError::InsufficientLiquidity
    );
    
    msg!("Обмен: {} токенов по цене {} N-Dollar => {} токенов", 
         amount_in, ndollar_amount, amount_out);
    msg!("Комиссия: {} N-Dollar", fee_amount);
    
    // Перевод токенов от пользователя в пул ликвидности
    let transfer_from_instruction = Transfer {
        from: ctx.accounts.user_from_account.to_account_info(),
        to: ctx.accounts.liquidity_from_account.to_account_info(),
        authority: user.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_from_instruction,
    );
    
    token::transfer(cpi_ctx, amount_in)?;
    
    // Перевод токенов из пула ликвидности пользователю
    let exchange_data = &ctx.accounts.exchange_data;
    let seeds = &[
        EXCHANGE_DATA_SEED,
        &exchange_data.authority.to_bytes(),
        &[exchange_data.bump],
    ];
    let signer = &[&seeds[..]];
    
    let transfer_to_instruction = Transfer {
        from: ctx.accounts.liquidity_to_account.to_account_info(),
        to: ctx.accounts.user_to_account.to_account_info(),
        authority: exchange_data.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_to_instruction,
        signer,
    );
    
    token::transfer(cpi_ctx, amount_out)?;
    
    // Обновляем статистику обмена
    let exchange_data = &mut ctx.accounts.exchange_data;
    exchange_data.total_volume_traded += amount_in;
    exchange_data.total_fees_collected += fee_amount;
    
    msg!("Своп токенов выполнен успешно");
    Ok(())
}

/// Обмен N-Dollar на SOL через Liquidity Manager
pub fn swap_ndollar_to_sol(
    ctx: Context<SwapNDollarToSol>,
    ndollar_amount: u64,
) -> Result<()> {
    // Проверяем достаточность баланса
    require!(
        ctx.accounts.user_ndollar_account.amount >= ndollar_amount,
        TradingError::InsufficientBalance
    );
    
    // Получаем данные из контекста
    let liquidity_manager_program_id = ctx.accounts.liquidity_manager_program.key();
    
    // Создаем список аккаунтов для CPI
    let accounts = vec![
        AccountMeta::new(ctx.accounts.user.key(), true),  // user (signer)
        AccountMeta::new(ctx.accounts.liquidity_manager.key(), false),  // liquidity_manager
        AccountMeta::new(ctx.accounts.user_ndollar_account.key(), false),  // user_ndollar_account
        AccountMeta::new(ctx.accounts.pool_sol_account.key(), false),  // pool_sol_account
        AccountMeta::new(ctx.accounts.pool_ndollar_account.key(), false),  // pool_ndollar_account
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),  // token_program
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),  // system_program
    ];
    
    // Создаем данные инструкции
    let mut data = Vec::new();
    data.extend_from_slice(&ndollar_amount.to_le_bytes());
    
    // Создаем инструкцию
    let ix = create_cpi_instruction(
        liquidity_manager_program_id,
        accounts,
        SWAP_NDOLLAR_TO_SOL_DISCRIMINATOR,
        Some(data),
    );
    
    // Собираем все аккаунты для вызова
    let account_infos = vec![
        ctx.accounts.user.to_account_info(),
        ctx.accounts.liquidity_manager.to_account_info(),
        ctx.accounts.user_ndollar_account.to_account_info(),
        ctx.accounts.pool_sol_account.to_account_info(),
        ctx.accounts.pool_ndollar_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];
    
    // Выполняем инструкцию
    anchor_lang::solana_program::program::invoke(
        &ix,
        account_infos.as_slice(),
    )?;
    
    msg!("Обмен N-Dollar на SOL выполнен успешно");
    Ok(())
}

/// Обмен SOL на N-Dollar через Liquidity Manager
pub fn swap_sol_to_ndollar(
    ctx: Context<SwapSolToNDollar>,
    sol_amount: u64,
) -> Result<()> {
    // Проверяем достаточность SOL
    require!(
        ctx.accounts.user.lamports() >= sol_amount + MIN_SOL_FOR_FEES,
        TradingError::InsufficientBalance
    );
    
    // Получаем данные из контекста
    let liquidity_manager_program_id = ctx.accounts.liquidity_manager_program.key();
    
    // Создаем список аккаунтов для CPI
    let accounts = vec![
        AccountMeta::new(ctx.accounts.user.key(), true),  // user (signer)
        AccountMeta::new(ctx.accounts.liquidity_manager.key(), false),  // liquidity_manager
        AccountMeta::new(ctx.accounts.user_ndollar_account.key(), false),  // user_ndollar_account
        AccountMeta::new(ctx.accounts.pool_sol_account.key(), false),  // pool_sol_account
        AccountMeta::new(ctx.accounts.pool_ndollar_account.key(), false),  // pool_ndollar_account
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),  // token_program
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),  // system_program
    ];
    
    // Создаем данные инструкции
    let mut data = Vec::new();
    data.extend_from_slice(&sol_amount.to_le_bytes());
    
    // Создаем инструкцию
    let ix = create_cpi_instruction(
        liquidity_manager_program_id,
        accounts,
        SWAP_SOL_TO_NDOLLAR_DISCRIMINATOR,
        Some(data),
    );
    
    // Собираем все аккаунты для вызова
    let account_infos = vec![
        ctx.accounts.user.to_account_info(),
        ctx.accounts.liquidity_manager.to_account_info(),
        ctx.accounts.user_ndollar_account.to_account_info(),
        ctx.accounts.pool_sol_account.to_account_info(),
        ctx.accounts.pool_ndollar_account.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];
    
    // Выполняем инструкцию
    anchor_lang::solana_program::program::invoke(
        &ix,
        account_infos.as_slice(),
    )?;
    
    msg!("Обмен SOL на N-Dollar выполнен успешно");
    Ok(())
}
