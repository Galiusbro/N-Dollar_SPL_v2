use anchor_lang::prelude::*;
use crate::contexts::CalculatePrice;
use crate::math;

/// Рассчитывает текущую цену токена и отправляет результат в логи
pub fn calculate_price(ctx: Context<CalculatePrice>) -> Result<()> {
    let bonding_curve = &ctx.accounts.bonding_curve;
    
    // Если supply = 0, используем начальную цену
    if bonding_curve.total_supply_in_curve == 0 {
        msg!("Текущая цена (начальная): {} NDollar", bonding_curve.initial_price);
        return Ok(());
    }
    
    // Рассчитываем текущую цену
    let current_price = math::get_current_price(
        bonding_curve.total_supply_in_curve,
        bonding_curve.reserve_balance,
        bonding_curve.power,
    )?;
    
    msg!("Текущая цена: {} NDollar", current_price);
    msg!("Общий supply: {}, резерв: {}", bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
    
    Ok(())
}

/// Симулирует покупку токенов, вычисляя примерное количество получаемых токенов
pub fn simulate_buy(
    ctx: Context<CalculatePrice>, 
    ndollar_amount: u64
) -> Result<()> {
    let bonding_curve = &ctx.accounts.bonding_curve;
    
    // Рассчитываем комиссию
    let fee_amount = math::calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
    let effective_amount = ndollar_amount - fee_amount;
    
    msg!("Сумма: {} NDollar, комиссия: {} NDollar", ndollar_amount, fee_amount);
    
    // Рассчитываем количество токенов
    let token_amount = math::calculate_buy_amount(
        bonding_curve.total_supply_in_curve,
        bonding_curve.reserve_balance,
        effective_amount,
        bonding_curve.power,
        bonding_curve.initial_price,
    )?;
    
    // Рассчитываем прирост цены
    let current_price_before = if bonding_curve.total_supply_in_curve == 0 {
        bonding_curve.initial_price
    } else {
        math::get_current_price(
            bonding_curve.total_supply_in_curve, 
            bonding_curve.reserve_balance, 
            bonding_curve.power
        )?
    };
    
    let new_total_supply = bonding_curve.total_supply_in_curve + token_amount;
    let new_reserve = bonding_curve.reserve_balance + ndollar_amount;
    
    let current_price_after = if new_total_supply == 0 {
        bonding_curve.initial_price
    } else {
        math::get_current_price(
            new_total_supply, 
            new_reserve, 
            bonding_curve.power
        )?
    };
    
    let price_increase_percent = if current_price_before == 0 {
        0
    } else {
        ((current_price_after as f64 - current_price_before as f64) / current_price_before as f64 * 100.0) as u64
    };
    
    msg!("Вы получите примерно {} токенов", token_amount);
    msg!("Текущая цена: {} NDollar", current_price_before);
    msg!("Новая цена после покупки: {} NDollar (+{}%)", 
         current_price_after, price_increase_percent);
    
    Ok(())
}