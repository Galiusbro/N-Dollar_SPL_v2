use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, MintTo, Burn};
use anchor_lang::solana_program::pubkey::Pubkey;
use admin_control::admin_cpi::get_fee_basis_points;

use crate::math_lib;
use crate::math_lib::constants::*;
use crate::utils::verify_program_auth;
use crate::instructions::*;
use crate::errors::BondingCurveError;

pub mod bonding_curve {
    use super::*;

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

    /// Покупка токенов через бондинговую кривую, оплата в N-Dollar
    pub fn buy_token(
        ctx: Context<TradeToken>,
        ndollar_amount: u64,
    ) -> Result<()> {
        // Проверка на нулевое количество
        require!(ndollar_amount > 0, BondingCurveError::ZeroAmount);
        
        // Проверка на слишком большую сумму
        require!(
            ndollar_amount <= MAX_NDOLLAR_TRANSACTION,
            BondingCurveError::TransactionTooLarge
        );
        
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        
        // Проверяем авторизацию программы через admin_control
        let admin_config_info = ctx.accounts.admin_config.to_account_info();
        let admin_control_program = ctx.accounts.admin_control_program.to_account_info();
        verify_program_auth(&admin_config_info, &admin_control_program)?;
        
        // Рассчитываем комиссию
        let fee_amount = math_lib::calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
        let effective_amount = ndollar_amount.checked_sub(fee_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
        
        // Рассчитываем количество токенов к получению
        let token_amount = math_lib::calculate_buy_amount(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            effective_amount,
            bonding_curve.power,
            bonding_curve.initial_price,
        )?;
        
        // Проверяем, что рассчитанное количество токенов корректно
        require!(token_amount > 0, BondingCurveError::ZeroOutput);
        
        msg!("Покупка: {} токенов за {} NDollar", token_amount, ndollar_amount);
        msg!("Комиссия: {} NDollar", fee_amount);
        
        // Проверяем, что у пользователя достаточно N-Dollar для покупки
        require!(
            ctx.accounts.buyer_ndollar_account.amount >= ndollar_amount,
            BondingCurveError::InsufficientFunds
        );
        
        // Переводим N-Dollar в пул ликвидности
        let transfer_instruction = Transfer {
            from: ctx.accounts.buyer_ndollar_account.to_account_info(),
            to: ctx.accounts.liquidity_pool.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, ndollar_amount)?;
        
        // Минтим новые токены покупателю
        let seeds = &[
            b"bonding_curve".as_ref(),
            &bonding_curve.coin_mint.to_bytes(),
            &[bonding_curve.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.coin_mint.to_account_info(),
            to: ctx.accounts.buyer_coin_account.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, token_amount)?;
        
        // Обновляем состояние бондинговой кривой
        bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
            .checked_add(token_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        bonding_curve.reserve_balance = bonding_curve.reserve_balance
            .checked_add(ndollar_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Обновляем constant_product
        bonding_curve.constant_product = math_lib::calculate_constant_product(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            bonding_curve.power,
        )?;
        
        bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
        
        msg!("Токены успешно куплены. Новый supply: {}, резерв: {}", 
             bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
        Ok(())
    }

    /// Продажа токенов через бондинговую кривую, получение N-Dollar
    pub fn sell_token(
        ctx: Context<TradeToken>,
        token_amount: u64,
    ) -> Result<()> {
        // Проверка на нулевое количество
        require!(token_amount > 0, BondingCurveError::ZeroAmount);
        
        // Проверка на слишком большое количество токенов
        require!(
            token_amount <= MAX_TOKEN_TRANSACTION,
            BondingCurveError::TransactionTooLarge
        );
        
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        
        // Проверяем авторизацию программы через admin_control
        let admin_config_info = ctx.accounts.admin_config.to_account_info();
        let admin_control_program = ctx.accounts.admin_control_program.to_account_info();
        verify_program_auth(&admin_config_info, &admin_control_program)?;
        
        // Проверка наличия достаточного количества токенов у продавца
        require!(
            ctx.accounts.buyer_coin_account.amount >= token_amount,
            BondingCurveError::InsufficientTokens
        );
        
        // Проверка наличия достаточной ликвидности в пуле
        require!(
            bonding_curve.reserve_balance > 0,
            BondingCurveError::InsufficientLiquidity
        );
        
        // Рассчитываем количество N-Dollar к получению и комиссию
        let (ndollar_amount, fee_amount) = math_lib::calculate_sell_amount(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            token_amount,
            bonding_curve.power,
            bonding_curve.fee_percent,
        )?;
        
        // Проверяем, что рассчитанные суммы корректны
        require!(ndollar_amount > 0, BondingCurveError::ZeroOutput);
        
        msg!("Продажа: {} токенов за {} NDollar", token_amount, ndollar_amount);
        msg!("Комиссия: {} NDollar", fee_amount);
        
        // Проверяем, что в пуле ликвидности достаточно средств
        require!(
            ctx.accounts.liquidity_pool.amount >= ndollar_amount,
            BondingCurveError::InsufficientLiquidity
        );
        
        // Сжигаем токены
        let seeds = &[
            b"bonding_curve".as_ref(),
            &bonding_curve.coin_mint.to_bytes(),
            &[bonding_curve.bump],
        ];
        let signer = &[&seeds[..]];
        
        // Сначала переводим токены от пользователя на PDA
        let transfer_instruction = Transfer {
            from: ctx.accounts.buyer_coin_account.to_account_info(),
            to: ctx.accounts.liquidity_pool.to_account_info(), // временно используем liquidity_pool, просто чтобы принять токены
            authority: ctx.accounts.buyer.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, token_amount)?;
        
        // Сжигаем токены с authority = PDA бондинговой кривой
        let burn_instruction = Burn {
            mint: ctx.accounts.coin_mint.to_account_info(),
            from: ctx.accounts.liquidity_pool.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, burn_instruction, signer);
        
        token::burn(cpi_ctx, token_amount)?;
        
        // Отправляем N-Dollar пользователю
        let transfer_instruction = Transfer {
            from: ctx.accounts.liquidity_pool.to_account_info(),
            to: ctx.accounts.buyer_ndollar_account.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_instruction, signer);
        
        token::transfer(cpi_ctx, ndollar_amount)?;
        
        // Обновляем состояние бондинговой кривой
        bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
            .checked_sub(token_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Вычитаем из резерва сумму выплаты плюс комиссию
        let total_out = ndollar_amount.checked_add(fee_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        bonding_curve.reserve_balance = bonding_curve.reserve_balance
            .checked_sub(total_out)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Обновляем constant_product
        bonding_curve.constant_product = math_lib::calculate_constant_product(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            bonding_curve.power,
        )?;
        
        bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
        
        msg!("Токены успешно проданы. Новый supply: {}, резерв: {}", 
             bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
        Ok(())
    }

    /// Функция для расчета текущей цены токена и симуляции операций
    pub fn calculate_price(
        ctx: Context<CalculatePrice>,
        action: u8,  // 0 - только текущая цена, 1 - симуляция покупки, 2 - симуляция продажи
        amount: u64, // Количество NDollar для покупки или количество токенов для продажи
    ) -> Result<(u64, u64)> {
        let bonding_curve = &ctx.accounts.bonding_curve;
        
        // Получаем текущую цену
        let current_price = if bonding_curve.total_supply_in_curve == 0 {
            bonding_curve.initial_price
        } else {
            math_lib::get_current_price(
                bonding_curve.total_supply_in_curve,
                bonding_curve.reserve_balance,
                bonding_curve.power,
            )?
        };
        
        // Если требуется только текущая цена, возвращаем ее
        if action == 0 || amount == 0 {
            msg!("Текущая цена токена: {}", current_price);
            return Ok((current_price, 0));
        }
        
        // Симуляция покупки
        if action == 1 {
            // Расчет комиссии
            let fee_amount = math_lib::calculate_fee(amount, bonding_curve.fee_percent)?;
            let effective_amount = amount.checked_sub(fee_amount)
                .ok_or(BondingCurveError::ArithmeticError)?;
            
            // Рассчитываем количество токенов к получению
            let token_amount = math_lib::calculate_buy_amount(
                bonding_curve.total_supply_in_curve,
                bonding_curve.reserve_balance,
                effective_amount,
                bonding_curve.power,
                bonding_curve.initial_price,
            )?;
            
            msg!("Симуляция покупки: {} NDollar => {} токенов (комиссия: {} NDollar)", 
                 amount, token_amount, fee_amount);
            return Ok((current_price, token_amount));
        }
        
        // Симуляция продажи
        if action == 2 {
            // Проверяем, что продажа возможна
            if bonding_curve.total_supply_in_curve == 0 || bonding_curve.reserve_balance == 0 {
                return Err(BondingCurveError::InsufficientLiquidity.into());
            }
            
            // Рассчитываем количество N-Dollar к получению и комиссию
            let (ndollar_amount, fee_amount) = math_lib::calculate_sell_amount(
                bonding_curve.total_supply_in_curve,
                bonding_curve.reserve_balance,
                amount,
                bonding_curve.power,
                bonding_curve.fee_percent,
            )?;
            
            msg!("Симуляция продажи: {} токенов => {} NDollar (комиссия: {} NDollar)", 
                 amount, ndollar_amount, fee_amount);
            return Ok((current_price, ndollar_amount));
        }
        
        Err(BondingCurveError::InvalidParameter.into())
    }

    /// Закрытие бондинговой кривой
    pub fn close_bonding_curve(ctx: Context<CloseBondingCurve>) -> Result<()> {
        // Проверяем авторизацию программы через admin_control
        let admin_config_info = ctx.accounts.admin_config.to_account_info();
        let admin_control_program = ctx.accounts.admin_control_program.to_account_info();
        verify_program_auth(&admin_config_info, &admin_control_program)?;
        
        // Проверяем, что бондинговая кривая пуста
        let bonding_curve = &ctx.accounts.bonding_curve;
        require!(
            bonding_curve.total_supply_in_curve == 0,
            BondingCurveError::InvalidParameter
        );
        require!(
            bonding_curve.reserve_balance == 0, 
            BondingCurveError::InvalidParameter
        );
        
        msg!("Бондинговая кривая успешно закрыта");
        Ok(())
    }
} 