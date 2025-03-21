use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};
use crate::contexts::{MintSupply, MintToLiquidity};
use crate::errors::NDollarError;
use crate::constants::*;
use crate::instructions::utils::{verify_admin_control_authorization, verify_time_manipulation};
use liquidity_manager::liquidity_cpi;

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

    // Определяем сумму для минта в зависимости от текущей недели
    let mint_amount = match admin_account.current_mint_week {
        1 => {
            // Первая неделя уже выпущена при инициализации
            if admin_account.current_mint_week < 2 {
                admin_account.current_mint_week = 2;
                0 // Не делаем дополнительный минт
            } else {
                amount // Пользовательский минт
            }
        },
        2 => {
            // Переходим на неделю 2, минтим 54 млрд
            admin_account.current_mint_week = 3;
            (WEEK2_SUPPLY * 1_000_000_000) as u64 // С учетом decimals
        },
        3 => {
            // Переходим на неделю 3, минтим 108 млрд
            admin_account.current_mint_week = 4;
            (WEEK3_SUPPLY * 1_000_000_000) as u64 // С учетом decimals
        },
        4 => {
            // Переходим на неделю 4, минтим 369 млрд
            admin_account.current_mint_week = 5; // Указываем неделю 5, чтобы знать, что расписание закончено
            (WEEK4_SUPPLY * 1_000_000_000) as u64 // С учетом decimals
        },
        _ => {
            // Расписание завершено, больше не минтим автоматически
            require!(amount > 0, NDollarError::InvalidMintAmount);
            amount // После завершения расписания разрешаем только ручной минт
        }
    };

    // Проверка, что сумма для минта ненулевая
    if mint_amount == 0 {
        msg!("Минт пропущен, так как сумма равна 0");
        return Ok(());
    }

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
    
    token::mint_to(cpi_ctx, mint_amount)?;
    
    // Обновляем информацию о последнем минте
    admin_account.last_mint_time = current_time;
    admin_account.last_block_time = current_time;
    admin_account.last_block_height = current_slot;
    admin_account.total_supply += mint_amount;
    
    msg!("Минт выполнен успешно, добавлено: {}", mint_amount);
    Ok(())
}

/// Минтинг токенов с автоматическим распределением в пул ликвидности
pub fn mint_to_liquidity(ctx: Context<MintToLiquidity>, amount: u64) -> Result<()> {
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

    // Определяем сумму для минта в зависимости от текущей недели
    let mint_amount = match admin_account.current_mint_week {
        1 => {
            // Первая неделя уже выпущена при инициализации
            if admin_account.current_mint_week < 2 {
                admin_account.current_mint_week = 2;
                0 // Не делаем дополнительный минт
            } else {
                amount // Пользовательский минт
            }
        },
        2 => {
            // Переходим на неделю 2, минтим 54 млрд
            admin_account.current_mint_week = 3;
            (WEEK2_SUPPLY * 1_000_000_000) as u64 // С учетом decimals
        },
        3 => {
            // Переходим на неделю 3, минтим 108 млрд
            admin_account.current_mint_week = 4;
            (WEEK3_SUPPLY * 1_000_000_000) as u64 // С учетом decimals
        },
        4 => {
            // Переходим на неделю 4, минтим 369 млрд
            admin_account.current_mint_week = 5; // Указываем неделю 5, чтобы знать, что расписание закончено
            (WEEK4_SUPPLY * 1_000_000_000) as u64 // С учетом decimals
        },
        _ => {
            // Расписание завершено, больше не минтим автоматически
            require!(amount > 0, NDollarError::InvalidMintAmount);
            amount // После завершения расписания разрешаем только ручной минт
        }
    };

    // Проверка, что сумма для минта ненулевая
    if mint_amount == 0 {
        msg!("Минт пропущен, так как сумма равна 0");
        return Ok(());
    }

    // Рассчитываем сколько идет в пул ликвидности, а сколько остается у админа
    let liquidity_amount = mint_amount
        .checked_mul(LIQUIDITY_POOL_PERCENTAGE as u64)
        .and_then(|v| v.checked_div(100))
        .ok_or(NDollarError::ArithmeticError)?;
    
    let admin_amount = mint_amount
        .checked_mul(ADMIN_RESERVE_PERCENTAGE as u64)
        .and_then(|v| v.checked_div(100))
        .ok_or(NDollarError::ArithmeticError)?;
    
    // Проверка, что суммы корректно рассчитаны
    let total_minted = liquidity_amount
        .checked_add(admin_amount)
        .ok_or(NDollarError::ArithmeticError)?;
    
    require!(
        total_minted == mint_amount,
        NDollarError::ArithmeticError
    );

    let seeds = &[
        b"admin_account".as_ref(),
        &admin_account.mint.to_bytes(),
        &[admin_account.bump],
    ];
    let signer = &[&seeds[..]];
    
    // Минтим токены в пул ликвидности
    if liquidity_amount > 0 {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.liquidity_pool_account.to_account_info(),
            authority: admin_account.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, liquidity_amount)?;
        msg!("Минт в пул ликвидности: {}", liquidity_amount);
        
        // Вызываем инструкцию обновления ликвидности после минта
        if let Some(liquidity_manager_program) = &ctx.accounts.liquidity_manager_program {
            msg!("Обновляем состояние ликвидности в liquidity-manager");
            
            // Подготавливаем аккаунты для CPI вызова
            let accounts = vec![
                ctx.accounts.liquidity_manager.to_account_info(),
                ctx.accounts.liquidity_pool_account.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ];
            
            // Выполняем CPI вызов к liquidity-manager
            liquidity_cpi::update_after_mint(
                liquidity_manager_program.to_account_info(),
                ctx.accounts.liquidity_manager.to_account_info(),
                ctx.accounts.liquidity_pool_account.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                liquidity_amount,
                accounts,
                signer,
            )?;
            
            msg!("Состояние ликвидности успешно обновлено");
        } else {
            msg!("Пропускаем обновление состояния ликвидности: программа не предоставлена");
        }
    }
    
    // Минтим резервную часть токенов администратору
    if admin_amount > 0 {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.admin_token_account.to_account_info(),
            authority: admin_account.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, admin_amount)?;
        msg!("Минт администратору: {}", admin_amount);
    }
    
    // Обновляем информацию о последнем минте
    admin_account.last_mint_time = current_time;
    admin_account.last_block_time = current_time;
    admin_account.last_block_height = current_slot;
    admin_account.total_supply += mint_amount;
    
    msg!("Минт с распределением выполнен успешно, всего добавлено: {}", mint_amount);
    Ok(())
}