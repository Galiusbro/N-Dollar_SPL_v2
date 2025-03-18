use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer, Burn};
use crate::constants::bonding_curve::*;
use crate::contexts::TradeToken;
use crate::errors::BondingCurveError;
use crate::instructions::utils::verify_program_auth;
use crate::math;

/// Покупка токенов через бондинговую кривую, оплата в N-Dollar
pub fn buy_token(
    ctx: Context<TradeToken>,
    ndollar_amount: u64,
) -> Result<()> {
    // Проверка на нулевое количество
    require!(ndollar_amount > 0, BondingCurveError::ZeroAmount);
    
    // Проверка на слишком большую сумму
    require!(
        ndollar_amount <= MAX_NDOLLAR_TRANSACTION,
        BondingCurveError::TransactionTooLarge
    );
    
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    
    // Проверяем авторизацию программы через admin_control
    let admin_config_info = ctx.accounts.admin_config.to_account_info();
    let admin_control_program = ctx.accounts.admin_control_program.to_account_info();
    verify_program_auth(&admin_config_info, &admin_control_program)?;
    
    // Рассчитываем комиссию
    let fee_amount = math::calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
    let effective_amount = ndollar_amount.checked_sub(fee_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
    
    // Рассчитываем количество токенов к получению
    let token_amount = math::calculate_buy_amount(
        bonding_curve.total_supply_in_curve,
        bonding_curve.reserve_balance,
        effective_amount,
        bonding_curve.power,
        bonding_curve.initial_price,
    )?;
    
    // Проверяем, что рассчитанное количество токенов корректно
    require!(token_amount > 0, BondingCurveError::ZeroOutput);
    
    msg!("Покупка: {} токенов за {} NDollar", token_amount, ndollar_amount);
    msg!("Комиссия: {} NDollar", fee_amount);
    
    // Проверяем, что у пользователя достаточно N-Dollar для покупки
    require!(
        ctx.accounts.buyer_ndollar_account.amount >= ndollar_amount,
        BondingCurveError::InsufficientFunds
    );
    
    // Переводим N-Dollar в пул ликвидности
    let transfer_instruction = Transfer {
        from: ctx.accounts.buyer_ndollar_account.to_account_info(),
        to: ctx.accounts.liquidity_pool.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );
    
    token::transfer(cpi_ctx, ndollar_amount)?;
    
    // Минтим новые токены покупателю
    let seeds = &[
        b"bonding_curve".as_ref(),
        &bonding_curve.coin_mint.to_bytes(),
        &[bonding_curve.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = MintTo {
        mint: ctx.accounts.coin_mint.to_account_info(),
        to: ctx.accounts.buyer_coin_account.to_account_info(),
        authority: bonding_curve.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::mint_to(cpi_ctx, token_amount)?;
    
    // Обновляем состояние бондинговой кривой
    bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
        .checked_add(token_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    bonding_curve.reserve_balance = bonding_curve.reserve_balance
        .checked_add(ndollar_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    // Обновляем constant_product
    bonding_curve.constant_product = math::calculate_constant_product(
        bonding_curve.total_supply_in_curve,
        bonding_curve.reserve_balance,
        bonding_curve.power,
    )?;
    
    bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
    
    msg!("Токены успешно куплены. Новый supply: {}, резерв: {}", 
         bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
    Ok(())
}

/// Продажа токенов через бондинговую кривую, получение N-Dollar
pub fn sell_token(
    ctx: Context<TradeToken>,
    token_amount: u64,
) -> Result<()> {
    // Проверка на нулевое количество
    require!(token_amount > 0, BondingCurveError::ZeroAmount);
    
    // Проверка на слишком большое количество токенов
    require!(
        token_amount <= MAX_TOKEN_TRANSACTION,
        BondingCurveError::TransactionTooLarge
    );
    
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    
    // Проверяем авторизацию программы через admin_control
    let admin_config_info = ctx.accounts.admin_config.to_account_info();
    let admin_control_program = ctx.accounts.admin_control_program.to_account_info();
    verify_program_auth(&admin_config_info, &admin_control_program)?;
    
    // Проверка наличия достаточного количества токенов у продавца
    require!(
        ctx.accounts.buyer_coin_account.amount >= token_amount,
        BondingCurveError::InsufficientTokens
    );
    
    // Проверка наличия достаточной ликвидности в пуле
    require!(
        bonding_curve.reserve_balance > 0,
        BondingCurveError::InsufficientLiquidity
    );
    
    // Рассчитываем количество N-Dollar к получению и комиссию
    let (ndollar_amount, fee_amount) = math::calculate_sell_amount(
        bonding_curve.total_supply_in_curve,
        bonding_curve.reserve_balance,
        token_amount,
        bonding_curve.power,
        bonding_curve.fee_percent,
    )?;
    
    // Проверяем, что рассчитанные суммы корректны
    require!(ndollar_amount > 0, BondingCurveError::ZeroOutput);
    
    msg!("Продажа: {} токенов за {} NDollar", token_amount, ndollar_amount);
    msg!("Комиссия: {} NDollar", fee_amount);
    
    // Проверяем, что в пуле ликвидности достаточно средств
    require!(
        ctx.accounts.liquidity_pool.amount >= ndollar_amount,
        BondingCurveError::InsufficientLiquidity
    );
    
    // Сжигаем токены
    let seeds = &[
        b"bonding_curve".as_ref(),
        &bonding_curve.coin_mint.to_bytes(),
        &[bonding_curve.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Burn {
        mint: ctx.accounts.coin_mint.to_account_info(),
        from: ctx.accounts.buyer_coin_account.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::burn(cpi_ctx, token_amount)?;
    
    // Переводим N-Dollar продавцу
    let cpi_accounts = Transfer {
        from: ctx.accounts.liquidity_pool.to_account_info(),
        to: ctx.accounts.buyer_ndollar_account.to_account_info(),
        authority: bonding_curve.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, ndollar_amount)?;
    
    // Обновляем состояние бондинговой кривой
    bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
        .checked_sub(token_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    bonding_curve.reserve_balance = bonding_curve.reserve_balance
        .checked_sub(ndollar_amount + fee_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    // Обновляем constant_product
    bonding_curve.constant_product = math::calculate_constant_product(
        bonding_curve.total_supply_in_curve,
        bonding_curve.reserve_balance,
        bonding_curve.power,
    )?;
    
    bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
    
    msg!("Токены успешно проданы. Новый supply: {}, резерв: {}", 
         bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
    Ok(())
}