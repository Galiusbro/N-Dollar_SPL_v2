use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo, Burn};
use anchor_lang::solana_program::pubkey::Pubkey;
use admin_control::admin_cpi::{verify_program_authorization, get_fee_basis_points};

declare_id!("HgiiaxwngpLK7jS3hC5EYXz8JkgSpMcA1xdaRc7tCqTL");

// Модуль математических функций для бондинговой кривой
pub mod math_lib;
use math_lib::constants::*;

/// Вспомогательная функция для проверки авторизации программы через admin_control
fn verify_program_auth<'info>(
    admin_config: &AccountInfo<'info>,
    admin_control_program: &AccountInfo<'info>
) -> Result<()> {
    // Проверяем, что текущая программа авторизована
    let is_authorized = verify_program_authorization(
        admin_config,
        &crate::ID,
        admin_control_program,
    )?;
    
    require!(is_authorized, BondingCurveError::UnauthorizedAccess);
    Ok(())
}

#[program]
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
        
        let cpi_accounts = Burn {
            mint: ctx.accounts.coin_mint.to_account_info(),
            from: ctx.accounts.buyer_coin_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, token_amount)?;
        
        // Переводим N-Dollar продавцу
        let cpi_accounts = Transfer {
            from: ctx.accounts.liquidity_pool.to_account_info(),
            to: ctx.accounts.buyer_ndollar_account.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::transfer(cpi_ctx, ndollar_amount)?;
        
        // Обновляем состояние бондинговой кривой
        bonding_curve.total_supply_in_curve = bonding_curve.total_supply_in_curve
            .checked_sub(token_amount)
            .ok_or(BondingCurveError::ArithmeticError)?;
            
        bonding_curve.reserve_balance = bonding_curve.reserve_balance
            .checked_sub(ndollar_amount + fee_amount)
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

    /// Рассчитывает текущую цену токена и отправляет результат в логи
    pub fn calculate_price(ctx: Context<CalculatePrice>) -> Result<()> {
        let bonding_curve = &ctx.accounts.bonding_curve;
        
        // Если supply = 0, используем начальную цену
        if bonding_curve.total_supply_in_curve == 0 {
            msg!("Текущая цена (начальная): {} NDollar", bonding_curve.initial_price);
            return Ok(());
        }
        
        // Рассчитываем текущую цену
        let current_price = math_lib::get_current_price(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            bonding_curve.power,
        )?;
        
        msg!("Текущая цена: {} NDollar", current_price);
        msg!("Общий supply: {}, резерв: {}", bonding_curve.total_supply_in_curve, bonding_curve.reserve_balance);
        
        Ok(())
    }

    /// Симулирует покупку токенов, вычисляя примерное количество получаемых токенов
    pub fn simulate_buy(
        ctx: Context<CalculatePrice>, 
        ndollar_amount: u64
    ) -> Result<()> {
        let bonding_curve = &ctx.accounts.bonding_curve;
        
        // Рассчитываем комиссию
        let fee_amount = math_lib::calculate_fee(ndollar_amount, bonding_curve.fee_percent)?;
        let effective_amount = ndollar_amount - fee_amount;
        
        msg!("Сумма: {} NDollar, комиссия: {} NDollar", ndollar_amount, fee_amount);
        
        // Рассчитываем количество токенов
        let token_amount = math_lib::calculate_buy_amount(
            bonding_curve.total_supply_in_curve,
            bonding_curve.reserve_balance,
            effective_amount,
            bonding_curve.power,
            bonding_curve.initial_price,
        )?;
        
        // Рассчитываем прирост цены
        let current_price_before = if bonding_curve.total_supply_in_curve == 0 {
            bonding_curve.initial_price
        } else {
            math_lib::get_current_price(
                bonding_curve.total_supply_in_curve, 
                bonding_curve.reserve_balance, 
                bonding_curve.power
            )?
        };
        
        let new_total_supply = bonding_curve.total_supply_in_curve + token_amount;
        let new_reserve = bonding_curve.reserve_balance + ndollar_amount;
        
        let current_price_after = if new_total_supply == 0 {
            bonding_curve.initial_price
        } else {
            math_lib::get_current_price(
                new_total_supply, 
                new_reserve, 
                bonding_curve.power
            )?
        };
        
        let price_increase_percent = if current_price_before == 0 {
            0
        } else {
            ((current_price_after as f64 - current_price_before as f64) / current_price_before as f64 * 100.0) as u64
        };
        
        msg!("Вы получите примерно {} токенов", token_amount);
        msg!("Текущая цена: {} NDollar", current_price_before);
        msg!("Новая цена после покупки: {} NDollar (+{}%)", 
             current_price_after, price_increase_percent);
        
        Ok(())
    }
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
    
    /// Admin control аккаунт
    /// Этот аккаунт хранит информацию об авторизованных программах и настройках
    #[account(
        seeds = [b"admin_config".as_ref(), creator.key().as_ref()],
        bump,
        seeds::program = admin_control_program.key()
    )]
    pub admin_config: Account<'info, admin_control::AdminConfig>,
    
    /// Программа admin_control
    pub admin_control_program: Program<'info, admin_control::program::AdminControl>,
    
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
    
    /// Admin control аккаунт для проверки авторизации
    #[account(
        seeds = [b"admin_config".as_ref(), buyer.key().as_ref()],
        bump,
        seeds::program = admin_control_program.key()
    )]
    pub admin_config: Account<'info, admin_control::AdminConfig>,
    
    /// Программа admin_control
    pub admin_control_program: Program<'info, admin_control::program::AdminControl>,
    
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
    pub admin_control_program: Pubkey, // Программа admin_control для авторизации
    pub bump: u8,                     // Bump для PDA
}

impl BondingCurve {
    // Обновили размер с учетом добавления нового поля admin_control_program
    pub const SPACE: usize = 32 + 32 + 32 + 1 + 8 + 2 + 32 + 8 + 8 + 16 + 8 + 32 + 1;
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
    #[msg("Отсутствует необходимый аккаунт")]
    MissingAccount,
    #[msg("Неавторизованный доступ")]
    UnauthorizedAccess,
}