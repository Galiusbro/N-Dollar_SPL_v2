use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo, Burn};
use anchor_lang::solana_program::pubkey::Pubkey;

declare_id!("HgiiaxwngpLK7jS3hC5EYXz8JkgSpMcA1xdaRc7tCqTL");

// #[program]
// pub mod bonding_curve {
//     use super::*;

//     /// Инициализация бондинговой кривой для нового мемкоина
//     pub fn initialize_bonding_curve(
//         ctx: Context<InitializeBondingCurve>,
//         coin_mint: Pubkey,
//         initial_price: u64,
//         power: u8,
//         fee_percent: u16,
//     ) -> Result<()> {
//         // Проверка параметров
//         require!(
//             power >= 1 && power <= 10,
//             BondingCurveError::InvalidParameter
//         );
//         require!(
//             initial_price > 0,
//             BondingCurveError::InvalidParameter
//         );
//         require!(
//             fee_percent <= 1000, // Максимум 10%
//             BondingCurveError::InvalidParameter
//         );
        
//         let bonding_curve = &mut ctx.accounts.bonding_curve;
//         bonding_curve.coin_mint = coin_mint;
//         bonding_curve.ndollar_mint = ctx.accounts.ndollar_mint.key();
//         bonding_curve.creator = ctx.accounts.creator.key();
//         bonding_curve.power = power;
//         bonding_curve.initial_price = initial_price;
//         bonding_curve.fee_percent = fee_percent;
//         bonding_curve.liquidity_pool = ctx.accounts.liquidity_pool.key();
//         bonding_curve.total_supply_in_curve = 0;
//         bonding_curve.reserve_balance = 0;
//         bonding_curve.constant_product = 0;
//         bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
//         bonding_curve.bump = ctx.bumps.bonding_curve;
        
//         msg!("Бондинговая кривая успешно инициализирована для монеты: {}", coin_mint);
//         msg!("Параметры: power={}, начальная цена={}, комиссия={}BP", power, initial_price, fee_percent);
//         Ok(())
//     }

//     /// Покупка токенов через бондинговую кривую, оплата в N-Dollar
//     pub fn buy_token(
//         ctx: Context<TradeToken>,
//         ndollar_amount: u64,
//     ) -> Result<()> {
//         // Проверка на нулевое количество
//         require!(ndollar_amount > 0, BondingCurveError::ZeroAmount);
        
//         // Проверка на слишком большую сумму
//         require!(
//             ndollar_amount <= 1_000_000_000 * 10u64.pow(9), // Макс 1000 N-Dollar
//             BondingCurveError::TransactionTooLarge
//         );
        
//         let bonding_curve = &mut ctx.accounts.bonding_curve;
        
//         // Рассчитываем комиссию
//         let fee_amount = calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
//         let effective_amount = ndollar_amount.checked_sub(fee_amount)
//             .ok_or(BondingCurveError::ArithmeticError)?;
        
//         // Рассчитываем количество токенов к получению
//         let token_amount = calculate_buy_amount(
//             bonding_curve.total_supply_in_curve,
//             bonding_curve.reserve_balance,
//             effective_amount,
//             bonding_curve.power,
//             bonding_curve.initial_price,
//         )?;
        
//         // Проверяем, что рассчитанное количество токенов корректно
//         require!(token_amount > 0, BondingCurveError::ZeroOutput);
        
//         msg!("Покупка: {} токенов за {} NDollar", token_amount, ndollar_amount);
//         msg!("Комиссия: {} NDollar", fee_amount);
        
//         // Проверяем, что у пользователя достаточно N-Dollar для покупки
//         require!(
//             ctx.accounts.buyer_ndollar_account.amount >= ndollar_amount,
//             BondingCurveError::InsufficientFunds
//         );
        
//         // Переводим N-Dollar в пул ликвидности
//         let transfer_instruction = Transfer {
//             from: ctx.accounts.buyer_ndollar_account.to_account_info(),
//             to: ctx.accounts.liquidity_pool.to_account_info(),
//             authority: ctx.accounts.buyer.to_account_info(),
//         };
        
