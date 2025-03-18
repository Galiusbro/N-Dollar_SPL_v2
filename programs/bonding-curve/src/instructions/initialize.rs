use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use admin_control::admin_cpi::get_fee_basis_points;
use crate::constants::bonding_curve::*;
use crate::contexts::InitializeBondingCurve;
use crate::errors::BondingCurveError;
use crate::instructions::utils::verify_program_auth;

/// Инициализация бондинговой кривой для нового мемкоина
pub fn initialize_bonding_curve(
    ctx: Context<InitializeBondingCurve>,
    coin_mint: Pubkey,
    initial_price: u64,
    power_opt: Option<u8>,
    fee_percent_opt: Option<u16>,
) -> Result<()> {
    // Проверка авторизации через admin_control
    let admin_config_info = &ctx.accounts.admin_config.to_account_info();
    let admin_control_program = &ctx.accounts.admin_control_program.to_account_info();
    
    verify_program_auth(admin_config_info, admin_control_program)?;
    
    // Получаем значения с использованием значений по умолчанию, если не указаны
    let power = power_opt.unwrap_or(DEFAULT_POWER);
    
    // Если fee_percent не указан, берем из admin_control
    let fee_percent = match fee_percent_opt {
        Some(fee) => fee,
        None => {
            // Получаем значение комиссии из admin_control
            get_fee_basis_points(admin_config_info, admin_control_program)?
        }
    };
    
    // Проверка параметров
    require!(
        power >= 1 && power <= 10,
        BondingCurveError::InvalidParameter
    );
    require!(
        initial_price >= MIN_INITIAL_PRICE,
        BondingCurveError::InvalidParameter
    );
    require!(
        fee_percent <= MAX_FEE_PERCENT,
        BondingCurveError::InvalidParameter
    );
    
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    bonding_curve.coin_mint = coin_mint;
    bonding_curve.ndollar_mint = ctx.accounts.ndollar_mint.key();
    bonding_curve.creator = ctx.accounts.creator.key();
    bonding_curve.power = power;
    bonding_curve.initial_price = initial_price;
    bonding_curve.fee_percent = fee_percent;
    bonding_curve.liquidity_pool = ctx.accounts.liquidity_pool.key();
    bonding_curve.total_supply_in_curve = 0;
    bonding_curve.reserve_balance = 0;
    bonding_curve.constant_product = 0;
    bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
    bonding_curve.bump = ctx.bumps.bonding_curve;
    bonding_curve.admin_control_program = admin_control_program.key();
    
    msg!("Бондинговая кривая успешно инициализирована для монеты: {}", coin_mint);
    msg!("Параметры: power={}, начальная цена={}, комиссия={}BP", power, initial_price, fee_percent);
    Ok(())
}