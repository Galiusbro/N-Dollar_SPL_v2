use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use anchor_spl::token::{self, Transfer};
use crate::contexts::{SwapSolToNDollar, SwapNDollarToSol};
use crate::errors::LiquidityError;
use crate::constants::*;

/// Покупка N-Dollar за SOL
pub fn swap_sol_to_ndollar(
    ctx: Context<SwapSolToNDollar>,
    sol_amount: u64,
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверка на максимальный размер свопа
    require!(
        sol_amount <= MAX_SOL_SWAP_AMOUNT,
        LiquidityError::ExceedsMaximumSwapLimit
    );
    
    // Рассчитываем количество N-Dollar на основе текущего курса
    // current_price = количество N-Dollar за 1 SOL (в лампортах)
    let ndollar_amount = sol_amount
        .checked_mul(liquidity_manager.current_price)
        .ok_or(LiquidityError::ArithmeticError)?
        .checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Комиссия 1%
    let fee_amount = ndollar_amount
        .checked_mul(FEE_PERCENTAGE)
        .and_then(|v| v.checked_div(100))
        .ok_or(LiquidityError::ArithmeticError)?;
    
    let net_ndollar_amount = ndollar_amount
        .checked_sub(fee_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Проверяем, достаточно ли N-Dollar в пуле
    require!(
        ctx.accounts.pool_ndollar_account.amount >= net_ndollar_amount,
        LiquidityError::InsufficientLiquidity
    );
    
    // Проверка на манипуляции с ценой - влияние на ликвидность
    let pool_sol_balance = ctx.accounts.pool_sol_account.lamports();
    
    // Рассчитываем, какой процент от пула мы добавляем этой транзакцией
    let price_impact_percentage = (sol_amount as u128)
        .checked_mul(100)
        .and_then(|v| v.checked_div(pool_sol_balance as u128))
        .unwrap_or(0) as u64;
    
    let current_time = Clock::get()?.unix_timestamp;
    
    // Проверка на крупную транзакцию и возможное влияние на цену
    if price_impact_percentage > PRICE_IMPACT_THRESHOLD_PERCENTAGE {
        // Если прошлая крупная транзакция была недавно (в пределах кулдауна)
        let time_since_last_large_swap = current_time - liquidity_manager.last_large_swap_time;
        
        if time_since_last_large_swap < liquidity_manager.price_impact_cooldown as i64 {
            // Проверка на сэндвич-атаку (покупка-продажа-покупка или наоборот)
            // Если прошлая транзакция была в обратном направлении, это может быть манипуляцией
            if !liquidity_manager.last_large_swap_direction {
                require!(
                    time_since_last_large_swap >= PRICE_STABILITY_WINDOW * 3, // Требуем больший интервал для разнонаправленных транзакций
                    LiquidityError::PriceManipulationDetected
                );
            }
        }
        
        // Обновляем время последнего крупного свопа
        liquidity_manager.last_large_swap_time = current_time;
        liquidity_manager.last_large_swap_amount = sol_amount;
        liquidity_manager.last_large_swap_direction = true; // SOL -> N-Dollar
        liquidity_manager.last_update_time = current_time;
    }
    
    // Переводим SOL от пользователя в пул ликвидности
    let sol_transfer_instruction = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.pool_sol_account.key(),
        sol_amount,
    );
    
    invoke(
        &sol_transfer_instruction,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.pool_sol_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    
    // Переводим N-Dollar из пула ликвидности пользователю
    let seeds = &[
        b"liquidity_manager".as_ref(),
        &liquidity_manager.authority.to_bytes(),
        &[liquidity_manager.bump],
    ];
    let signer = &[&seeds[..]];
    
    let transfer_instruction = Transfer {
        from: ctx.accounts.pool_ndollar_account.to_account_info(),
        to: ctx.accounts.user_ndollar_account.to_account_info(),
        authority: liquidity_manager.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
        signer,
    );
    
    token::transfer(cpi_ctx, net_ndollar_amount)?;
    
    // Обновляем статистику
    liquidity_manager.total_users += 1;
    
    // Обновляем цену на основе изменения ликвидности
    // Увеличиваем цену на 0.1% за каждый SOL добавленный в пул
    let price_increase_percentage = sol_amount
        .checked_mul(10) // 0.1% за каждый SOL (10 базисных пунктов)
        .and_then(|v| v.checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL))
        .ok_or(LiquidityError::ArithmeticError)?;
    
    if price_increase_percentage > 0 {
        let price_increase = liquidity_manager.current_price
            .checked_mul(price_increase_percentage)
            .and_then(|v| v.checked_div(1000)) // Делим на 1000, поскольку это 0.1%
            .ok_or(LiquidityError::ArithmeticError)?;
        
        // Не позволяем цене вырасти более чем на 200% от начальной
        let max_price: u64 = MAX_PRICE;
        
        liquidity_manager.current_price = if liquidity_manager.current_price < max_price.checked_sub(price_increase).unwrap_or(max_price) {
            liquidity_manager.current_price
                .checked_add(price_increase)
                .ok_or(LiquidityError::ArithmeticError)?
        } else {
            max_price
        };
    }
    
    // Обновляем общую ликвидность
    liquidity_manager.total_liquidity = liquidity_manager.total_liquidity
        .checked_add(sol_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    liquidity_manager.last_update_time = current_time;
    
    msg!("Своп выполнен успешно: {} SOL -> {} N-Dollar", sol_amount, net_ndollar_amount);
    Ok(())
}

/// Обмен N-Dollar на SOL
pub fn swap_ndollar_to_sol(
    ctx: Context<SwapNDollarToSol>,
    ndollar_amount: u64,
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверка на максимальный размер свопа
    require!(
        ndollar_amount <= MAX_NDOLLAR_SWAP_AMOUNT,
        LiquidityError::ExceedsMaximumSwapLimit
    );
    
    // Проверяем, что у пользователя есть достаточно N-Dollar
    require!(
        ctx.accounts.user_ndollar_account.amount >= ndollar_amount,
        LiquidityError::InsufficientTokenBalance
    );
    
    // Защита от слишком малых значений
    require!(
        ndollar_amount > 0,
        LiquidityError::InvalidAmount
    );
    
    // Для улучшения отладки
    msg!("Запрошенная сумма N-Dollar для обмена: {}", ndollar_amount);
    msg!("Текущий курс: 1 SOL = {} N-Dollar", liquidity_manager.current_price);
    
    let lamports_per_sol = anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
    
    // Специальная обработка для очень малых сумм N-Dollar
    // Если сумма меньше 0.01 от текущей цены, используем минимальную сумму SOL
    let min_ndollar_threshold = liquidity_manager.current_price / 100; // 0.01 от цены 1 SOL
    
    let sol_amount = if ndollar_amount < min_ndollar_threshold {
        // Возвращаем пропорциональное количество SOL, но не менее 0.001 SOL
        let min_sol = lamports_per_sol / 1000; // 0.001 SOL
        
        // Расчет пропорционального количества SOL
        let proportional_sol = (ndollar_amount as u128)
            .checked_mul(lamports_per_sol as u128)
            .and_then(|v| v.checked_div(liquidity_manager.current_price as u128))
            .ok_or(LiquidityError::ArithmeticError)? as u64;
        
        std::cmp::max(proportional_sol, min_sol)
    } else {
        // Стандартный расчет для обычных сумм
        (ndollar_amount as u128)
            .checked_mul(lamports_per_sol as u128)
            .and_then(|v| v.checked_div(liquidity_manager.current_price as u128))
            .ok_or(LiquidityError::ArithmeticError)? as u64
    };
    
    msg!("Расчет: {} N-Dollar по курсу {} за 1 SOL = {} SOL (в ламппортах)", 
        ndollar_amount, 
        liquidity_manager.current_price, 
        sol_amount);
    
    // Защита от переполнения - ограничиваем макс. сумму SOL, которую может получить пользователь
    require!(
        sol_amount <= 100 * lamports_per_sol, // Максимум 100 SOL
        LiquidityError::ExceedsMaximumAmount
    );
    
    // Защита от нулевого результата
    require!(
        sol_amount > 0,
        LiquidityError::ArithmeticError
    );
    
    // Комиссия 1%
    let fee_percentage = FEE_PERCENTAGE;
    let fee_amount = sol_amount
        .checked_mul(fee_percentage)
        .and_then(|v| v.checked_div(100))
        .unwrap_or(1); // Минимум 1 лампорт комиссии
    
    let net_sol_amount = sol_amount
        .checked_sub(fee_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Проверяем, достаточно ли SOL в пуле
    require!(
        ctx.accounts.pool_sol_account.lamports() >= net_sol_amount,
        LiquidityError::InsufficientLiquidity
    );
    
    // Проверка на манипуляции с ценой - влияние на ликвидность
    let pool_ndollar_balance = ctx.accounts.pool_ndollar_account.amount;
    
    // Рассчитываем, какой процент от пула мы забираем этой транзакцией
    let price_impact_percentage = (ndollar_amount as u128)
        .checked_mul(100)
        .and_then(|v| v.checked_div(pool_ndollar_balance as u128))
        .unwrap_or(0) as u64;
    
    let current_time = Clock::get()?.unix_timestamp;
    
    // Проверка на крупную транзакцию и возможное влияние на цену
    if price_impact_percentage > PRICE_IMPACT_THRESHOLD_PERCENTAGE {
        // Если прошлая крупная транзакция была недавно (в пределах кулдауна)
        let time_since_last_large_swap = current_time - liquidity_manager.last_large_swap_time;
        
        if time_since_last_large_swap < liquidity_manager.price_impact_cooldown as i64 {
            // Проверка на сэндвич-атаку (покупка-продажа-покупка или наоборот)
            // Если прошлая транзакция была в обратном направлении, это может быть манипуляцией
            if liquidity_manager.last_large_swap_direction {
                require!(
                    time_since_last_large_swap >= PRICE_STABILITY_WINDOW * 3, // Требуем больший интервал для разнонаправленных транзакций
                    LiquidityError::PriceManipulationDetected
                );
            }
        }
        
        // Обновляем время последнего крупного свопа
        liquidity_manager.last_large_swap_time = current_time;
        liquidity_manager.last_large_swap_amount = ndollar_amount;
        liquidity_manager.last_large_swap_direction = false; // N-Dollar -> SOL
        liquidity_manager.last_update_time = current_time;
    }
    
    // Переводим N-Dollar от пользователя в пул ликвидности
    let transfer_instruction = Transfer {
        from: ctx.accounts.user_ndollar_account.to_account_info(),
        to: ctx.accounts.pool_ndollar_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );
    
    token::transfer(cpi_ctx, ndollar_amount)?;
    
    // Создаем семена для pool_sol_account PDA
    let pool_seeds = &[
        b"pool_sol".as_ref(),
        &liquidity_manager.key().to_bytes(),
        &[ctx.bumps.pool_sol_account],
    ];
    let pool_signer = &[&pool_seeds[..]];
    
    // Переводим SOL из пула ликвидности пользователю
    let sol_transfer_instruction = system_instruction::transfer(
        &ctx.accounts.pool_sol_account.key(),
        &ctx.accounts.user.key(),
        net_sol_amount,
    );
    
    invoke_signed(
        &sol_transfer_instruction,
        &[
            ctx.accounts.pool_sol_account.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        pool_signer,
    )?;
    
    // Обновляем статистику
    if liquidity_manager.total_liquidity >= net_sol_amount {
        liquidity_manager.total_liquidity = liquidity_manager.total_liquidity
            .checked_sub(net_sol_amount)
            .ok_or(LiquidityError::ArithmeticError)?;
    } else {
        liquidity_manager.total_liquidity = 0;
    }
    
    // Обновляем цену на основе изменения ликвидности
    // Уменьшаем цену на 0.1% за каждый SOL изъятый из пула
    let price_decrease_percentage = net_sol_amount
        .checked_mul(10) // 0.1% за каждый SOL (10 базисных пунктов)
        .and_then(|v| v.checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL))
        .ok_or(LiquidityError::ArithmeticError)?;
    
    if price_decrease_percentage > 0 {
        let price_decrease = liquidity_manager.current_price
            .checked_mul(price_decrease_percentage)
            .and_then(|v| v.checked_div(1000)) // Делим на 1000, поскольку это 0.1%
            .ok_or(LiquidityError::ArithmeticError)?;
        
        // Не позволяем цене упасть ниже 50% от начальной
        let min_price = MIN_PRICE;
        
        liquidity_manager.current_price = if liquidity_manager.current_price > price_decrease.checked_add(min_price).unwrap_or(min_price) {
            liquidity_manager.current_price
                .checked_sub(price_decrease)
                .ok_or(LiquidityError::ArithmeticError)?
        } else {
            min_price
        };
    }
    
    liquidity_manager.last_update_time = current_time;
    
    msg!("Своп выполнен успешно: {} N-Dollar -> {} SOL", ndollar_amount, net_sol_amount);
    Ok(())
}

/// Покупка N-Dollar за SOL с защитой от проскальзывания
pub fn swap_sol_to_ndollar_with_slippage(
    ctx: Context<SwapSolToNDollar>,
    sol_amount: u64,
    min_ndollar_amount: u64, // Минимальное количество N-Dollar, которое пользователь хочет получить
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверка на максимальный размер свопа
    require!(
        sol_amount <= MAX_SOL_SWAP_AMOUNT,
        LiquidityError::ExceedsMaximumSwapLimit
    );
    
    // Рассчитываем количество N-Dollar на основе текущего курса
    // current_price = количество N-Dollar за 1 SOL (в лампортах)
    let ndollar_amount = sol_amount
        .checked_mul(liquidity_manager.current_price)
        .ok_or(LiquidityError::ArithmeticError)?
        .checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Комиссия 1%
    let fee_percentage = FEE_PERCENTAGE;
    let fee_amount = ndollar_amount
        .checked_mul(fee_percentage)
        .and_then(|v| v.checked_div(100))
        .ok_or(LiquidityError::ArithmeticError)?;
    
    let net_ndollar_amount = ndollar_amount
        .checked_sub(fee_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Защита от проскальзывания - проверяем, что пользователь получит не меньше указанного минимума
    require!(
        net_ndollar_amount >= min_ndollar_amount,
        LiquidityError::SlippageExceeded
    );
    
    // Проверяем, достаточно ли N-Dollar в пуле
    require!(
        ctx.accounts.pool_ndollar_account.amount >= net_ndollar_amount,
        LiquidityError::InsufficientLiquidity
    );
    
    // Проверка на манипуляции с ценой - влияние на ликвидность
    let pool_sol_balance = ctx.accounts.pool_sol_account.lamports();
    
    // Рассчитываем, какой процент от пула мы добавляем этой транзакцией
    let price_impact_percentage = (sol_amount as u128)
        .checked_mul(100)
        .and_then(|v| v.checked_div(pool_sol_balance as u128))
        .unwrap_or(0) as u64;
    
    let current_time = Clock::get()?.unix_timestamp;
    
    // Проверка на крупную транзакцию и возможное влияние на цену
    if price_impact_percentage > PRICE_IMPACT_THRESHOLD_PERCENTAGE {
        // Если прошлая крупная транзакция была недавно (в пределах кулдауна)
        let time_since_last_large_swap = current_time - liquidity_manager.last_large_swap_time;
        
        if time_since_last_large_swap < liquidity_manager.price_impact_cooldown as i64 {
            // Проверка на сэндвич-атаку (покупка-продажа-покупка или наоборот)
            // Если прошлая транзакция была в обратном направлении, это может быть манипуляцией
            if !liquidity_manager.last_large_swap_direction {
                require!(
                    time_since_last_large_swap >= PRICE_STABILITY_WINDOW * 3, // Требуем больший интервал для разнонаправленных транзакций
                    LiquidityError::PriceManipulationDetected
                );
            }
        }
        
        // Обновляем время последнего крупного свопа
        liquidity_manager.last_large_swap_time = current_time;
        liquidity_manager.last_large_swap_amount = sol_amount;
        liquidity_manager.last_large_swap_direction = true; // SOL -> N-Dollar
        liquidity_manager.last_update_time = current_time;
    }
    
    // Переводим SOL от пользователя в пул ликвидности
    let sol_transfer_instruction = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.pool_sol_account.key(),
        sol_amount,
    );
    
    invoke(
        &sol_transfer_instruction,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.pool_sol_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    
    // Переводим N-Dollar из пула ликвидности пользователю
    let seeds = &[
        b"liquidity_manager".as_ref(),
        &liquidity_manager.authority.to_bytes(),
        &[liquidity_manager.bump],
    ];
    let signer = &[&seeds[..]];
    
    let transfer_instruction = Transfer {
        from: ctx.accounts.pool_ndollar_account.to_account_info(),
        to: ctx.accounts.user_ndollar_account.to_account_info(),
        authority: liquidity_manager.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
        signer,
    );
    
    token::transfer(cpi_ctx, net_ndollar_amount)?;
    
    // Обновляем статистику
    liquidity_manager.total_liquidity = liquidity_manager.total_liquidity
        .checked_add(sol_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    liquidity_manager.total_users = liquidity_manager.total_users
        .checked_add(1)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Обновляем цену на основе изменения ликвидности
    // Увеличиваем цену на 0.1% за каждый SOL добавленный в пул
    let price_increase_percentage = sol_amount
        .checked_mul(10) // 0.1% за каждый SOL (10 базисных пунктов)
        .and_then(|v| v.checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL))
        .ok_or(LiquidityError::ArithmeticError)?;
    
    if price_increase_percentage > 0 {
        let price_increase = liquidity_manager.current_price
            .checked_mul(price_increase_percentage)
            .and_then(|v| v.checked_div(1000)) // Делим на 1000, поскольку это 0.1%
            .ok_or(LiquidityError::ArithmeticError)?;
        
        liquidity_manager.current_price = liquidity_manager.current_price
            .checked_add(price_increase)
            .ok_or(LiquidityError::ArithmeticError)?;
    }
    
    liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
    
    msg!("Своп выполнен успешно: {} SOL -> {} N-Dollar", sol_amount, net_ndollar_amount);
    Ok(())
}

/// Обмен N-Dollar на SOL с защитой от проскальзывания
pub fn swap_ndollar_to_sol_with_slippage(
    ctx: Context<SwapNDollarToSol>,
    ndollar_amount: u64,
    min_sol_amount: u64, // Минимальное количество SOL, которое пользователь хочет получить
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверка на максимальный размер свопа
    require!(
        ndollar_amount <= MAX_NDOLLAR_SWAP_AMOUNT,
        LiquidityError::ExceedsMaximumSwapLimit
    );
    
    // Проверяем, что у пользователя есть достаточно N-Dollar
    require!(
        ctx.accounts.user_ndollar_account.amount >= ndollar_amount,
        LiquidityError::InsufficientTokenBalance
    );
    
    // Защита от слишком малых значений
    require!(
        ndollar_amount > 0,
        LiquidityError::InvalidAmount
    );
    
    // Для улучшения отладки
    msg!("Запрошенная сумма N-Dollar для обмена: {}", ndollar_amount);
    msg!("Текущий курс: 1 SOL = {} N-Dollar", liquidity_manager.current_price);
    
    let lamports_per_sol = anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
    
    // Специальная обработка для очень малых сумм N-Dollar
    // Если сумма меньше 0.01 от текущей цены, используем минимальную сумму SOL
    let min_ndollar_threshold = liquidity_manager.current_price / 100; // 0.01 от цены 1 SOL
    
    let sol_amount = if ndollar_amount < min_ndollar_threshold {
        // Возвращаем пропорциональное количество SOL, но не менее 0.001 SOL
        let min_sol = lamports_per_sol / 1000; // 0.001 SOL
        
        // Расчет пропорционального количества SOL
        let proportional_sol = (ndollar_amount as u128)
            .checked_mul(lamports_per_sol as u128)
            .and_then(|v| v.checked_div(liquidity_manager.current_price as u128))
            .ok_or(LiquidityError::ArithmeticError)? as u64;
        
        std::cmp::max(proportional_sol, min_sol)
    } else {
        // Стандартный расчет для обычных сумм
        (ndollar_amount as u128)
            .checked_mul(lamports_per_sol as u128)
            .and_then(|v| v.checked_div(liquidity_manager.current_price as u128))
            .ok_or(LiquidityError::ArithmeticError)? as u64
    };
    
    msg!("Расчет: {} N-Dollar по курсу {} за 1 SOL = {} SOL (в ламппортах)", 
        ndollar_amount, 
        liquidity_manager.current_price, 
        sol_amount);
    
    // Защита от переполнения - ограничиваем макс. сумму SOL, которую может получить пользователь
    require!(
        sol_amount <= 100 * lamports_per_sol, // Максимум 100 SOL
        LiquidityError::ExceedsMaximumAmount
    );
    
    // Защита от нулевого результата
    require!(
        sol_amount > 0,
        LiquidityError::ArithmeticError
    );
    
    // Комиссия 1%
    let fee_percentage = FEE_PERCENTAGE;
    let fee_amount = sol_amount
        .checked_mul(fee_percentage)
        .and_then(|v| v.checked_div(100))
        .unwrap_or(1); // Минимум 1 лампорт комиссии
    
    let net_sol_amount = sol_amount
        .checked_sub(fee_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    
    // Защита от проскальзывания - проверяем, что пользователь получит не меньше указанного минимума
    require!(
        net_sol_amount >= min_sol_amount,
        LiquidityError::SlippageExceeded
    );
    
    // Проверяем, достаточно ли SOL в пуле
    require!(
        ctx.accounts.pool_sol_account.lamports() >= net_sol_amount,
        LiquidityError::InsufficientLiquidity
    );
    
    // Проверка на манипуляции с ценой - влияние на ликвидность
    let pool_ndollar_balance = ctx.accounts.pool_ndollar_account.amount;
    
    // Рассчитываем, какой процент от пула мы забираем этой транзакцией
    let price_impact_percentage = (ndollar_amount as u128)
        .checked_mul(100)
        .and_then(|v| v.checked_div(pool_ndollar_balance as u128))
        .unwrap_or(0) as u64;
    
    let current_time = Clock::get()?.unix_timestamp;
    
    // Проверка на крупную транзакцию и возможное влияние на цену
    if price_impact_percentage > PRICE_IMPACT_THRESHOLD_PERCENTAGE {
        // Если прошлая крупная транзакция была недавно (в пределах кулдауна)
        let time_since_last_large_swap = current_time - liquidity_manager.last_large_swap_time;
        
        if time_since_last_large_swap < liquidity_manager.price_impact_cooldown as i64 {
            // Проверка на сэндвич-атаку (покупка-продажа-покупка или наоборот)
            // Если прошлая транзакция была в обратном направлении, это может быть манипуляцией
            if liquidity_manager.last_large_swap_direction {
                require!(
                    time_since_last_large_swap >= PRICE_STABILITY_WINDOW * 3, // Требуем больший интервал для разнонаправленных транзакций
                    LiquidityError::PriceManipulationDetected
                );
            }
        }
        
        // Обновляем время последнего крупного свопа
        liquidity_manager.last_large_swap_time = current_time;
        liquidity_manager.last_large_swap_amount = ndollar_amount;
        liquidity_manager.last_large_swap_direction = false; // N-Dollar -> SOL
        liquidity_manager.last_update_time = current_time;
    }
    
    // Переводим N-Dollar от пользователя в пул ликвидности
    let transfer_instruction = Transfer {
        from: ctx.accounts.user_ndollar_account.to_account_info(),
        to: ctx.accounts.pool_ndollar_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );
    
    token::transfer(cpi_ctx, ndollar_amount)?;
    
    // Создаем семена для pool_sol_account PDA
    let pool_seeds = &[
        b"pool_sol".as_ref(),
        &liquidity_manager.key().to_bytes(),
        &[ctx.bumps.pool_sol_account],
    ];
    let pool_signer = &[&pool_seeds[..]];
    
    // Переводим SOL из пула ликвидности пользователю
    let sol_transfer_instruction = system_instruction::transfer(
        &ctx.accounts.pool_sol_account.key(),
        &ctx.accounts.user.key(),
        net_sol_amount,
    );
    
    invoke_signed(
        &sol_transfer_instruction,
        &[
            ctx.accounts.pool_sol_account.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        pool_signer,
    )?;
    
    // Обновляем статистику
    if liquidity_manager.total_liquidity >= net_sol_amount {
        liquidity_manager.total_liquidity = liquidity_manager.total_liquidity
            .checked_sub(net_sol_amount)
            .ok_or(LiquidityError::ArithmeticError)?;
    } else {
        liquidity_manager.total_liquidity = 0;
    }
    
    liquidity_manager.last_update_time = current_time;
    
    msg!("Своп выполнен успешно с защитой от проскальзывания: {} N-Dollar -> {} SOL (мин. запрошено: {})", 
        ndollar_amount, net_sol_amount, min_sol_amount);
    Ok(())
}