//         let cpi_ctx = CpiContext::new(
//             ctx.accounts.token_program.to_account_info(),
//             transfer_instruction,
//         );
        
//         token::transfer(cpi_ctx, ndollar_amount)?;
        
//         // Минтим новые токены покупателю
//         let seeds = &[
//             b"bonding_curve".as_ref(),
//             &bonding_curve.coin_mint.to_bytes(),
//             &[bonding_curve.bump],
//         ];
//         let signer = &[&seeds[..]];
        
//         let cpi_accounts = MintTo {
//             mint: ctx.accounts.coin_mint.to_account_info(),
//             to: ctx.accounts.buyer_coin_account.to_account_info(),
//             authority: bonding_curve.to_account_info(),
//         };
        
//         let cpi_program = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
//         token::mint_to(cpi_ctx, token_amount)?;
        
//         // Обновляем состояние бондинговой кривой
//         bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
//             .checked_add(token_amount)
//             .ok_or(BondingCurveError::ArithmeticError)?;
            
//         bonding_curve.reserve_balance = bonding_curve.reserve_balance
//             .checked_add(ndollar_amount)
//             .ok_or(BondingCurveError::ArithmeticError)?;
            
//         // Обновляем constant_product
//         bonding_curve.constant_product = calculate_constant_product(
//             bonding_curve.total_supply_in_curve,
//             bonding_curve.reserve_balance,
//             bonding_curve.power,
//         )?;
        
//         bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
        
//         msg!("Токены успешно куплены. Новый supply: {}, резерв: {}", 
//              bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
//         Ok(())
//     }

//     /// Продажа токенов через бондинговую кривую, получение N-Dollar
//     pub fn sell_token(
//         ctx: Context<TradeToken>,
//         token_amount: u64,
//     ) -> Result<()> {
//         // Проверка на нулевое количество
//         require!(token_amount > 0, BondingCurveError::ZeroAmount);
        
//         // Проверка на слишком большое количество токенов
//         require!(
//             token_amount <= 1_000_000_000, // Разумное ограничение
//             BondingCurveError::TransactionTooLarge
//         );
        
//         let bonding_curve = &mut ctx.accounts.bonding_curve;
        
//         // Проверка наличия достаточного количества токенов у продавца
//         require!(
//             ctx.accounts.buyer_coin_account.amount >= token_amount,
//             BondingCurveError::InsufficientTokens
//         );
        
//         // Проверка, что в контракте достаточно токенов для вычитания
//         require!(
//             bonding_curve.total_supply_in_curve >= token_amount,
//             BondingCurveError::InsufficientTokens
//         );
        
//         // Рассчитываем сумму N-Dollar к получению и комиссию
//         let (ndollar_amount, fee_amount) = calculate_sell_amount(
//             bonding_curve.total_supply_in_curve,
//             bonding_curve.reserve_balance,
//             token_amount,
//             bonding_curve.power,
//             bonding_curve.fee_percent,
//         )?;
        
//         // Проверяем, что рассчитанная сумма корректна
//         require!(ndollar_amount > 0, BondingCurveError::ZeroOutput);
        
//         // Проверяем, что в пуле достаточно ликвидности
//         require!(
//             ctx.accounts.liquidity_pool.amount >= ndollar_amount,
//             BondingCurveError::InsufficientLiquidity
//         );
        
//         msg!("Продажа: {} NDollar за {} токенов", ndollar_amount, token_amount);
//         msg!("Комиссия: {} NDollar", fee_amount);
        
//         // Сжигаем проданные токены
//         let cpi_accounts = Burn {
//             mint: ctx.accounts.coin_mint.to_account_info(),
//             from: ctx.accounts.buyer_coin_account.to_account_info(),
//             authority: ctx.accounts.buyer.to_account_info(),
//         };
        
//         let cpi_program = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
//         token::burn(cpi_ctx, token_amount)?;
        
//         // Отправляем N-Dollar продавцу из пула ликвидности
//         let seeds = &[
//             b"bonding_curve".as_ref(),
//             &bonding_curve.coin_mint.to_bytes(),
//             &[bonding_curve.bump],
//         ];
//         let signer = &[&seeds[..]];
        
