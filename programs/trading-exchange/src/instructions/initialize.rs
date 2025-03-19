use anchor_lang::prelude::*;
use admin_control::admin_cpi;
use crate::contexts::{InitializeExchange, InitializeTradingExchange, InitializeControlAccounts};

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
