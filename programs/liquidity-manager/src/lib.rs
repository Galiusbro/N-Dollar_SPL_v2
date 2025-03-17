use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program::system_instruction;

declare_id!("4uP4rbMsgqMf9GKSAbbiDKKLJ1a2Rp4SEYE5jdhiVYLU");

#[program]
pub mod liquidity_manager {
    use super::*;

    /// Инициализация менеджера ликвидности
    pub fn initialize_liquidity_manager(
        ctx: Context<InitializeLiquidityManager>,
    ) -> Result<()> {
        let liquidity_manager = &mut ctx.accounts.liquidity_manager;
        liquidity_manager.authority = ctx.accounts.authority.key();
        liquidity_manager.n_dollar_mint = ctx.accounts.n_dollar_mint.key();
        liquidity_manager.total_liquidity = 0;
        liquidity_manager.total_users = 0;
        liquidity_manager.current_price = 1_000_000_000; // 1 SOL = 1000 N-Dollar (с учетом децималов)
        liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
        liquidity_manager.bump = ctx.bumps.liquidity_manager;
        
        msg!("Менеджер ликвидности успешно инициализирован");
        Ok(())
    }
    
    /// Покупка N-Dollar за SOL
    pub fn swap_sol_to_ndollar(
        ctx: Context<SwapSolToNDollar>,
        sol_amount: u64,
    ) -> Result<()> {
        let liquidity_manager = &mut ctx.accounts.liquidity_manager;
        
        // Рассчитываем количество N-Dollar на основе текущего курса
        // current_price = количество N-Dollar за 1 SOL (в лампортах)
        let ndollar_amount = sol_amount
            .checked_mul(liquidity_manager.current_price)
            .ok_or(LiquidityError::ArithmeticError)?
            .checked_div(anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL)
            .ok_or(LiquidityError::ArithmeticError)?;
        
        // Комиссия 1%
        let fee_percentage = 1;
        let fee_amount = ndollar_amount
            .checked_mul(fee_percentage)
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
        
        msg!("Успешно обменяно {} SOL на {} N-Dollar", 
            sol_amount as f64 / anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL as f64,
            net_ndollar_amount);
        msg!("Новая цена: 1 SOL = {} N-Dollar", 
            liquidity_manager.current_price as f64 / 1_000_000_000.0);
        Ok(())
    }
    
    /// Продажа N-Dollar за SOL
    pub fn swap_ndollar_to_sol(
        ctx: Context<SwapNDollarToSol>,
        ndollar_amount: u64,
    ) -> Result<()> {
        let liquidity_manager = &mut ctx.accounts.liquidity_manager;
        
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
        let fee_percentage = 1;
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
            let min_price = 500_000_000; // 50% от начальной цены 1_000_000_000
            
            liquidity_manager.current_price = if liquidity_manager.current_price > price_decrease.checked_add(min_price).unwrap_or(min_price) {
                liquidity_manager.current_price
                    .checked_sub(price_decrease)
                    .ok_or(LiquidityError::ArithmeticError)?
            } else {
                min_price
            };
        }
        
        liquidity_manager.last_update_time = Clock::get()?.unix_timestamp;
        
        msg!("Успешно обменяно {} N-Dollar на {} SOL", 
            ndollar_amount,
            net_sol_amount as f64 / anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL as f64);
        msg!("Новая цена: 1 SOL = {} N-Dollar", 
            liquidity_manager.current_price as f64 / 1_000_000_000.0);
        Ok(())
    }
    
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
}

#[derive(Accounts)]
pub struct InitializeLiquidityManager<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub n_dollar_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"liquidity_manager".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + LiquidityManager::SPACE
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SwapSolToNDollar<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"liquidity_manager".as_ref(), liquidity_manager.authority.as_ref()],
        bump = liquidity_manager.bump
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Это аккаунт для хранения SOL, принадлежащий пулу ликвидности
    #[account(
        mut,
        seeds = [b"pool_sol".as_ref(), liquidity_manager.key().as_ref()],
        bump,
    )]
    pub pool_sol_account: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = pool_ndollar_account.mint == liquidity_manager.n_dollar_mint
    )]
    pub pool_ndollar_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SwapNDollarToSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"liquidity_manager".as_ref(), liquidity_manager.authority.as_ref()],
        bump = liquidity_manager.bump
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Это аккаунт для хранения SOL, принадлежащий пулу ликвидности
    #[account(
        mut,
        seeds = [b"pool_sol".as_ref(), liquidity_manager.key().as_ref()],
        bump,
    )]
    pub pool_sol_account: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = pool_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = pool_ndollar_account.owner == liquidity_manager.key()
    )]
    pub pool_ndollar_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageLiquidity<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"liquidity_manager".as_ref(), authority.key().as_ref()],
        bump = liquidity_manager.bump,
        constraint = liquidity_manager.authority == authority.key()
    )]
    pub liquidity_manager: Account<'info, LiquidityManager>,
    
    #[account(
        mut,
        constraint = authority_ndollar_account.mint == liquidity_manager.n_dollar_mint,
        constraint = authority_ndollar_account.owner == authority.key()
    )]
    pub authority_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Это аккаунт для хранения SOL, принадлежащий пулу ликвидности
    #[account(
        mut,
        seeds = [b"pool_sol".as_ref(), liquidity_manager.key().as_ref()],
        bump,
    )]
    pub pool_sol_account: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = pool_ndollar_account.mint == liquidity_manager.n_dollar_mint
    )]
    pub pool_ndollar_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct LiquidityManager {
    pub authority: Pubkey,
    pub n_dollar_mint: Pubkey,
    pub total_liquidity: u64,
    pub total_users: u64,
    pub current_price: u64,  // Цена в N-Dollar за 1 SOL
    pub last_update_time: i64,
    pub bump: u8,
}

impl LiquidityManager {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1;
}

#[error_code]
pub enum LiquidityError {
    #[msg("Недостаточно ликвидности")]
    InsufficientLiquidity,
    #[msg("Недостаточно токенов на балансе")]
    InsufficientTokenBalance,
    #[msg("Арифметическая ошибка")]
    ArithmeticError,
    #[msg("Несанкционированный доступ")]
    UnauthorizedAccess,
    #[msg("Недопустимая сумма")]
    InvalidAmount,
    #[msg("Превышен максимальный лимит")]
    ExceedsMaximumAmount,
}
