use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::pubkey::Pubkey;

declare_id!("7i7EpxhmCxmDhBvcTNFVXqq2SRQNt7HG98ANxRdcF6Dh");

// Идентификатор программы Liquidity Manager
const LIQUIDITY_MANAGER_ID: &str = "4uP4rbMsgqMf9GKSAbbiDKKLJ1a2Rp4SEYE5jdhiVYLU";

#[program]
pub mod trading_exchange {
    use super::*;

    /// Свап между различными токенами
    pub fn swap_tokens(
        ctx: Context<SwapTokens>,
        amount_in: u64,
    ) -> Result<()> {
        // Получаем информацию о токенах
        let _from_mint = ctx.accounts.from_mint.key();
        let _to_mint = ctx.accounts.to_mint.key();
        let user = &ctx.accounts.user;
        
        // Проверка, что у пользователя достаточно токенов для обмена
        require!(
            ctx.accounts.user_from_account.amount >= amount_in,
            TradingError::InsufficientTokenBalance
        );
        
        // Определяем курс обмена и комиссию
        // В реальном проекте это должно быть определено путем запроса к AMM или другому механизму ценообразования
        // Для простоты используем фиксированный курс 1:1 и комиссию 1%
        let fee_percentage = 1; // 1%
        let fee_amount = amount_in.checked_mul(fee_percentage)
            .and_then(|v| v.checked_div(100))
            .ok_or(TradingError::ArithmeticError)?;
        
        let net_amount = amount_in.checked_sub(fee_amount)
            .ok_or(TradingError::ArithmeticError)?;
        
        // Здесь должна быть логика определения курса обмена
        // Для простоты используем 1:1
        let amount_out = net_amount;
        
        // Проверка, что в пуле ликвидности достаточно токенов для обмена
        require!(
            ctx.accounts.liquidity_to_account.amount >= amount_out,
            TradingError::InsufficientLiquidity
        );
        
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

    /// Покупка N-Dollar за SOL через Liquidity Manager
    pub fn buy_n_dollar(
        ctx: Context<BuyNDollar>,
        sol_amount: u64
    ) -> Result<()> {
        // Получаем данные из контекста
        let liquidity_manager_program_id = ctx.accounts.liquidity_manager_program.key();
        
        // Проверяем, что программа - это действительно Liquidity Manager
        require!(
            liquidity_manager_program_id.to_string() == LIQUIDITY_MANAGER_ID,
            TradingError::InvalidLiquidityManagerProgram
        );
        
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
        
        invoke(
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
        
        msg!("Успешный вызов функции покупки N-Dollar через Liquidity Manager");
        Ok(())
    }
    
    /// Продажа N-Dollar за SOL через Liquidity Manager
    pub fn sell_n_dollar(
        ctx: Context<SellNDollar>,
        ndollar_amount: u64
    ) -> Result<()> {
        // Получаем данные из контекста
        let liquidity_manager_program_id = ctx.accounts.liquidity_manager_program.key();
        
        // Проверяем, что программа - это действительно Liquidity Manager
        require!(
            liquidity_manager_program_id.to_string() == LIQUIDITY_MANAGER_ID,
            TradingError::InvalidLiquidityManagerProgram
        );
        
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
        
        invoke(
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
        
        msg!("Успешный вызов функции продажи N-Dollar через Liquidity Manager");
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
        
        msg!("Торговая биржа успешно инициализирована");
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
    pub liquidity_manager: UncheckedAccount<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: UncheckedAccount<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: UncheckedAccount<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: UncheckedAccount<'info>,
    
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
    pub liquidity_manager: UncheckedAccount<'info>,
    
    /// CHECK: Аккаунт для хранения SOL
    #[account(mut)]
    pub pool_sol_account: UncheckedAccount<'info>,
    
    /// CHECK: Аккаунт для хранения N-Dollar в пуле
    #[account(mut)]
    pub pool_ndollar_account: UncheckedAccount<'info>,
    
    /// CHECK: Программа менеджера ликвидности
    pub liquidity_manager_program: UncheckedAccount<'info>,
    
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
}
