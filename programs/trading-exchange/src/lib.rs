use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::pubkey::Pubkey;
use bonding_curve::BondingCurve;
// Импортируем модуль admin_control для авторизации
use admin_control::admin_cpi;

declare_id!("7i7EpxhmCxmDhBvcTNFVXqq2SRQNt7HG98ANxRdcF6Dh");

#[program]
pub mod trading_exchange {
    use super::*;

    /// Свап между различными токенами по их курсу относительно N-Dollar
    pub fn swap_tokens(
        ctx: Context<SwapTokens>,
        amount_in: u64,
    ) -> Result<()> {
        // Проверка авторизации через admin_control, если admin_config передан
        if ctx.accounts.admin_config.is_some() && ctx.accounts.admin_control_program.is_some() {
            // Проверка, что текущая программа авторизована в admin_control
            let program_id = crate::ID;
            let is_authorized = admin_cpi::verify_program_authorization(
                &ctx.accounts.admin_config.as_ref().unwrap().to_account_info(),
                &program_id,
                &ctx.accounts.admin_control_program.as_ref().unwrap().to_account_info(),
            )?;
            
            require!(is_authorized, TradingError::UnauthorizedAccess);
        }
        
        // Получаем информацию о токенах
        let from_mint = ctx.accounts.from_mint.key();
        let to_mint = ctx.accounts.to_mint.key();
        let user = &ctx.accounts.user;
        
        // Проверка, что у пользователя достаточно токенов для обмена
        require!(
            ctx.accounts.user_from_account.amount >= amount_in,
            TradingError::InsufficientTokenBalance
        );
        
        // Проверяем активность контрольного аккаунта
        require!(
            ctx.accounts.control_state.is_active,
            TradingError::ProgramNotActive
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
        // Для этого используем имитацию продажи через бондинговую кривую
        let ndollar_amount = calculate_token_ndollar_value(
            amount_in, 
            from_bonding_curve.total_supply_in_curve,
            from_bonding_curve.reserve_balance,
            from_bonding_curve.power
        )?;
        
        // 2. Определяем комиссию за свап
        // Получаем комиссию из admin_control, если возможно
        let fee_percentage = if ctx.accounts.admin_config.is_some() && ctx.accounts.admin_control_program.is_some() {
            // Получаем комиссию из admin_config
            let fee_basis_points = admin_cpi::get_fee_basis_points(
                &ctx.accounts.admin_config.as_ref().unwrap().to_account_info(),
                &ctx.accounts.admin_control_program.as_ref().unwrap().to_account_info(),
            )?;
            
            // Преобразуем из базисных пунктов (1/100 процента) в проценты
            fee_basis_points as u64 / 100
        } else {
            1 // По умолчанию 1%
        };
        
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
            b"exchange_data".as_ref(),
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

    /// Инициализация данных обмена
    pub fn initialize_exchange(
        ctx: Context<InitializeExchange>,
    ) -> Result<()> {
        let exchange_data = &mut ctx.accounts.exchange_data;
        exchange_data.authority = ctx.accounts.authority.key();
        exchange_data.total_volume_traded = 0;
        exchange_data.total_fees_collected = 0;
        exchange_data.last_update_time = Clock::get()?.unix_timestamp;
        exchange_data.bump = ctx.bumps.exchange_data;
        
        msg!("Данные обмена успешно инициализированы");
        Ok(())
    }

    /// Инициализация торговой биржи
    pub fn initialize_trading_exchange(
        ctx: Context<InitializeTradingExchange>,
        n_dollar_mint: Pubkey,
    ) -> Result<()> {
        let trading_exchange = &mut ctx.accounts.trading_exchange;
        trading_exchange.authority = ctx.accounts.authority.key();
        trading_exchange.n_dollar_mint = n_dollar_mint;
        trading_exchange.bump = ctx.bumps.trading_exchange;
        
        // Регистрация в admin_control, если admin_config передан
        if ctx.accounts.admin_config.is_some() && ctx.accounts.admin_control_program.is_some() {
            // Инициализируем Trading Exchange в admin_control через CPI
            let admin_cpi_accounts = admin_cpi::account::AuthorizeProgram {
                authority: ctx.accounts.authority.to_account_info(),
                admin_config: ctx.accounts.admin_config.as_ref().unwrap().to_account_info(),
            };
            
            // Авторизуем текущую программу в admin_control
            let program_id = crate::ID;
            admin_cpi::direct_cpi::authorize_program(
                ctx.accounts.admin_control_program.as_ref().unwrap().to_account_info(),
                admin_cpi_accounts,
                program_id,
            )?;
            
            msg!("Trading Exchange зарегистрирован в admin_control");
        }
        
        msg!("Trading Exchange инициализирован");
        Ok(())
    }

    /// Создание контрольных аккаунтов программы через PDA
    pub fn initialize_control_accounts(
        ctx: Context<InitializeControlAccounts>,
    ) -> Result<()> {
        let control_state = &mut ctx.accounts.control_state;
        control_state.authority = ctx.accounts.authority.key();
        control_state.is_active = true;
        control_state.last_updated = Clock::get()?.unix_timestamp;
        control_state.bump = ctx.bumps.control_state;
        
        msg!("Контрольные аккаунты программы trading-exchange успешно инициализированы");
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
        
        // Проверяем активность контрольного аккаунта
        require!(
            ctx.accounts.control_state.is_active,
            TradingError::ProgramNotActive
        );
        
        // Получаем данные из контекста
        let liquidity_manager_program_id = ctx.accounts.liquidity_manager_program.key();
        
        // Вызываем CPI к Liquidity Manager
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: liquidity_manager_program_id,
            accounts: vec![
                AccountMeta::new(ctx.accounts.user.key(), true),  // user (signer)
                AccountMeta::new(ctx.accounts.liquidity_manager.key(), false),  // liquidity_manager
                AccountMeta::new(ctx.accounts.user_ndollar_account.key(), false),  // user_ndollar_account
                AccountMeta::new(ctx.accounts.pool_sol_account.key(), false),  // pool_sol_account
                AccountMeta::new(ctx.accounts.pool_ndollar_account.key(), false),  // pool_ndollar_account
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),  // token_program
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),  // system_program
            ],
            data: {
                // Подготовка данных инструкции с правильным дискриминатором
                let disc = anchor_lang::solana_program::hash::hash("global:swap_ndollar_to_sol".as_bytes());
                let swap_discriminator = disc.to_bytes()[..8].to_vec();
                
                // Расширяем вектор байтов данными аргументов
                let mut ix_data = swap_discriminator;
                ix_data.extend_from_slice(&ndollar_amount.to_le_bytes());
                ix_data
            },
        };
        
