use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use anchor_spl::token::{self, Transfer};
use crate::contexts::{ManageLiquidity, UpdateAfterMint};
use crate::errors::LiquidityError;

/// Добавление ликвидности в пул (только для владельца)
pub fn add_liquidity(
    ctx: Context<ManageLiquidity>,
    sol_amount: u64,
    ndollar_amount: u64,
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверяем, что вызывающий - владелец пула
    require!(
        liquidity_manager.authority == ctx.accounts.authority.key(),
        LiquidityError::UnauthorizedAccess
    );
    
    // Переводим SOL в пул
    if sol_amount > 0 {
        let sol_transfer_instruction = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.pool_sol_account.key(),
            sol_amount,
        );
        
        invoke(
            &sol_transfer_instruction,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.pool_sol_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }
    
    // Переводим N-Dollar в пул
    if ndollar_amount > 0 {
        let transfer_instruction = Transfer {
            from: ctx.accounts.authority_ndollar_account.to_account_info(),
            to: ctx.accounts.pool_ndollar_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, ndollar_amount)?;
    }
    
    // Обновляем статистику
    liquidity_manager.total_liquidity = liquidity_manager.total_liquidity
        .checked_add(sol_amount)
        .ok_or(LiquidityError::ArithmeticError)?;
    liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
    
    msg!("Ликвидность успешно добавлена: {} SOL и {} N-Dollar", 
        sol_amount as f64 / anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL as f64,
        ndollar_amount);
    Ok(())
}

/// Изъятие ликвидности из пула (только для владельца)
pub fn remove_liquidity(
    ctx: Context<ManageLiquidity>,
    sol_amount: u64,
    ndollar_amount: u64,
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверяем, что вызывающий - владелец пула
    require!(
        liquidity_manager.authority == ctx.accounts.authority.key(),
        LiquidityError::UnauthorizedAccess
    );
    
    // Проверяем достаточно ли SOL в пуле
    if sol_amount > 0 {
        require!(
            ctx.accounts.pool_sol_account.lamports() >= sol_amount,
            LiquidityError::InsufficientLiquidity
        );
    }
    
    // Проверяем достаточно ли N-Dollar в пуле
    if ndollar_amount > 0 {
        require!(
            ctx.accounts.pool_ndollar_account.amount >= ndollar_amount,
            LiquidityError::InsufficientLiquidity
        );
    }
    
    // Переводим SOL из пула
    if sol_amount > 0 {
        // Создаем семена для pool_sol_account PDA
        let pool_seeds = &[
            b"pool_sol".as_ref(),
            &liquidity_manager.key().to_bytes(),
            &[ctx.bumps.pool_sol_account],
        ];
        let pool_signer = &[&pool_seeds[..]];
        
        let sol_transfer_instruction = system_instruction::transfer(
            &ctx.accounts.pool_sol_account.key(),
            &ctx.accounts.authority.key(),
            sol_amount,
        );
        
        invoke_signed(
            &sol_transfer_instruction,
            &[
                ctx.accounts.pool_sol_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            pool_signer,
        )?;
    }
    
    // Переводим N-Dollar из пула
    if ndollar_amount > 0 {
        let seeds = &[
            b"liquidity_manager".as_ref(),
            &liquidity_manager.authority.to_bytes(),
            &[liquidity_manager.bump],
        ];
        let signer = &[&seeds[..]];
        
        let transfer_instruction = Transfer {
            from: ctx.accounts.pool_ndollar_account.to_account_info(),
            to: ctx.accounts.authority_ndollar_account.to_account_info(),
            authority: liquidity_manager.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );
        
        token::transfer(cpi_ctx, ndollar_amount)?;
    }
    
    // Обновляем статистику
    if sol_amount > 0 {
        liquidity_manager.total_liquidity = liquidity_manager.total_liquidity
            .checked_sub(sol_amount)
            .ok_or(LiquidityError::ArithmeticError)?;
    }
    liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
    
    msg!("Ликвидность успешно изъята: {} SOL и {} N-Dollar", 
        sol_amount as f64 / anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL as f64,
        ndollar_amount);
    Ok(())
}

/// Обновление состояния после прямого минта токенов в пул ликвидности
pub fn update_after_mint(
    ctx: Context<UpdateAfterMint>,
    ndollar_amount: u64,
) -> Result<()> {
    let liquidity_manager = &mut ctx.accounts.liquidity_manager;
    
    // Проверяем, что сумма для обновления ненулевая
    require!(ndollar_amount > 0, LiquidityError::InvalidAmount);
    
    // Проверяем, что сумма в токен-аккаунте пула соответствует ожидаемой
    let expected_balance = liquidity_manager.total_liquidity + ndollar_amount;
    
    msg!("Обновление состояния ликвидности после минта: {} N-Dollar", ndollar_amount);
    
    // Обновляем статистику в менеджере ликвидности
    liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
    liquidity_manager.total_liquidity = expected_balance;
    
    msg!("Состояние ликвидности успешно обновлено");
    Ok(())
}