//         let transfer_instruction = Transfer {
//             from: ctx.accounts.liquidity_pool.to_account_info(),
//             to: ctx.accounts.buyer_ndollar_account.to_account_info(),
//             authority: bonding_curve.to_account_info(),
//         };
        
//         let cpi_ctx = CpiContext::new_with_signer(
//             ctx.accounts.token_program.to_account_info(),
//             transfer_instruction,
//             signer,
//         );
        
//         token::transfer(cpi_ctx, ndollar_amount)?;
        
//         // Обновляем состояние бондинговой кривой
//         bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
//             .checked_sub(token_amount)
//             .ok_or(BondingCurveError::ArithmeticError)?;
            
//         bonding_curve.reserve_balance = bonding_curve.reserve_balance
//             .checked_sub(ndollar_amount)
//             .ok_or(BondingCurveError::ArithmeticError)?;
            
//         // Обновляем constant_product
//         bonding_curve.constant_product = calculate_constant_product(
//             bonding_curve.total_supply_in_curve,
//             bonding_curve.reserve_balance,
//             bonding_curve.power,
//         )?;
        
//         bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
        
//         msg!("Токены успешно проданы. Новый supply: {}, резерв: {}", 
//              bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
//         Ok(())
//     }

//     /// Расчёт текущей цены токена
//     pub fn calculate_price(ctx: Context<CalculatePrice>) -> Result<()> {
//         let bonding_curve = &ctx.accounts.bonding_curve;
        
//         if bonding_curve.total_supply_in_curve == 0 {
//             msg!("Текущая цена токена: {}", bonding_curve.initial_price);
//             return Ok(());
//         }
        
//         let price = get_current_price(
//             bonding_curve.total_supply_in_curve,
//             bonding_curve.reserve_balance,
//             bonding_curve.power,
//         )?;
        
//         msg!("Текущая цена токена: {} NDollar", price);
//         Ok(())
//     }
    
//     /// Симуляция покупки для расчета цены с учетом слиппеджа
//     pub fn simulate_buy(
//         ctx: Context<CalculatePrice>, 
//         ndollar_amount: u64
//     ) -> Result<()> {
//         let bonding_curve = &ctx.accounts.bonding_curve;
        
//         // Рассчитываем комиссию
//         let fee_amount = calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
//         let effective_amount = ndollar_amount.checked_sub(fee_amount)
//             .ok_or(BondingCurveError::ArithmeticError)?;
        
//         // Рассчитываем количество токенов к получению
//         let token_amount = calculate_buy_amount(
//             bonding_curve.total_supply_in_curve,
//             bonding_curve.reserve_balance,
//             effective_amount,
//             bonding_curve.power,
//             bonding_curve.initial_price,
//         )?;
        
//         let current_price = if bonding_curve.total_supply_in_curve > 0 {
//             get_current_price(
//                 bonding_curve.total_supply_in_curve,
//                 bonding_curve.reserve_balance,
//                 bonding_curve.power,
//             )?
//         } else {
//             bonding_curve.initial_price
//         };
        
//         let avg_price = if token_amount > 0 {
//             effective_amount / token_amount
//         } else {
//             0
//         };
        
//         msg!("Симуляция покупки: {} NDollar", ndollar_amount);
//         msg!("Получите: {} токенов", token_amount);
//         msg!("Комиссия: {} NDollar", fee_amount);
//         msg!("Текущая цена: {} NDollar", current_price);
//         msg!("Средняя цена с учетом слиппеджа: {} NDollar", avg_price);
        
//         Ok(())
//     }
// }

/// Константы для бондинговой кривой
pub mod curve_constants {
    // Степенной показатель кривой по умолчанию (2 = квадратичная кривая)
    pub const DEFAULT_POWER: u8 = 2;
    
    // Комиссия по умолчанию в базисных пунктах (50 = 0.5%)
    pub const DEFAULT_FEE_PERCENT: u16 = 50;
    
    // Минимальная начальная цена
    pub const MIN_INITIAL_PRICE: u64 = 1;
    
    // Максимальный размер транзакции в единицах токена 
    pub const MAX_TOKEN_TRANSACTION: u64 = 1_000_000_000;
    