        // Выполняем инструкцию
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.liquidity_manager.to_account_info(),
                ctx.accounts.user_ndollar_account.to_account_info(),
                ctx.accounts.pool_sol_account.to_account_info(),
                ctx.accounts.pool_ndollar_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
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
            ctx.accounts.user.lamports() >= sol_amount + 1_000_000, // +0.001 SOL для комиссии
            TradingError::InsufficientBalance
        );
        
        // Проверяем активность контрольного аккаунта
        require!(
            ctx.accounts.control_state.is_active,
            TradingError::ProgramNotActive
        );
        
        // Получаем данные из контекста
        let liquidity_manager_program_id = ctx.accounts.liquidity_manager_program.key();
        
        // Вызываем CPI к Liquidity Manager
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: liquidity_manager_program_id,
            accounts: vec![
                AccountMeta::new(ctx.accounts.user.key(), true),  // user (signer)
                AccountMeta::new(ctx.accounts.liquidity_manager.key(), false),  // liquidity_manager
                AccountMeta::new(ctx.accounts.user_ndollar_account.key(), false),  // user_ndollar_account
                AccountMeta::new(ctx.accounts.pool_sol_account.key(), false),  // pool_sol_account
                AccountMeta::new(ctx.accounts.pool_ndollar_account.key(), false),  // pool_ndollar_account
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),  // token_program
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),  // system_program
            ],
            data: {
                // Подготовка данных инструкции с правильным дискриминатором
                let disc = anchor_lang::solana_program::hash::hash("global:swap_sol_to_ndollar".as_bytes());
                let swap_discriminator = disc.to_bytes()[..8].to_vec();
                
                // Расширяем вектор байтов данными аргументов
                let mut ix_data = swap_discriminator;
                ix_data.extend_from_slice(&sol_amount.to_le_bytes());
                ix_data
            },
        };
        
        // Выполняем инструкцию
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.liquidity_manager.to_account_info(),
                ctx.accounts.user_ndollar_account.to_account_info(),
                ctx.accounts.pool_sol_account.to_account_info(),
                ctx.accounts.pool_ndollar_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        msg!("Обмен SOL на N-Dollar выполнен успешно");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeExchange<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"exchange_data".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + ExchangeData::SPACE
    )]
    pub exchange_data: Account<'info, ExchangeData>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeTradingExchange<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"trading_exchange".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + TradingExchange::SPACE
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    /// Опциональный admin_config аккаунт из программы admin_control для регистрации
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова
    #[account(mut)]
    pub admin_config: Option<AccountInfo<'info>>,
    
    /// Опциональная программа admin_control для CPI вызовов
    /// CHECK: ID программы admin_control
    pub admin_control_program: Option<AccountInfo<'info>>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeControlAccounts<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"control_state".as_ref(), authority.key().as_ref()],
        bump,
        space = 8 + ControlState::SPACE
    )]
    pub control_state: Account<'info, ControlState>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SwapTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"exchange_data".as_ref(), exchange_data.authority.as_ref()],
        bump = exchange_data.bump
    )]
    pub exchange_data: Account<'info, ExchangeData>,
    
    #[account(
        seeds = [b"control_state".as_ref(), exchange_data.authority.as_ref()],
        bump,
        constraint = control_state.is_active == true
    )]
    pub control_state: Account<'info, ControlState>,
    
    pub from_mint: Account<'info, Mint>,
    pub to_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_from_account.mint == from_mint.key(),
        constraint = user_from_account.owner == user.key()
    )]
    pub user_from_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_to_account.mint == to_mint.key(),
        constraint = user_to_account.owner == user.key()
    )]
    pub user_to_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidity_from_account.mint == from_mint.key()
    )]
    pub liquidity_from_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidity_to_account.mint == to_mint.key()
    )]
    pub liquidity_to_account: Account<'info, TokenAccount>,
    
    /// Аккаунт бондинговой кривой для входящего токена
    #[account(
        seeds = [b"bonding_curve".as_ref(), from_mint.key().as_ref()],
        bump,
        seeds::program = bonding_curve_program.key()
    )]
    pub from_bonding_curve: Account<'info, BondingCurve>,
    
    /// Аккаунт бондинговой кривой для исходящего токена
    #[account(
        seeds = [b"bonding_curve".as_ref(), to_mint.key().as_ref()],
        bump,
        seeds::program = bonding_curve_program.key()
    )]
    pub to_bonding_curve: Account<'info, BondingCurve>,
    
    /// Опциональный admin_config аккаунт из программы admin_control для проверки авторизации
    /// CHECK: Этот аккаунт проверяется внутри CPI вызова
    pub admin_config: Option<AccountInfo<'info>>,
    
    /// Опциональная программа admin_control для CPI вызовов
    /// CHECK: ID программы admin_control
    pub admin_control_program: Option<AccountInfo<'info>>,
    
    /// Программа бондинговой кривой
    pub bonding_curve_program: Program<'info, bonding_curve::program::BondingCurve>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyNDollar<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"trading_exchange".as_ref(), trading_exchange.authority.as_ref()],
        bump = trading_exchange.bump
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == trading_exchange.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: AccountInfo<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellNDollar<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"trading_exchange".as_ref(), trading_exchange.authority.as_ref()],
        bump = trading_exchange.bump
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == trading_exchange.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: AccountInfo<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SwapNDollarToSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"trading_exchange".as_ref(), trading_exchange.authority.as_ref()],
        bump = trading_exchange.bump
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    #[account(
        seeds = [b"control_state".as_ref(), trading_exchange.authority.as_ref()],
        bump,
        constraint = control_state.is_active == true
    )]
    pub control_state: Account<'info, ControlState>,
    
    #[account(
        mut,
        constraint = user_ndollar_account.mint == trading_exchange.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: AccountInfo<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(sol_amount: u64)]
