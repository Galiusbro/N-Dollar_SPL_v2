use anchor_lang::prelude::*;
use crate::errors::BondingCurveError;

/// Константы для бондинговой кривой
pub mod constants {
    // Степенной показатель кривой по умолчанию (2 = квадратичная кривая)
    pub const DEFAULT_POWER: u8 = 2;
    
    // Комиссия по умолчанию в базисных пунктах (50 = 0.5%)
    pub const DEFAULT_FEE_PERCENT: u16 = 50;
    
    // Минимальная начальная цена
    pub const MIN_INITIAL_PRICE: u64 = 1;
    
    // Максимальный размер транзакции в единицах токена 
    pub const MAX_TOKEN_TRANSACTION: u64 = 1_000_000_000;
    
    // Максимальный размер транзакции в N-Dollar (1000 N-Dollar, с 9 знаками после запятой)
    pub const MAX_NDOLLAR_TRANSACTION: u64 = 1_000_000_000 * 10u64.pow(9);
    
    // Максимальное значение fee_percent (10% = 1000 базисных пунктов)
    pub const MAX_FEE_PERCENT: u16 = 1000;
    
    // Минимальное количество токенов, которое можно купить
    pub const MIN_TOKEN_AMOUNT: u64 = 1;
    
    // Минимальные значения для предотвращения числовых ошибок
    pub const MIN_SAFE_SUPPLY: u64 = 10;
}

/// Вспомогательная функция для вычисления комиссии
pub fn calculate_fee(amount: u64, fee_percent: u16) -> Result<u64> {
    // Преобразуем fee_percent из базисных пунктов (1/100 процента) в проценты от суммы
    // fee_percent 100 = 1% комиссии
    let fee_u128 = (amount as u128)
        .checked_mul(fee_percent as u128)
        .ok_or(BondingCurveError::ArithmeticError)?
        .checked_div(10_000) // Делим на 10000, так как fee_percent в базисных пунктах
        .ok_or(BondingCurveError::ArithmeticError)?;
    
    // Проверяем переполнение при обратном преобразовании в u64
    if fee_u128 > u64::MAX as u128 {
        return Err(BondingCurveError::ArithmeticError.into());
    }
    
    Ok(fee_u128 as u64)
}

/// Вычисляет constant product для кривой.
/// Формула: constant = reserve_balance * (total_supply^power)
pub fn calculate_constant_product(
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
) -> Result<u128> {
    // Проверяем, что степень находится в допустимом диапазоне
    require!(
        power >= 1 && power <= 10,
        BondingCurveError::InvalidParameter
    );
    
    // Используем u128 для промежуточных вычислений
    let supply_u128 = total_supply as u128;
    let reserve_u128 = reserve_balance as u128;
    
    // Если supply или reserve равны 0, constant_product тоже 0
    if supply_u128 == 0 || reserve_u128 == 0 {
        return Ok(0);
    }
    
    // Вычисляем total_supply^power с безопасной обработкой больших чисел
    let mut supply_pow = 1u128;
    for _ in 0..power {
        supply_pow = supply_pow.checked_mul(supply_u128)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Если результат становится слишком большим, сокращаем для предотвращения переполнения
        if supply_pow > (u64::MAX as u128) * (u64::MAX as u128) {
            return Err(BondingCurveError::ArithmeticError.into());
        }
    }
    
    // Вычисляем constant product
    let constant = reserve_u128.checked_mul(supply_pow)
        .ok_or(BondingCurveError::ArithmeticError)?;
    
    Ok(constant)
}

/// Рассчитывает текущую цену токена.
/// Формула: price = (reserve_balance * power) / total_supply
pub fn get_current_price(
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
) -> Result<u64> {
    // Проверяем, что степень находится в допустимом диапазоне
    require!(
        power >= 1 && power <= 10,
        BondingCurveError::InvalidParameter
    );
    
    if total_supply == 0 {
        return Err(BondingCurveError::ZeroDivision.into());
    }
    
    // Используем u128 для безопасности вычислений
    let supply_u128 = total_supply as u128;
    let reserve_u128 = reserve_balance as u128;
    let power_u128 = power as u128;
    
    // Рассчитываем цену: (reserve * power) / supply
    let numerator = reserve_u128.checked_mul(power_u128)
        .ok_or(BondingCurveError::ArithmeticError)?;
    let price_u128 = numerator.checked_div(supply_u128)
        .ok_or(BondingCurveError::ZeroDivision)?;
    
    // Проверяем на переполнение при конвертации обратно в u64
    if price_u128 > u64::MAX as u128 {
        return Err(BondingCurveError::ArithmeticError.into());
    }
    
    Ok(price_u128 as u64)
}