    // Максимальный размер транзакции в N-Dollar (1000 N-Dollar, с 9 знаками после запятой)
    pub const MAX_NDOLLAR_TRANSACTION: u64 = 1_000_000_000 * 10u64.pow(9);
    
    // Максимальное значение fee_percent (10% = 1000 базисных пунктов)
    pub const MAX_FEE_PERCENT: u16 = 1000;
    
    // Минимальное количество токенов, которое можно купить
    pub const MIN_TOKEN_AMOUNT: u64 = 1;
    
    // Минимальные значения для предотвращения числовых ошибок
    pub const MIN_SAFE_SUPPLY: u64 = 10;
}

#[program]
pub mod bonding_curve {
    use super::*;
    use curve_constants::*;

    /// Инициализация бондинговой кривой для нового мемкоина
    pub fn initialize_bonding_curve(
        ctx: Context<InitializeBondingCurve>,
        coin_mint: Pubkey,
        initial_price: u64,
        power_opt: Option<u8>,
        fee_percent_opt: Option<u16>,
    ) -> Result<()> {
        // Получаем значения с использованием значений по умолчанию, если не указаны
        let power = power_opt.unwrap_or(DEFAULT_POWER);
        let fee_percent = fee_percent_opt.unwrap_or(DEFAULT_FEE_PERCENT);
        
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
        
        // Рассчитываем комиссию
        let fee_amount = calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
        let effective_amount = ndollar_amount.checked_sub(fee_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
        
        msg!("Эффективная сумма после комиссии: {} NDollar", effective_amount);
        
        // Рассчитываем количество токенов к получению
        let token_amount = calculate_buy_amount(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            effective_amount,
            bonding_curve.power,
            bonding_curve.initial_price,
        )?;
        
        // Проверяем, что рассчитанное количество токенов корректно
        require!(token_amount >= MIN_TOKEN_AMOUNT, BondingCurveError::ZeroOutput);
        
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
        bonding_curve.constant_product = calculate_constant_product(
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
        
        // Проверка наличия достаточного количества токенов у продавца
        require!(
            ctx.accounts.buyer_coin_account.amount >= token_amount,
            BondingCurveError::InsufficientTokens
        );
        
        // Проверка, что в контракте достаточно токенов для вычитания
        require!(
            bonding_curve.total_supply_in_curve >= token_amount,
            BondingCurveError::InsufficientTokens
        );
        
        // Проверка, что после продажи останется достаточно токенов
        require!(
            bonding_curve.total_supply_in_curve.checked_sub(token_amount).unwrap_or(0) >= MIN_SAFE_SUPPLY ||
            token_amount == bonding_curve.total_supply_in_curve, // Разрешаем продать все токены
            BondingCurveError::AmountTooLarge
        );
        
        // Рассчитываем сумму N-Dollar к получению и комиссию
        let (ndollar_amount, fee_amount) = calculate_sell_amount(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            token_amount,
            bonding_curve.power,
            bonding_curve.fee_percent,
        )?;
        
        // Проверяем, что рассчитанная сумма корректна
        require!(ndollar_amount > 0, BondingCurveError::ZeroOutput);
        
        // Проверяем, что в пуле достаточно ликвидности
        require!(
            ctx.accounts.liquidity_pool.amount >= ndollar_amount,
            BondingCurveError::InsufficientLiquidity
        );
        
        msg!("Продажа: {} NDollar за {} токенов", ndollar_amount, token_amount);
        msg!("Комиссия: {} NDollar", fee_amount);
        
        // Сжигаем проданные токены
        let cpi_accounts = Burn {
            mint: ctx.accounts.coin_mint.to_account_info(),
            from: ctx.accounts.buyer_coin_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, token_amount)?;
        
        // Отправляем N-Dollar продавцу из пула ликвидности
        let seeds = &[
            b"bonding_curve".as_ref(),
            &bonding_curve.coin_mint.to_bytes(),
            &[bonding_curve.bump],
        ];
        let signer = &[&seeds[..]];
        
        let transfer_instruction = Transfer {
            from: ctx.accounts.liquidity_pool.to_account_info(),
            to: ctx.accounts.buyer_ndollar_account.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );
        
        token::transfer(cpi_ctx, ndollar_amount)?;
        
        // Обновляем состояние бондинговой кривой
        bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
            .checked_sub(token_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        bonding_curve.reserve_balance = bonding_curve.reserve_balance
            .checked_sub(ndollar_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Особый случай: если продали все токены, сбрасываем состояние
        if bonding_curve.total_supply_in_curve == 0 {
            bonding_curve.constant_product = 0;
            msg!("Все токены проданы, состояние бондинговой кривой сброшено");
        } else {
            // Обновляем constant_product
            bonding_curve.constant_product = calculate_constant_product(
                bonding_curve.total_supply_in_curve,
                bonding_curve.reserve_balance,
                bonding_curve.power,
            )?;
        }
        
        bonding_curve.last_update_time = Clock::get()?.unix_timestamp;
        
        msg!("Токены успешно проданы. Новый supply: {}, резерв: {}", 
             bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
        Ok(())
    }

    /// Расчёт текущей цены токена
    pub fn calculate_price(ctx: Context<CalculatePrice>) -> Result<()> {
        let bonding_curve = &ctx.accounts.bonding_curve;
        
        if bonding_curve.total_supply_in_curve == 0 {
            msg!("Текущая цена токена: {} NDollar (начальная цена)", bonding_curve.initial_price);
            return Ok(());
        }
        
        let price = get_current_price(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            bonding_curve.power,
        )?;
        
        msg!("Текущая цена токена: {} NDollar", price);
        Ok(())
    }
    
    /// Симуляция покупки для расчета цены с учетом слиппеджа
    pub fn simulate_buy(
        ctx: Context<CalculatePrice>, 
        ndollar_amount: u64
    ) -> Result<()> {
        let bonding_curve = &ctx.accounts.bonding_curve;
        
        // Проверка входных параметров
        if ndollar_amount == 0 {
            msg!("Симуляция для 0 NDollar: получите 0 токенов");
            return Ok(());
        }
        
        // Для больших сумм возвращаем приблизительный расчет
        if ndollar_amount > 1_000_000_000 {
            msg!("Сумма слишком большая для точного расчета. Используйте меньшие суммы.");
            return Ok(());
        }
        
        // Рассчитываем комиссию
        let fee_amount = calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
        let effective_amount = ndollar_amount.checked_sub(fee_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
        
        // Для первой покупки используем простое деление по начальной цене
        if bonding_curve.total_supply_in_curve == 0 {
            if bonding_curve.initial_price == 0 {
                msg!("Начальная цена не может быть нулевой");
                return Ok(());
            }
            
            let token_amount = effective_amount / bonding_curve.initial_price;
            
            msg!("Симуляция покупки: {} NDollar", ndollar_amount);
            msg!("Получите: {} токенов", token_amount);
            msg!("Комиссия: {} NDollar", fee_amount);
            msg!("Текущая цена: {} NDollar (начальная цена)", bonding_curve.initial_price);
            
            return Ok(());
        }
        
        // Для малых сумм используем линейную аппроксимацию
        if ndollar_amount < 1000 {
            let current_price = get_current_price(
                bonding_curve.total_supply_in_curve,
                bonding_curve.reserve_balance,
                bonding_curve.power,
            )?;
            
            let token_amount = if current_price > 0 { effective_amount / current_price } else { 0 };
            
            msg!("Симуляция покупки: {} NDollar", ndollar_amount);
            msg!("Получите: {} токенов", token_amount);
            msg!("Комиссия: {} NDollar", fee_amount);
            msg!("Текущая цена: {} NDollar", current_price);
            
            return Ok(());
        }
        
        // Для более точного расчета для больших сумм можно использовать упрощенную версию
        // без бинарного поиска, чтобы избежать арифметических ошибок
        let current_reserve = bonding_curve.reserve_balance;
        let current_supply = bonding_curve.total_supply_in_curve;
        let power = bonding_curve.power;
        
        // Используем текущую цену как основу для оценки
        let current_price = get_current_price(current_supply, current_reserve, power)?;
        let new_reserve = current_reserve.checked_add(effective_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Аппроксимируем новое предложение с учетом слиппеджа
        // Для квадратичной кривой (power=2) цена примерно пропорциональна sqrt(reserve)
        let token_amount_estimate = (effective_amount * 90) / (current_price * 100); // примерно 90% от линейной оценки
        
        msg!("Симуляция покупки: {} NDollar", ndollar_amount);
        msg!("Примерно получите: {} токенов (приблизительная оценка)", token_amount_estimate);
        msg!("Комиссия: {} NDollar", fee_amount);
        msg!("Текущая цена: {} NDollar", current_price);
        msg!("Слиппедж: примерно 10% (для больших сумм)");
        
        Ok(())
    }
}

/// Вычисляет constant product для кривой.
/// Формула: constant = reserve_balance * (total_supply^power)
fn calculate_constant_product(
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
) -> Result<u128> {
    // Используем u128 для промежуточных вычислений
    let supply_u128 = total_supply as u128;
    let reserve_u128 = reserve_balance as u128;
    
    // Если supply или reserve равны 0, constant_product тоже 0
    if supply_u128 == 0 || reserve_u128 == 0 {
        return Ok(0);
    }
    
    // Вычисляем total_supply^power с безопасной обработкой больших чисел
    let mut supply_pow = 1u128;
    for _ in 0..power {
        supply_pow = supply_pow.checked_mul(supply_u128)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Если результат становится слишком большим, сокращаем для предотвращения переполнения
        if supply_pow > (u64::MAX as u128) * (u64::MAX as u128) {
            return Err(BondingCurveError::ArithmeticError.into());
        }
    }
    
    // Вычисляем constant product
    let constant = reserve_u128.checked_mul(supply_pow)
        .ok_or(BondingCurveError::ArithmeticError)?;
    
    Ok(constant)
}

/// Рассчитывает текущую цену токена.
/// Формула: price = (reserve_balance * power) / total_supply
fn get_current_price(
    total_supply: u64,
    reserve_balance: u64,
    power: u8,
) -> Result<u64> {
    if total_supply == 0 {
        return Err(BondingCurveError::ZeroDivision.into());
    }
    
    // Используем u128 для безопасности вычислений
    let supply_u128 = total_supply as u128;
    let reserve_u128 = reserve_balance as u128;
    let power_u128 = power as u128;
    
    // Рассчитываем цену: (reserve * power) / supply
    let numerator = reserve_u128.checked_mul(power_u128)
        .ok_or(BondingCurveError::ArithmeticError)?;
    let price_u128 = numerator.checked_div(supply_u128)
        .ok_or(BondingCurveError::ZeroDivision)?;
    
    // Проверяем на переполнение при конвертации обратно в u64
    if price_u128 > u64::MAX as u128 {
        return Err(BondingCurveError::ArithmeticError.into());
    }
    
    Ok(price_u128 as u64)
}

/// Рассчитывает количество токенов, которое пользователь получит при покупке.
fn calculate_buy_amount(
    total_supply: u64,
    reserve_balance: u64,
    ndollar_amount: u64,
    power: u8,
    initial_price: u64,
) -> Result<u64> {
    // Обработка очень маленьких сумм - требуем минимальную сумму для покупки
    if ndollar_amount < 10 { // Например, менее 10 ламп (0.00000001 NDollar)
        return Err(BondingCurveError::AmountTooSmall.into());
    }
    
    // Если это первая покупка или резерв пуст, используем начальную цену
    if total_supply == 0 || reserve_balance == 0 {
        // Простое деление по начальной цене
        if initial_price == 0 {
            return Err(BondingCurveError::ZeroDivision.into());
        }
        
        // Проверяем минимальное количество токенов
        let amount = ndollar_amount / initial_price;
        if amount < 1 {
            return Err(BondingCurveError::ZeroOutput.into());
        }
        
        return Ok(amount);
    }
    
    // Для малых сумм (до 0.1% от резерва) используем линейную аппроксимацию
    if ndollar_amount < reserve_balance / 1000 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        if current_price == 0 {
            return Err(BondingCurveError::ZeroDivision.into());
        }
        
        // Проверяем минимальное количество токенов
        let amount = ndollar_amount / current_price;
        if amount < 1 {
            return Err(BondingCurveError::ZeroOutput.into());
        }
        
        return Ok(amount);
    }
    
    // Для более крупных сумм используем безопасный расчет с учетом слиппеджа
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    // Безопасная оценка с учетом слиппеджа (примерно 90% от линейной оценки для крупных сумм)
    let linear_estimate = if current_price > 0 { ndollar_amount / current_price } else { 0 };
    let amount = if linear_estimate > 1000 {
        // Для крупных покупок учитываем слиппедж
        linear_estimate * 9 / 10 // Примерно 90% от линейной оценки
    } else {
        linear_estimate
    };
    
    // Проверяем минимальное количество токенов
    if amount < 1 {
        return Err(BondingCurveError::ZeroOutput.into());
    }
    
    Ok(amount)
}

/// Рассчитывает количество NDollar и комиссию при продаже токенов.
fn calculate_sell_amount(
    total_supply: u64,
    reserve_balance: u64,
    token_amount: u64,
    power: u8,
    fee_percent: u16,
) -> Result<(u64, u64)> {
    // Проверка на нулевой supply или резерв
    if total_supply == 0 || reserve_balance == 0 {
        return Err(BondingCurveError::InsufficientLiquidity.into());
    }
    
    // Проверка, что мы не пытаемся продать больше, чем есть в обращении
    if token_amount >= total_supply {
        if token_amount == total_supply {
            // Особый случай: продаем все токены, возвращаем весь резерв за вычетом минимального остатка
            // для поддержания ликвидности
            let reserve_amount = reserve_balance.saturating_sub(1000); // Оставляем минимальный резерв
            let fee_amount = calculate_fee(reserve_amount, fee_percent)?;
            let final_amount = reserve_amount.checked_sub(fee_amount)
                .ok_or(BondingCurveError::ArithmeticError)?;
                
            return Ok((final_amount, fee_amount));
        } else {
            return Err(BondingCurveError::InsufficientTokens.into());
        }
    }
    
    // Для малых сумм (до 0.1% от total_supply) используем линейную аппроксимацию
    if token_amount < total_supply / 1000 {
        let current_price = get_current_price(total_supply, reserve_balance, power)?;
        let reserve_delta = token_amount.checked_mul(current_price)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        // Проверяем, что не пытаемся получить больше резерва, чем есть
        if reserve_delta > reserve_balance {
            return Err(BondingCurveError::InsufficientLiquidity.into());
        }
        
        // Рассчитываем комиссию
        let fee_amount = calculate_fee(reserve_delta, fee_percent)?;
        
        // Вычисляем итоговую сумму к получению
        let reserve_amount = reserve_delta.checked_sub(fee_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        return Ok((reserve_amount, fee_amount));
    }
    
    // Для более крупных продаж используем безопасный расчет с учетом слиппеджа
    let current_price = get_current_price(total_supply, reserve_balance, power)?;
    
    // Безопасная оценка: с учетом слиппеджа (примерно 85% от линейной оценки для крупных сумм)
    let linear_estimate = token_amount.checked_mul(current_price)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    let reserve_delta = if token_amount > total_supply / 10 {
        // Для крупных продаж (более 10% от всех токенов) учитываем слиппедж
        linear_estimate * 85 / 100 // 85% от линейной оценки
    } else {
        linear_estimate
    };
    
    // Проверяем, что не пытаемся получить больше резерва, чем есть
    if reserve_delta > reserve_balance {
        return Err(BondingCurveError::InsufficientLiquidity.into());
    }
    
    // Рассчитываем комиссию
    let fee_amount = calculate_fee(reserve_delta, fee_percent)?;
    
    // Вычисляем итоговую сумму к получению
    let reserve_amount = reserve_delta.checked_sub(fee_amount)
        .ok_or(BondingCurveError::ArithmeticError)?;
        
    // Проверяем, что сумма к получению ненулевая
    if reserve_amount == 0 {
        return Err(BondingCurveError::ZeroOutput.into());
    }
    
    Ok((reserve_amount, fee_amount))
}

/// Вычисляет комиссию на основе суммы и процента комиссии
fn calculate_fee(amount: u64, fee_percent: u16) -> Result<u64> {
    // fee_percent выражен в базисных пунктах (1% = 100)
    if fee_percent > 10000 { // Максимум 100%
        return Err(BondingCurveError::InvalidParameter.into());
    }
    
    let fee_amount = (amount as u128)
        .checked_mul(fee_percent as u128)
        .ok_or(BondingCurveError::ArithmeticError)?
        .checked_div(10000) // 100% = 10000 базисных пунктов
        .ok_or(BondingCurveError::ArithmeticError)?;
    
    if fee_amount > u64::MAX as u128 {
        return Err(BondingCurveError::ArithmeticError.into());
    }
    
    Ok(fee_amount as u64)
}

#[derive(Accounts)]
pub struct InitializeBondingCurve<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        seeds = [b"bonding_curve".as_ref(), coin_mint.key().as_ref()],
        bump,
        space = 8 + BondingCurve::SPACE
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(mut)]
    pub coin_mint: Account<'info, Mint>,
    
    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        token::mint = ndollar_mint
    )]
    pub liquidity_pool: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TradeToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"bonding_curve".as_ref(), coin_mint.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(
        mut,
        constraint = coin_mint.key() == bonding_curve.coin_mint
    )]
    pub coin_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = ndollar_mint.key() == bonding_curve.ndollar_mint
    )]
    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = buyer_coin_account.mint == coin_mint.key(),
        constraint = buyer_coin_account.owner == buyer.key()
    )]
    pub buyer_coin_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = buyer_ndollar_account.mint == ndollar_mint.key(),
        constraint = buyer_ndollar_account.owner == buyer.key()
    )]
    pub buyer_ndollar_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidity_pool.key() == bonding_curve.liquidity_pool
    )]
    pub liquidity_pool: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CalculatePrice<'info> {
    #[account(
        seeds = [b"bonding_curve".as_ref(), coin_mint.key().as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(
        constraint = coin_mint.key() == bonding_curve.coin_mint
    )]
    pub coin_mint: Account<'info, Mint>,
}