pub struct SwapSolToNDollar<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"trading_exchange".as_ref(), trading_exchange.authority.as_ref()],
        bump = trading_exchange.bump
    )]
    pub trading_exchange: Account<'info, TradingExchange>,
    
    #[account(
        seeds = [b"control_state".as_ref(), trading_exchange.authority.as_ref()],
        bump,
        constraint = control_state.is_active == true
    )]
    pub control_state: Account<'info, ControlState>,
    
    #[account(
        mut,
        constraint = user.lamports() >= sol_amount + 1_000_000, // +0.001 SOL для комиссии
        constraint = user_ndollar_account.mint == trading_exchange.n_dollar_mint,
        constraint = user_ndollar_account.owner == user.key()
    )]
    pub user_ndollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: Аккаунт менеджера ликвидности
    #[account(mut)]
    pub liquidity_manager: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: AccountInfo<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: AccountInfo<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct ExchangeData {
    pub authority: Pubkey,
    pub total_volume_traded: u64,
    pub total_fees_collected: u64,
    pub last_update_time: i64,
    pub bump: u8,
}

impl ExchangeData {
    pub const SPACE: usize = 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct TradingExchange {
    pub authority: Pubkey,
    pub n_dollar_mint: Pubkey,
    pub bump: u8,
}

impl TradingExchange {
    pub const SPACE: usize = 32 + 32 + 1;
}

#[account]
pub struct ControlState {
    pub authority: Pubkey,
    pub is_active: bool,
    pub last_updated: i64,
    pub bump: u8,
}

impl ControlState {
    pub const SPACE: usize = 32 + 1 + 8 + 1;
}

#[error_code]
pub enum TradingError {
    #[msg("Недостаточно токенов на балансе")]
    InsufficientTokenBalance,
    #[msg("Недостаточно ликвидности в пуле")]
    InsufficientLiquidity,
    #[msg("Арифметическая ошибка при расчете")]
    ArithmeticError,
    #[msg("Неверный идентификатор программы Liquidity Manager")]
    InvalidLiquidityManagerProgram,
    #[msg("Недостаточность баланса")]
    InsufficientBalance,
    #[msg("Неверная бондинговая кривая для токена")]
    InvalidBondingCurve,
    #[msg("Неверная пара токенов для обмена")]
    InvalidTokenPair,
    #[msg("Неверные параметры для расчета")]
    InvalidParameters,
    #[msg("Результат расчета равен нулю")]
    ZeroOutput,
    #[msg("Программа не активна")]
    ProgramNotActive,
    #[msg("Неавторизованный доступ")]
    UnauthorizedAccess,
}

// Вспомогательные функции для расчетов с бондинговыми кривыми

/// Рассчитывает стоимость токенов в N-Dollar на основе бондинговой кривой
fn calculate_token_ndollar_value(
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
    if token_amount < total_supply / 1000 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        let ndollar_amount = token_amount.checked_mul(current_price)
            .ok_or(TradingError::ArithmeticError)?;
            
        return Ok(ndollar_amount);
    }
    
    // Для более крупных сумм учитываем слиппедж (уменьшаем выход на 15% от линейной оценки)
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    let linear_estimate = token_amount.checked_mul(current_price)
        .ok_or(TradingError::ArithmeticError)?;
        
    let ndollar_amount = if token_amount > total_supply / 10 {
        // Для крупных продаж (более 10% от всех токенов) учитываем слиппедж
        linear_estimate.checked_mul(85).and_then(|v| v.checked_div(100))
            .ok_or(TradingError::ArithmeticError)?
    } else {
        linear_estimate
    };
    
    Ok(ndollar_amount)
}

/// Рассчитывает количество токенов, соответствующее сумме N-Dollar
fn calculate_ndollar_token_amount(
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
    if ndollar_amount < reserve_balance / 1000 {
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
        linear_estimate.checked_mul(9).and_then(|v| v.checked_div(10))
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
fn get_current_price(
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
