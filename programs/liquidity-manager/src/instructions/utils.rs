use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use crate::errors::LiquidityError;
use crate::constants::*;

/// Проверяет, не превышает ли влияние на цену допустимые пределы
/// Возвращает процентное влияние на пул ликвидности
pub fn check_price_impact(
    amount: u64,
    pool_balance: u64,
) -> Result<u64> {
    let price_impact_percentage = (amount as u128)
        .checked_mul(100)
        .and_then(|v| v.checked_div(pool_balance as u128))
        .unwrap_or(0) as u64;
        
    Ok(price_impact_percentage)
}

/// Проверяет и обновляет данные о крупном свопе
pub fn update_large_swap_data<'a>(
    liquidity_manager: &mut Account<'a, crate::state::LiquidityManager>,
    price_impact_percentage: u64,
    amount: u64,
    is_buy_direction: bool,
    current_time: i64,
) -> Result<bool> {
    let mut is_large_swap = false;
    
    // Проверка на крупную транзакцию и возможное влияние на цену
    if price_impact_percentage > PRICE_IMPACT_THRESHOLD_PERCENTAGE {
        is_large_swap = true;
        
        // Обновляем данные о последнем крупном свопе
        liquidity_manager.last_large_swap_time = current_time;
        liquidity_manager.last_large_swap_amount = amount;
        liquidity_manager.last_large_swap_direction = is_buy_direction;
        liquidity_manager.last_update_time = current_time;
    }
    
    Ok(is_large_swap)
}

/// Передает SOL от отправителя получателю
pub fn transfer_sol<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
    system_program: &AccountInfo<'a>,
    is_from_pda: bool,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let sol_transfer_instruction = system_instruction::transfer(
        &from.key(),
        &to.key(),
        amount,
    );
    
    if is_from_pda {
        invoke_signed(
            &sol_transfer_instruction,
            &[
                from.clone(),
                to.clone(),
                system_program.clone(),
            ],
            seeds.unwrap(),
        )?;
    } else {
        invoke(
            &sol_transfer_instruction,
            &[
                from.clone(),
                to.clone(),
                system_program.clone(),
            ],
        )?;
    }
    
    Ok(())
}

/// Рассчитывает процентное изменение цены
pub fn calculate_price_change_percentage(
    amount: u64,
) -> Result<u64> {
    // Рассчитываем процентное изменение: 0.1% за каждый SOL (10 базисных пунктов)
    let percentage = amount
        .checked_mul(10)
        .and_then(|v| v.checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL))
        .ok_or(LiquidityError::ArithmeticError)?;
        
    Ok(percentage)
}