use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use std::convert::TryInto;

// ID программы (замени на свой реальный ID)
declare_id!("GvFsepxBQ2q8xZ3PYYDooMdnMBzWQKkpKavzT7vM83rZ"); // Пример

// Константа для масштабирования вычислений (для дробных частей)
// 10^12 должно быть достаточно для цен и N-Dollar/Meme с 9 децималами
const PRECISION_FACTOR: u128 = 1_000_000_000_000; // 10^12
const _HALF_PRECISION_FACTOR: u128 = PRECISION_FACTOR / 2;

// Константы из спецификации
const START_PRICE_NUMERATOR: u128 = 5; // 0.00005 = 5 / 100000
const START_PRICE_DENOMINATOR: u128 = 100_000;
const TARGET_PRICE_NUMERATOR: u128 = 1; // 1 = 1 / 1
const TARGET_PRICE_DENOMINATOR: u128 = 1;
const TOTAL_BONDING_TOKENS: u64 = 30_000_000; // Без децималов

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize_curve(ctx: Context<InitializeCurve>) -> Result<()> {
        msg!("Initializing Bonding Curve...");

        let curve = &mut ctx.accounts.bonding_curve;
        let bump = ctx.bumps.bonding_curve;

        // Проверяем, что необходимые аккаунты переданы
        require!(ctx.accounts.mint.key() != Pubkey::default(), BondingCurveError::InvalidMintAccount);
        require!(ctx.accounts.n_dollar_mint.key() != Pubkey::default(), BondingCurveError::InvalidMintAccount);
        require!(ctx.accounts.bonding_curve_token_account.amount > 0, BondingCurveError::BondingAccountEmpty);

        // Получаем децималы
        let token_decimals = ctx.accounts.mint.decimals;
        let n_dollar_decimals = ctx.accounts.n_dollar_mint.decimals;
        msg!("Token Decimals: {}, N-Dollar Decimals: {}", token_decimals, n_dollar_decimals);

        // Рассчитываем общее предложение для кривой с учетом децималов
        let total_bonding_supply_with_decimals = TOTAL_BONDING_TOKENS
            .checked_mul(10u64.pow(token_decimals as u32))
            .ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("Total Bonding Supply (with decimals): {}", total_bonding_supply_with_decimals);

        // Проверяем, что на счету кривой ожидаемое количество токенов
        require!(
            ctx.accounts.bonding_curve_token_account.amount == total_bonding_supply_with_decimals,
            BondingCurveError::IncorrectInitialSupply
        );

        // Рассчитываем параметры линейной кривой P(x) = m*x + c
        // x - количество проданных токенов *без* децималов
        // c = start_price = 0.00005
        // m = (target_price - start_price) / total_bonding_tokens
        //    = (1 - 0.00005) / 30,000,000 = 0.99995 / 30,000,000

        // c (intercept) scaled
        let intercept_scaled = START_PRICE_NUMERATOR
            .checked_mul(PRECISION_FACTOR)
            .ok_or(BondingCurveError::CalculationOverflow)?
            .checked_div(START_PRICE_DENOMINATOR)
            .ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("Intercept (c) scaled: {}", intercept_scaled);


        // m (slope) scaled
        // target_price_scaled = 1 * PRECISION_FACTOR
        // start_price_scaled = intercept_scaled
        let target_price_scaled = TARGET_PRICE_NUMERATOR
            .checked_mul(PRECISION_FACTOR)
            .ok_or(BondingCurveError::CalculationOverflow)?
            .checked_div(TARGET_PRICE_DENOMINATOR)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        let price_diff_scaled = target_price_scaled
            .checked_sub(intercept_scaled)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        // Важно: TOTAL_BONDING_TOKENS не имеет децималов при расчете m
        let slope_scaled_numerator = price_diff_scaled;
        let slope_scaled_denominator = TOTAL_BONDING_TOKENS as u128; // m = numerator / denominator

        msg!("Slope (m) scaled: {} / {}", slope_scaled_numerator, slope_scaled_denominator);


        // Сохраняем состояние
        curve.mint = ctx.accounts.mint.key();
        curve.n_dollar_mint = ctx.accounts.n_dollar_mint.key();
        curve.bonding_curve_token_account = ctx.accounts.bonding_curve_token_account.key();
        curve.n_dollar_treasury = ctx.accounts.n_dollar_treasury.key();
        curve.initial_bonding_supply = total_bonding_supply_with_decimals;
        curve.slope_numerator = slope_scaled_numerator;
        curve.slope_denominator = slope_scaled_denominator;
        curve.intercept_scaled = intercept_scaled; // c * PRECISION
        curve.token_decimals = token_decimals;
        curve.n_dollar_decimals = n_dollar_decimals;
        curve.authority = ctx.accounts.authority.key(); // Кто инициализировал
        curve.is_initialized = true;
        curve.bump = bump;
        
        msg!("Bonding Curve Initialized for mint: {}", curve.mint);
        Ok(())
    }

    pub fn buy(ctx: Context<BuySell>, amount_tokens_to_buy: u64) -> Result<()> {
        msg!("Executing Buy...");
        let curve = &ctx.accounts.bonding_curve;
        require!(curve.is_initialized, BondingCurveError::NotInitialized);
        require!(amount_tokens_to_buy > 0, BondingCurveError::ZeroAmount);

        let current_supply = ctx.accounts.bonding_curve_token_account.amount;
        msg!("Current supply in curve: {}", current_supply);
        require!(current_supply >= amount_tokens_to_buy, BondingCurveError::InsufficientLiquidity);

        // x - количество уже проданных токенов (с децималами)
        let tokens_sold = curve.initial_bonding_supply
            .checked_sub(current_supply)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        // dx - количество покупаемых токенов (с децималами)
        let dx = amount_tokens_to_buy;

        // Рассчитываем стоимость в N-Dollar
        let cost_n_dollar = calculate_buy_cost(
            curve.slope_numerator,
            curve.slope_denominator,
            curve.intercept_scaled,
            tokens_sold,
            dx,
            curve.token_decimals,
            curve.n_dollar_decimals
        )?;
        msg!("Calculated cost (N-Dollar lamports): {}", cost_n_dollar);

        // Проверяем баланс пользователя N-Dollar
        require!(ctx.accounts.user_n_dollar_account.amount >= cost_n_dollar, BondingCurveError::InsufficientFunds);

        // 1. Перевод N-Dollar от пользователя в казну кривой
        let cpi_accounts_ndollar = Transfer {
            from: ctx.accounts.user_n_dollar_account.to_account_info(),
            to: ctx.accounts.n_dollar_treasury.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program_ndollar = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_ndollar = CpiContext::new(cpi_program_ndollar, cpi_accounts_ndollar);
        token::transfer(cpi_ctx_ndollar, cost_n_dollar)?;
        msg!("Transferred {} N-Dollar lamports from user to treasury", cost_n_dollar);

        // 2. Перевод мем-токенов от кривой пользователю
        // Нужны signer seeds для PDA кривой
        let mint_key = curve.mint.key();
        let curve_bump = curve.bump;
        let seeds = &[
            b"bonding_curve".as_ref(),
            mint_key.as_ref(),
            &[curve_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts_token = Transfer {
            from: ctx.accounts.bonding_curve_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.bonding_curve_authority.to_account_info(), // PDA кривой
        };
        let cpi_program_token = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_token = CpiContext::new_with_signer(cpi_program_token, cpi_accounts_token, signer_seeds);
        token::transfer(cpi_ctx_token, amount_tokens_to_buy)?;
        msg!("Transferred {} tokens from curve to user", amount_tokens_to_buy);

        msg!("Buy transaction successful");
        Ok(())
    }

     pub fn sell(ctx: Context<BuySell>, amount_tokens_to_sell: u64) -> Result<()> {
        msg!("Executing Sell...");
        let curve = &ctx.accounts.bonding_curve;
        require!(curve.is_initialized, BondingCurveError::NotInitialized);
        require!(amount_tokens_to_sell > 0, BondingCurveError::ZeroAmount);

        let current_supply = ctx.accounts.bonding_curve_token_account.amount;
        let n_dollar_in_treasury = ctx.accounts.n_dollar_treasury.amount;
        msg!("Current supply in curve: {}", current_supply);
        msg!("N-Dollar in treasury: {}", n_dollar_in_treasury);

        // Проверяем, что у пользователя достаточно токенов для продажи
        require!(ctx.accounts.user_token_account.amount >= amount_tokens_to_sell, BondingCurveError::InsufficientFunds);

        // x - количество уже проданных токенов (с децималами) до этой продажи
        let tokens_sold_before = curve.initial_bonding_supply
            .checked_sub(current_supply)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        // dx - количество продаваемых токенов (с децималами)
        let dx = amount_tokens_to_sell;

         // Убедимся, что не пытаемся продать больше, чем было продано
         // (т.е. x после продажи не может быть отрицательным)
        require!(tokens_sold_before >= dx, BondingCurveError::CannotSellMoreThanSold);

        // Рассчитываем выручку в N-Dollar
        let revenue_n_dollar = calculate_sell_revenue(
            curve.slope_numerator,
            curve.slope_denominator,
            curve.intercept_scaled,
            tokens_sold_before, // Используем x *до* продажи
            dx,
            curve.token_decimals,
            curve.n_dollar_decimals
        )?;
        msg!("Calculated revenue (N-Dollar lamports): {}", revenue_n_dollar);

        // Проверяем, достаточно ли N-Dollar в казне
        require!(n_dollar_in_treasury >= revenue_n_dollar, BondingCurveError::InsufficientTreasury);

        // 1. Перевод мем-токенов от пользователя кривой
        let cpi_accounts_token = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.bonding_curve_token_account.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(), // Пользователь подписывает
        };
        let cpi_program_token = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_token = CpiContext::new(cpi_program_token, cpi_accounts_token);
        token::transfer(cpi_ctx_token, amount_tokens_to_sell)?;
        msg!("Transferred {} tokens from user to curve", amount_tokens_to_sell);

        // 2. Перевод N-Dollar из казны пользователю
        // Нужны signer seeds для PDA кривой
        let mint_key = curve.mint.key();
        let curve_bump = curve.bump;
        let seeds = &[
            b"bonding_curve".as_ref(),
            mint_key.as_ref(),
            &[curve_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts_ndollar = Transfer {
            from: ctx.accounts.n_dollar_treasury.to_account_info(), // Из казны
            to: ctx.accounts.user_n_dollar_account.to_account_info(), // Пользователю
            authority: ctx.accounts.bonding_curve_authority.to_account_info(), // PDA кривой
        };
        let cpi_program_ndollar = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_ndollar = CpiContext::new_with_signer(cpi_program_ndollar, cpi_accounts_ndollar, signer_seeds);
        token::transfer(cpi_ctx_ndollar, revenue_n_dollar)?;
        msg!("Transferred {} N-Dollar lamports from treasury to user", revenue_n_dollar);

        msg!("Sell transaction successful");
        Ok(())
    }

}

// --- Вспомогательные функции для расчетов ---

// Расчет стоимости покупки dx токенов, когда x уже продано
// Cost = m/2 * ((x+dx)^2 - x^2) + c*dx (все величины с децималами, результат в N-Dollar lamports)
fn calculate_buy_cost(
    m_num: u128, // slope numerator (scaled by PRECISION_FACTOR)
    m_den: u128, // slope denominator (no scaling, it's TOTAL_BONDING_TOKENS)
    c_scaled: u128, // intercept scaled by PRECISION_FACTOR
    x: u64,      // tokens sold (with token decimals)
    dx: u64,     // tokens to buy (with token decimals)
    token_decimals: u8,
    n_dollar_decimals: u8,
) -> Result<u64> {
    msg!("Calculating buy cost: m_num={}, m_den={}, c_scaled={}, x={}, dx={}", m_num, m_den, c_scaled, x, dx);
    let x_u128 = x as u128;
    let dx_u128 = dx as u128;

    let token_decimal_factor = 10u128.pow(token_decimals as u32);
    let n_dollar_decimal_factor = 10u128.pow(n_dollar_decimals as u32);

    // --- Расчет компонента с m ---
    // m/2 * ((x+dx)^2 - x^2) = m/2 * (2*x*dx + dx^2) = m*x*dx + m/2 * dx^2
    // Все x и dx уже с децималами токена. m относится к токенам *без* децималов.

    // Переводим x и dx к масштабу без децималов для использования с m
    let x_base = x_u128.checked_div(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
    let _dx_base = dx_u128.checked_div(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
    // dx_base может быть 0 если dx < token_decimal_factor, обрабатываем это
    let _dx_remainder = dx_u128.checked_rem(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
    // Используем более точный dx_base_precise = dx / 10^dec
    let dx_base_precise_num = dx_u128;
    let dx_base_precise_den = token_decimal_factor;


    // m * x_base * dx_base_precise = (m_num / m_den) * x_base * (dx_num / dx_den)
    // = (m_num * x_base * dx_num) / (m_den * dx_den)
    let term1_num = m_num
        .checked_mul(x_base).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
    let term1_den = m_den
        .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
    let term1_scaled = term1_num // scaled by PRECISION
        .checked_div(term1_den).ok_or(BondingCurveError::CalculationOverflow)?;

    // m/2 * dx_base_precise^2 = (m_num / m_den / 2) * (dx_num / dx_den)^2
    // = (m_num * dx_num^2) / (m_den * 2 * dx_den^2)
    let term2_num = m_num
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
     let term2_den = m_den
        .checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
     let term2_scaled = term2_num // scaled by PRECISION
         .checked_div(term2_den).ok_or(BondingCurveError::CalculationOverflow)?;


    // --- Расчет компонента с c ---
    // c * dx_base_precise = c_scaled * (dx_num / dx_den) / PRECISION_FACTOR
    // = (c_scaled * dx_num) / (dx_den * PRECISION_FACTOR)
    let term3_num = c_scaled
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
    let term3_den = dx_base_precise_den
        .checked_mul(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;
    let term3_scaled = term3_num // scaled by PRECISION
         .checked_div(term3_den).ok_or(BondingCurveError::CalculationOverflow)?;

    // Суммируем компоненты (все scaled by PRECISION)
    let total_cost_scaled = term1_scaled
        .checked_add(term2_scaled).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_add(term3_scaled).ok_or(BondingCurveError::CalculationOverflow)?;
    msg!("Total cost scaled by PRECISION: {}", total_cost_scaled);


    // Переводим в N-Dollar lamports
    // cost_lamports = total_cost_scaled * n_dollar_decimal_factor / PRECISION_FACTOR
    let cost_lamports_u128 = total_cost_scaled
        .checked_mul(n_dollar_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_div(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;

     // Округляем вверх, чтобы пользователь заплатил чуть больше, если есть остаток
     let cost_lamports_remainder = total_cost_scaled
        .checked_mul(n_dollar_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_rem(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;

     let final_cost_u128 = if cost_lamports_remainder > 0 {
         cost_lamports_u128.checked_add(1).ok_or(BondingCurveError::CalculationOverflow)?
     } else {
         cost_lamports_u128
     };


    final_cost_u128.try_into().map_err(|_| BondingCurveError::CalculationOverflow.into())

}

// Расчет выручки от продажи dx токенов, когда x уже было продано
// Revenue = m/2 * (x^2 - (x-dx)^2) + c*dx (все величины с децималами, результат в N-Dollar lamports)
fn calculate_sell_revenue(
    m_num: u128,
    m_den: u128,
    c_scaled: u128,
    x: u64,      // tokens sold before this sell (with token decimals)
    dx: u64,     // tokens to sell (with token decimals)
    token_decimals: u8,
    n_dollar_decimals: u8,
) -> Result<u64> {
     msg!("Calculating sell revenue: m_num={}, m_den={}, c_scaled={}, x={}, dx={}", m_num, m_den, c_scaled, x, dx);
    // Логика расчета похожа на calculate_buy_cost, но интегрируем от x-dx до x
    // Revenue = m/2 * (x^2 - (x-dx)^2) + c*dx = m*x*dx - m/2*dx^2 + c*dx
    let x_u128 = x as u128;
    let dx_u128 = dx as u128;

    let token_decimal_factor = 10u128.pow(token_decimals as u32);
    let n_dollar_decimal_factor = 10u128.pow(n_dollar_decimals as u32);

    // Переводим x и dx к масштабу без децималов для использования с m
    let x_base = x_u128.checked_div(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
    let dx_base_precise_num = dx_u128;
    let dx_base_precise_den = token_decimal_factor;


    // m * x_base * dx_base_precise
    let term1_num = m_num
        .checked_mul(x_base).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
    let term1_den = m_den
        .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
    let term1_scaled = term1_num // scaled by PRECISION
        .checked_div(term1_den).ok_or(BondingCurveError::CalculationOverflow)?;

    // m/2 * dx_base_precise^2
    let term2_num = m_num
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
     let term2_den = m_den
        .checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
     let term2_scaled = term2_num // scaled by PRECISION
         .checked_div(term2_den).ok_or(BondingCurveError::CalculationOverflow)?;

    // c * dx_base_precise
    let term3_num = c_scaled
        .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
    let term3_den = dx_base_precise_den
        .checked_mul(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;
    let term3_scaled = term3_num // scaled by PRECISION
         .checked_div(term3_den).ok_or(BondingCurveError::CalculationOverflow)?;

    // Суммируем/вычитаем компоненты (все scaled by PRECISION)
    // Revenue = term1 - term2 + term3
    let total_revenue_scaled = term1_scaled
        .checked_sub(term2_scaled).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_add(term3_scaled).ok_or(BondingCurveError::CalculationOverflow)?;
    msg!("Total revenue scaled by PRECISION: {}", total_revenue_scaled);


    // Переводим в N-Dollar lamports
    // revenue_lamports = total_revenue_scaled * n_dollar_decimal_factor / PRECISION_FACTOR
     let revenue_lamports_u128 = total_revenue_scaled
        .checked_mul(n_dollar_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?
        .checked_div(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;

     // Округляем вниз при продаже (в пользу протокола)
     let final_revenue_u128 = revenue_lamports_u128;


    final_revenue_u128.try_into().map_err(|_| BondingCurveError::CalculationOverflow.into())
}


// --- Аккаунты ---
#[derive(Accounts)]
pub struct InitializeCurve<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + BondingCurve::INIT_SPACE,
        seeds = [b"bonding_curve", mint.key().as_ref()], // Сиды для PDA состояния
        bump // Сохраняем канонический бамп
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    pub mint: Account<'info, Mint>,
    pub n_dollar_mint: Account<'info, Mint>,

    #[account(
        // Проверяем, что владелец - это ПРАВИЛЬНЫЙ PDA authority.
        constraint = bonding_curve_token_account.owner == bonding_curve_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
        constraint = bonding_curve_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = n_dollar_mint,
        token::authority = bonding_curve_authority, // Anchor проверит, что переданный bonding_curve_authority может быть authority для этого ATA (т.е. он PDA)
        seeds = [b"n_dollar_treasury", mint.key().as_ref()],
        bump
    )]
    pub n_dollar_treasury: Account<'info, TokenAccount>,

    /// CHECK: PDA кривой. Его правильность неявно проверяется через token::authority для n_dollar_treasury
    /// и через constraint owner для bonding_curve_token_account. Нет необходимости в seeds/bump здесь.
    pub bonding_curve_authority: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuySell<'info> {
    #[account(
        mut, // Может изменяться при уточнении supply? Нет, только читаем
        seeds = [b"bonding_curve", bonding_curve.mint.as_ref()], // Используем mint из состояния
        bump = bonding_curve.bump, // Используем бамп из состояния
        has_one = mint @ BondingCurveError::InvalidMintAccount,
        has_one = n_dollar_mint @ BondingCurveError::InvalidMintAccount,
        has_one = bonding_curve_token_account @ BondingCurveError::InvalidTokenAccount,
        has_one = n_dollar_treasury @ BondingCurveError::InvalidTokenAccount,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(mut)] // Нужно для чтения децималов? Нет, можно не мут.
    pub mint: Account<'info, Mint>,
    #[account(mut)] // Нужно для чтения децималов? Нет, можно не мут.
    pub n_dollar_mint: Account<'info, Mint>,

    // ATA кривой для мем-токена (Откуда/куда идут токены)
    #[account(mut)]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,

    // ATA кривой для N-Dollar (Казна) (Куда/откуда идут N-Dollar)
    #[account(mut)]
    pub n_dollar_treasury: Account<'info, TokenAccount>,

    // ATA пользователя для мем-токена
    #[account(
        mut,
        constraint = user_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
        constraint = user_token_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // ATA пользователя для N-Dollar
    #[account(
        mut,
        constraint = user_n_dollar_account.mint == n_dollar_mint.key() @ BondingCurveError::InvalidTokenAccount,
        constraint = user_n_dollar_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
    )]
    pub user_n_dollar_account: Account<'info, TokenAccount>,

    /// CHECK: PDA кривой, используется как authority для ATA казны и токенов.
    #[account(
        seeds = [b"bonding_curve", bonding_curve.mint.as_ref()],
        bump = bonding_curve.bump
    )]
    pub bonding_curve_authority: AccountInfo<'info>,

    #[account(mut)]
    pub user_authority: Signer<'info>, // Пользователь, который покупает/продает

    pub token_program: Program<'info, Token>,
}


// --- Состояние ---

#[account]
#[derive(InitSpace)] // InitSpace сам посчитает размер
pub struct BondingCurve {
    pub mint: Pubkey,                        // 32
    pub n_dollar_mint: Pubkey,               // 32
    pub bonding_curve_token_account: Pubkey, // 32
    pub n_dollar_treasury: Pubkey,           // 32
    pub initial_bonding_supply: u64,         // 8
    // --- Атрибуты #[space] УБРАНЫ ---
    pub slope_numerator: u128,               // 16
    pub slope_denominator: u128,             // 16
    pub intercept_scaled: u128,              // 16
    // --- ------------------------ ---
    pub token_decimals: u8,                  // 1
    pub n_dollar_decimals: u8,               // 1
    pub authority: Pubkey,                   // 32
    pub is_initialized: bool,                // 1
    pub bump: u8,                            // 1
}


// --- Ошибки ---
#[error_code]
pub enum BondingCurveError {
    #[msg("Bonding curve account is not initialized.")]
    NotInitialized,
    #[msg("Input amount cannot be zero.")]
    ZeroAmount,
    #[msg("Calculation resulted in an overflow.")]
    CalculationOverflow,
    #[msg("Insufficient token liquidity on the bonding curve.")]
    InsufficientLiquidity,
    #[msg("Insufficient funds in user account.")]
    InsufficientFunds,
    #[msg("Insufficient N-Dollar in treasury to cover the sale.")]
    InsufficientTreasury,
    #[msg("Invalid mint account provided.")]
    InvalidMintAccount,
    #[msg("Invalid token account provided.")]
    InvalidTokenAccount,
    #[msg("Invalid token account owner.")]
    InvalidTokenAccountOwner,
     #[msg("Initial supply in bonding token account does not match expected amount.")]
    IncorrectInitialSupply,
    #[msg("Bonding curve token account is empty.")]
    BondingAccountEmpty,
    #[msg("Cannot sell more tokens than have been bought from the curve.")]
    CannotSellMoreThanSold,
}