#[account]
pub struct BondingCurve {
    pub coin_mint: Pubkey,            // Мемкоин
    pub ndollar_mint: Pubkey,         // Резервная валюта (NDollar)
    pub creator: Pubkey,              // Создатель кривой
    pub power: u8,                    // Степенной показатель для кривой
    pub initial_price: u64,           // Начальная цена токена
    pub fee_percent: u16,             // Процент комиссии в базисных пунктах (1% = 100)
    pub liquidity_pool: Pubkey,       // Хранилище резервной валюты
    pub total_supply_in_curve: u64,   // Текущее предложение токена
    pub reserve_balance: u64,         // Баланс резервной валюты
    pub constant_product: u128,       // Константа произведения
    pub last_update_time: i64,        // Время последнего обновления
    pub bump: u8,                     // Bump для PDA
}

impl BondingCurve {
    pub const SPACE: usize = 32 + 32 + 32 + 1 + 8 + 2 + 32 + 8 + 8 + 16 + 8 + 1;
}

#[error_code]
pub enum BondingCurveError {
    #[msg("Недостаточно средств для покупки")]
    InsufficientFunds,
    #[msg("Недостаточно токенов для продажи")]
    InsufficientTokens,
    #[msg("Недостаточно ликвидности в пуле")]
    InsufficientLiquidity,
    #[msg("Арифметическая ошибка при расчете")]
    ArithmeticError,
    #[msg("Деление на ноль при расчетах")]
    ZeroDivision,
    #[msg("Количество токенов должно быть больше нуля")]
    ZeroAmount,
    #[msg("Рассчитано нулевое количество токенов")]
    ZeroOutput,
    #[msg("Превышен максимальный размер транзакции")]
    TransactionTooLarge,
    #[msg("Некорректный параметр")]
    InvalidParameter,
    #[msg("Ошибка при выполнении транзакции токена")]
    TokenTransferError,
    #[msg("Слишком маленькое количество токенов для операции")]
    AmountTooSmall,
    #[msg("Слишком большое количество токенов для операции")]
    AmountTooLarge,
}