/// Рассчитывает количество токенов, которое пользователь получит при покупке.
pub fn calculate_buy_amount(
    total_supply: u64,
    reserve_balance: u64,
    ndollar_amount: u64,
    power: u8,
    initial_price: u64,
) -> Result<u64> {
    // Проверяем, что степень находится в допустимом диапазоне
    require!(
        power >= 1 && power <= 10,
        BondingCurveError::InvalidParameter
    );
    
    // Обработка очень маленьких сумм - требуем минимальную сумму для покупки
    if ndollar_amount < 10 { // Например, менее 10 ламп (0.00000001 NDollar)
        return Err(BondingCurveError::AmountTooSmall.into());
    }
    
    // Если это первая покупка или резерв пуст, используем начальную цену
    if total_supply == 0 || reserve_balance == 0 {
        // Простое деление по начальной цене
        if initial_price == 0 {
            return Err(BondingCurveError::ZeroDivision.into());
        }
        
        // Проверяем минимальное количество токенов
        let amount = ndollar_amount / initial_price;
        if amount < 1 {
            return Err(BondingCurveError::ZeroOutput.into());
        }
        
        return Ok(amount);
    }
    
    // Для малых сумм (до 0.1% от резерва) используем линейную аппроксимацию
    if ndollar_amount < reserve_balance / 1000 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        if current_price == 0 {
            return Err(BondingCurveError::ZeroDivision.into());
        }
        
        // Проверяем минимальное количество токенов
        let amount = ndollar_amount / current_price;
        if amount < 1 {
            return Err(BondingCurveError::ZeroOutput.into());
        }
        
        return Ok(amount);
    }
    
    // Для более крупных сумм используем безопасный расчет с учетом слиппеджа
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    // Безопасная оценка с учетом слиппеджа (примерно 90% от линейной оценки для крупных сумм)
    let linear_estimate = if current_price > 0 { ndollar_amount / current_price } else { 0 };
    let amount = if linear_estimate > 1000 {
        // Для крупных покупок учитываем слиппедж
        linear_estimate * 9 / 10 // Примерно 90% от линейной оценки
    } else {
        linear_estimate
    };
    
    // Проверяем минимальное количество токенов
    if amount < 1 {
        return Err(BondingCurveError::ZeroOutput.into());
    }
    
    Ok(amount)
}

/// Рассчитывает количество NDollar и комиссию при продаже токенов.
pub fn calculate_sell_amount(
    total_supply: u64,
    reserve_balance: u64,
    token_amount: u64,
    power: u8,
    fee_percent: u16,
) -> Result<(u64, u64)> {
    // Проверяем, что степень находится в допустимом диапазоне
    require!(
        power >= 1 && power <= 10,
        BondingCurveError::InvalidParameter
    );
    
    // Проверка на нулевой supply или резерв
    if total_supply == 0 || reserve_balance == 0 {
        return Err(BondingCurveError::InsufficientLiquidity.into());
    }
    
    // Проверка, что мы не пытаемся продать больше, чем есть в обращении
    if token_amount >= total_supply {
        if token_amount == total_supply {
            // Особый случай: продаем все токены, возвращаем весь резерв за вычетом минимального остатка
            // для поддержания ликвидности
            let reserve_amount = reserve_balance.saturating_sub(1000); // Оставляем минимальный резерв
            let fee_amount = calculate_fee(reserve_amount, fee_percent)?;
            let final_amount = reserve_amount.checked_sub(fee_amount)
                .ok_or(BondingCurveError::ArithmeticError)?;
                
            return Ok((final_amount, fee_amount));
        } else {
            return Err(BondingCurveError::InsufficientTokens.into());
        }
    }
    
    // Для малых сумм (до 0.1% от total_supply) используем линейную аппроксимацию
    if token_amount < total_supply / 1000 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        let reserve_delta = token_amount.checked_mul(current_price)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Проверяем, что не пытаемся получить больше резерва, чем есть
        if reserve_delta > reserve_balance {
            return Err(BondingCurveError::InsufficientLiquidity.into());
        }
        
        // Рассчитываем комиссию
        let fee_amount = calculate_fee(reserve_delta, fee_percent)?;
        
        // Вычисляем итоговую сумму к получению
        let reserve_amount = reserve_delta.checked_sub(fee_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        return Ok((reserve_amount, fee_amount));
    }
    
    // Для более крупных продаж используем безопасный расчет с учетом слиппеджа
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    // Безопасная оценка: с учетом слиппеджа (примерно 85% от линейной оценки для крупных сумм)
    let linear_estimate = token_amount.checked_mul(current_price)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    let reserve_delta = if token_amount > total_supply / 10 {
        // Для крупных продаж (более 10% от всех токенов) учитываем слиппедж
        linear_estimate * 85 / 100 // 85% от линейной оценки
    } else {
        linear_estimate
    };
    
    // Проверяем, что не пытаемся получить больше резерва, чем есть
    if reserve_delta > reserve_balance {
        return Err(BondingCurveError::InsufficientLiquidity.into());
    }
    
    // Рассчитываем комиссию
    let fee_amount = calculate_fee(reserve_delta, fee_percent)?;
    
    // Вычисляем итоговую сумму к получению
    let reserve_amount = reserve_delta.checked_sub(fee_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
    
    Ok((reserve_amount, fee_amount))
} 