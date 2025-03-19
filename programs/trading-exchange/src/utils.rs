use anchor_lang::prelude::*;
use crate::errors::TradingError;
use crate::constants::*;

/// Рассчитывает стоимость токенов в N-Dollar на основе бондинговой кривой
pub fn calculate_token_ndollar_value(
    token_amount: u64,
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
) -> Result<u64> {
    // Проверка на нулевые значения
    if total_supply == 0 || reserve_balance == 0 {
        return Err(TradingError::InvalidParameters.into());
    }
    
    // Для малых сумм (до 0.1% от total_supply) используем линейную аппроксимацию
    if token_amount < total_supply / SMALL_TRADE_THRESHOLD as u64 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        let ndollar_amount = token_amount.checked_mul(current_price)
            .ok_or(TradingError::ArithmeticError)?;
            
        return Ok(ndollar_amount);
    }
    
    // Для более крупных сумм учитываем слиппедж (уменьшаем выход на 15% от линейной оценки)
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    let linear_estimate = token_amount.checked_mul(current_price)
        .ok_or(TradingError::ArithmeticError)?;
        
    let ndollar_amount = if token_amount > total_supply / LARGE_TRADE_THRESHOLD as u64 {
        // Для крупных продаж (более 10% от всех токенов) учитываем слиппедж
        linear_estimate.checked_mul(LARGE_SELL_SLIPPAGE_FACTOR as u64)
            .and_then(|v| v.checked_div(100))
            .ok_or(TradingError::ArithmeticError)?
    } else {
        linear_estimate
    };
    
    Ok(ndollar_amount)
}

/// Рассчитывает количество токенов, соответствующее сумме N-Dollar
pub fn calculate_ndollar_token_amount(
    ndollar_amount: u64,
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
    initial_price: u64,
) -> Result<u64> {
    // Если это первая покупка или резерв пуст, используем начальную цену
    if total_supply == 0 || reserve_balance == 0 {
        if initial_price == 0 {
            return Err(TradingError::InvalidParameters.into());
        }
        
        let amount = ndollar_amount.checked_div(initial_price)
            .ok_or(TradingError::ArithmeticError)?;
            
        if amount < 1 {
            return Err(TradingError::ZeroOutput.into());
        }
        
        return Ok(amount);
    }
    
    // Для малых сумм (до 0.1% от резерва) используем линейную аппроксимацию
    if ndollar_amount < reserve_balance / SMALL_TRADE_THRESHOLD as u64 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        if current_price == 0 {
            return Err(TradingError::InvalidParameters.into());
        }
        
        let amount = ndollar_amount.checked_div(current_price)
            .ok_or(TradingError::ArithmeticError)?;
            
        if amount < 1 {
            return Err(TradingError::ZeroOutput.into());
        }
        
        return Ok(amount);
    }
    
    // Для более крупных сумм используем безопасный расчет с учетом слиппеджа
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    // Безопасная оценка с учетом слиппеджа (примерно 90% от линейной оценки для крупных сумм)
    let linear_estimate = if current_price > 0 { 
        ndollar_amount.checked_div(current_price).unwrap_or(0) 
    } else { 
        0 
    };
    
    let amount = if linear_estimate > 1000 {
        // Для крупных покупок учитываем слиппедж
        linear_estimate.checked_mul(LARGE_BUY_SLIPPAGE_FACTOR as u64)
            .and_then(|v| v.checked_div(100))
            .ok_or(TradingError::ArithmeticError)?
    } else {
        linear_estimate
    };
    
    if amount < 1 {
        return Err(TradingError::ZeroOutput.into());
    }
    
    Ok(amount)
}

/// Рассчитывает текущую цену токена на бондинговой кривой
pub fn get_current_price(
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
) -> Result<u64> {
    // Проверка на нулевые значения
    if total_supply == 0 {
        return Err(TradingError::InvalidParameters.into());
    }
    
    // Для power = 1: P = R / S
    // Для power = 2: P = 2 * R / S
    // Для power = n: P = n * R / S
    let price = match power {
        1 => reserve_balance.checked_div(total_supply),
        _ => {
            let numerator = reserve_balance.checked_mul(power as u64)
                .ok_or(TradingError::ArithmeticError)?;
            numerator.checked_div(total_supply)
        }
    }.ok_or(TradingError::ArithmeticError)?;
    
    Ok(price)
}
