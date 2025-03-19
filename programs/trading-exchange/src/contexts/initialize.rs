use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeExchange<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [EXCHANGE_DATA_SEED, authority.key().as_ref()],
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
        seeds = [TRADING_EXCHANGE_SEED, authority.key().as_ref()],
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
        seeds = [CONTROL_STATE_SEED, authority.key().as_ref()],
        bump,
        space = 8 + ControlState::SPACE
    )]
    pub control_state: Account<'info, ControlState>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
