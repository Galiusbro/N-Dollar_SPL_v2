// use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
// use std::convert::TryInto;

// // ID программы (замени на свой реальный ID)
// declare_id!("GvFsepxBQ2q8xZ3PYYDooMdnMBzWQKkpKavzT7vM83rZ"); // Пример

// // Константа для масштабирования вычислений (для дробных частей)
// // 10^12 должно быть достаточно для цен и N-Dollar/Meme с 9 децималами
// const PRECISION_FACTOR: u128 = 1_000_000_000_000; // 10^12
// const _HALF_PRECISION_FACTOR: u128 = PRECISION_FACTOR / 2;

// // Константы из спецификации
// const START_PRICE_NUMERATOR: u128 = 5; // 0.00005 = 5 / 100000
// const START_PRICE_DENOMINATOR: u128 = 100_000;
// const TARGET_PRICE_NUMERATOR: u128 = 1; // 1 = 1 / 1
// const TARGET_PRICE_DENOMINATOR: u128 = 1;
// const TOTAL_BONDING_TOKENS: u64 = 30_000_000; // Без децималов

// #[program]
// pub mod bonding_curve {
//     use super::*;

//     pub fn initialize_curve(ctx: Context<InitializeCurve>) -> Result<()> {
//         msg!("Initializing Bonding Curve...");

//         let curve = &mut ctx.accounts.bonding_curve;
//         let bump = ctx.bumps.bonding_curve;

//         // Проверяем, что необходимые аккаунты переданы
//         require!(ctx.accounts.mint.key() != Pubkey::default(), BondingCurveError::InvalidMintAccount);
//         require!(ctx.accounts.n_dollar_mint.key() != Pubkey::default(), BondingCurveError::InvalidMintAccount);
//         require!(ctx.accounts.bonding_curve_token_account.amount > 0, BondingCurveError::BondingAccountEmpty);

//         // Получаем децималы
//         let token_decimals = ctx.accounts.mint.decimals;
//         let n_dollar_decimals = ctx.accounts.n_dollar_mint.decimals;
//         msg!("Token Decimals: {}, N-Dollar Decimals: {}", token_decimals, n_dollar_decimals);

//         // Рассчитываем общее предложение для кривой с учетом децималов
//         let total_bonding_supply_with_decimals = TOTAL_BONDING_TOKENS
//             .checked_mul(10u64.pow(token_decimals as u32))
//             .ok_or(BondingCurveError::CalculationOverflow)?;
//         msg!("Total Bonding Supply (with decimals): {}", total_bonding_supply_with_decimals);

//         // Проверяем, что на счету кривой ожидаемое количество токенов
//         require!(
//             ctx.accounts.bonding_curve_token_account.amount == total_bonding_supply_with_decimals,
//             BondingCurveError::IncorrectInitialSupply
//         );

//         // Рассчитываем параметры линейной кривой P(x) = m*x + c
//         // x - количество проданных токенов *без* децималов
//         // c = start_price = 0.00005
//         // m = (target_price - start_price) / total_bonding_tokens
//         //    = (1 - 0.00005) / 30,000,000 = 0.99995 / 30,000,000

//         // c (intercept) scaled
//         let intercept_scaled = START_PRICE_NUMERATOR
//             .checked_mul(PRECISION_FACTOR)
//             .ok_or(BondingCurveError::CalculationOverflow)?
//             .checked_div(START_PRICE_DENOMINATOR)
//             .ok_or(BondingCurveError::CalculationOverflow)?;
//         msg!("Intercept (c) scaled: {}", intercept_scaled);


//         // m (slope) scaled
//         // target_price_scaled = 1 * PRECISION_FACTOR
//         // start_price_scaled = intercept_scaled
//         let target_price_scaled = TARGET_PRICE_NUMERATOR
//             .checked_mul(PRECISION_FACTOR)
//             .ok_or(BondingCurveError::CalculationOverflow)?
//             .checked_div(TARGET_PRICE_DENOMINATOR)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         let price_diff_scaled = target_price_scaled
//             .checked_sub(intercept_scaled)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // Важно: TOTAL_BONDING_TOKENS не имеет децималов при расчете m
//         let slope_scaled_numerator = price_diff_scaled;
//         let slope_scaled_denominator = TOTAL_BONDING_TOKENS as u128; // m = numerator / denominator

//         msg!("Slope (m) scaled: {} / {}", slope_scaled_numerator, slope_scaled_denominator);


//         // Сохраняем состояние
//         curve.mint = ctx.accounts.mint.key();
//         curve.n_dollar_mint = ctx.accounts.n_dollar_mint.key();
//         curve.bonding_curve_token_account = ctx.accounts.bonding_curve_token_account.key();
//         curve.n_dollar_treasury = ctx.accounts.n_dollar_treasury.key();
//         curve.initial_bonding_supply = total_bonding_supply_with_decimals;
//         curve.slope_numerator = slope_scaled_numerator;
//         curve.slope_denominator = slope_scaled_denominator;
//         curve.intercept_scaled = intercept_scaled; // c * PRECISION
//         curve.token_decimals = token_decimals;
//         curve.n_dollar_decimals = n_dollar_decimals;
//         curve.authority = ctx.accounts.authority.key(); // Кто инициализировал
//         curve.is_initialized = true;
//         curve.bump = bump;
        
//         msg!("Bonding Curve Initialized for mint: {}", curve.mint);
//         Ok(())
//     }

//     pub fn buy(ctx: Context<BuySell>, amount_tokens_to_buy: u64) -> Result<()> {
//         msg!("Executing Buy...");
//         let curve = &ctx.accounts.bonding_curve;
//         require!(curve.is_initialized, BondingCurveError::NotInitialized);
//         require!(amount_tokens_to_buy > 0, BondingCurveError::ZeroAmount);

//         let current_supply = ctx.accounts.bonding_curve_token_account.amount;
//         msg!("Current supply in curve: {}", current_supply);
//         require!(current_supply >= amount_tokens_to_buy, BondingCurveError::InsufficientLiquidity);

//         // x - количество уже проданных токенов (с децималами)
//         let tokens_sold = curve.initial_bonding_supply
//             .checked_sub(current_supply)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // dx - количество покупаемых токенов (с децималами)
//         let dx = amount_tokens_to_buy;

//         // Рассчитываем стоимость в N-Dollar
//         let cost_n_dollar = calculate_buy_cost(
//             curve.slope_numerator,
//             curve.slope_denominator,
//             curve.intercept_scaled,
//             tokens_sold,
//             dx,
//             curve.token_decimals,
//             curve.n_dollar_decimals
//         )?;
//         msg!("Calculated cost (N-Dollar lamports): {}", cost_n_dollar);

//         // Проверяем баланс пользователя N-Dollar
//         require!(ctx.accounts.user_n_dollar_account.amount >= cost_n_dollar, BondingCurveError::InsufficientFunds);

//         // 1. Перевод N-Dollar от пользователя в казну кривой
//         let cpi_accounts_ndollar = Transfer {
//             from: ctx.accounts.user_n_dollar_account.to_account_info(),
//             to: ctx.accounts.n_dollar_treasury.to_account_info(),
//             authority: ctx.accounts.user_authority.to_account_info(),
//         };
//         let cpi_program_ndollar = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx_ndollar = CpiContext::new(cpi_program_ndollar, cpi_accounts_ndollar);
//         token::transfer(cpi_ctx_ndollar, cost_n_dollar)?;
//         msg!("Transferred {} N-Dollar lamports from user to treasury", cost_n_dollar);

//         // 2. Перевод мем-токенов от кривой пользователю
//         // Нужны signer seeds для PDA кривой
//         let mint_key = curve.mint.key();
//         let curve_bump = curve.bump;
//         let seeds = &[
//             b"bonding_curve".as_ref(),
//             mint_key.as_ref(),
//             &[curve_bump]
//         ];
//         let signer_seeds = &[&seeds[..]];

//         let cpi_accounts_token = Transfer {
//             from: ctx.accounts.bonding_curve_token_account.to_account_info(),
//             to: ctx.accounts.user_token_account.to_account_info(),
//             authority: ctx.accounts.bonding_curve_authority.to_account_info(), // PDA кривой
//         };
//         let cpi_program_token = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx_token = CpiContext::new_with_signer(cpi_program_token, cpi_accounts_token, signer_seeds);
//         token::transfer(cpi_ctx_token, amount_tokens_to_buy)?;
//         msg!("Transferred {} tokens from curve to user", amount_tokens_to_buy);

//         msg!("Buy transaction successful");
//         Ok(())
//     }

//      pub fn sell(ctx: Context<BuySell>, amount_tokens_to_sell: u64) -> Result<()> {
//         msg!("Executing Sell...");
//         let curve = &ctx.accounts.bonding_curve;
//         require!(curve.is_initialized, BondingCurveError::NotInitialized);
//         require!(amount_tokens_to_sell > 0, BondingCurveError::ZeroAmount);

//         let current_supply = ctx.accounts.bonding_curve_token_account.amount;
//         let n_dollar_in_treasury = ctx.accounts.n_dollar_treasury.amount;
//         msg!("Current supply in curve: {}", current_supply);
//         msg!("N-Dollar in treasury: {}", n_dollar_in_treasury);

//         // Проверяем, что у пользователя достаточно токенов для продажи
//         require!(ctx.accounts.user_token_account.amount >= amount_tokens_to_sell, BondingCurveError::InsufficientFunds);

//         // x - количество уже проданных токенов (с децималами) до этой продажи
//         let tokens_sold_before = curve.initial_bonding_supply
//             .checked_sub(current_supply)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // dx - количество продаваемых токенов (с децималами)
//         let dx = amount_tokens_to_sell;

//          // Убедимся, что не пытаемся продать больше, чем было продано
//          // (т.е. x после продажи не может быть отрицательным)
//         require!(tokens_sold_before >= dx, BondingCurveError::CannotSellMoreThanSold);

//         // Рассчитываем выручку в N-Dollar
//         let revenue_n_dollar = calculate_sell_revenue(
//             curve.slope_numerator,
//             curve.slope_denominator,
//             curve.intercept_scaled,
//             tokens_sold_before, // Используем x *до* продажи
//             dx,
//             curve.token_decimals,
//             curve.n_dollar_decimals
//         )?;
//         msg!("Calculated revenue (N-Dollar lamports): {}", revenue_n_dollar);

//         // Проверяем, достаточно ли N-Dollar в казне
//         require!(n_dollar_in_treasury >= revenue_n_dollar, BondingCurveError::InsufficientTreasury);

//         // 1. Перевод мем-токенов от пользователя кривой
//         let cpi_accounts_token = Transfer {
//             from: ctx.accounts.user_token_account.to_account_info(),
//             to: ctx.accounts.bonding_curve_token_account.to_account_info(),
//             authority: ctx.accounts.user_authority.to_account_info(), // Пользователь подписывает
//         };
//         let cpi_program_token = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx_token = CpiContext::new(cpi_program_token, cpi_accounts_token);
//         token::transfer(cpi_ctx_token, amount_tokens_to_sell)?;
//         msg!("Transferred {} tokens from user to curve", amount_tokens_to_sell);

//         // 2. Перевод N-Dollar из казны пользователю
//         // Нужны signer seeds для PDA кривой
//         let mint_key = curve.mint.key();
//         let curve_bump = curve.bump;
//         let seeds = &[
//             b"bonding_curve".as_ref(),
//             mint_key.as_ref(),
//             &[curve_bump]
//         ];
//         let signer_seeds = &[&seeds[..]];

//         let cpi_accounts_ndollar = Transfer {
//             from: ctx.accounts.n_dollar_treasury.to_account_info(), // Из казны
//             to: ctx.accounts.user_n_dollar_account.to_account_info(), // Пользователю
//             authority: ctx.accounts.bonding_curve_authority.to_account_info(), // PDA кривой
//         };
//         let cpi_program_ndollar = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx_ndollar = CpiContext::new_with_signer(cpi_program_ndollar, cpi_accounts_ndollar, signer_seeds);
//         token::transfer(cpi_ctx_ndollar, revenue_n_dollar)?;
//         msg!("Transferred {} N-Dollar lamports from treasury to user", revenue_n_dollar);

//         msg!("Sell transaction successful");
//         Ok(())
//     }

// }

// // --- Вспомогательные функции для расчетов ---

// // Расчет стоимости покупки dx токенов, когда x уже продано
// // Cost = m/2 * ((x+dx)^2 - x^2) + c*dx (все величины с децималами, результат в N-Dollar lamports)
// fn calculate_buy_cost(
//     m_num: u128, // slope numerator (scaled by PRECISION_FACTOR)
//     m_den: u128, // slope denominator (no scaling, it's TOTAL_BONDING_TOKENS)
//     c_scaled: u128, // intercept scaled by PRECISION_FACTOR
//     x: u64,      // tokens sold (with token decimals)
//     dx: u64,     // tokens to buy (with token decimals)
//     token_decimals: u8,
//     n_dollar_decimals: u8,
// ) -> Result<u64> {
//     msg!("Calculating buy cost: m_num={}, m_den={}, c_scaled={}, x={}, dx={}", m_num, m_den, c_scaled, x, dx);
//     let x_u128 = x as u128;
//     let dx_u128 = dx as u128;

//     let token_decimal_factor = 10u128.pow(token_decimals as u32);
//     let n_dollar_decimal_factor = 10u128.pow(n_dollar_decimals as u32);

//     // --- Расчет компонента с m ---
//     // m/2 * ((x+dx)^2 - x^2) = m/2 * (2*x*dx + dx^2) = m*x*dx + m/2 * dx^2
//     // Все x и dx уже с децималами токена. m относится к токенам *без* децималов.

//     // Переводим x и dx к масштабу без децималов для использования с m
//     let x_base = x_u128.checked_div(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
//     let _dx_base = dx_u128.checked_div(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
//     // dx_base может быть 0 если dx < token_decimal_factor, обрабатываем это
//     let _dx_remainder = dx_u128.checked_rem(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
//     // Используем более точный dx_base_precise = dx / 10^dec
//     let dx_base_precise_num = dx_u128;
//     let dx_base_precise_den = token_decimal_factor;


//     // m * x_base * dx_base_precise = (m_num / m_den) * x_base * (dx_num / dx_den)
//     // = (m_num * x_base * dx_num) / (m_den * dx_den)
//     let term1_num = m_num
//         .checked_mul(x_base).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term1_den = m_den
//         .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term1_scaled = term1_num // scaled by PRECISION
//         .checked_div(term1_den).ok_or(BondingCurveError::CalculationOverflow)?;

//     // m/2 * dx_base_precise^2 = (m_num / m_den / 2) * (dx_num / dx_den)^2
//     // = (m_num * dx_num^2) / (m_den * 2 * dx_den^2)
//     let term2_num = m_num
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
//      let term2_den = m_den
//         .checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
//      let term2_scaled = term2_num // scaled by PRECISION
//          .checked_div(term2_den).ok_or(BondingCurveError::CalculationOverflow)?;


//     // --- Расчет компонента с c ---
//     // c * dx_base_precise = c_scaled * (dx_num / dx_den) / PRECISION_FACTOR
//     // = (c_scaled * dx_num) / (dx_den * PRECISION_FACTOR)
//     let term3_num = c_scaled
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term3_den = dx_base_precise_den
//         .checked_mul(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term3_scaled = term3_num // scaled by PRECISION
//          .checked_div(term3_den).ok_or(BondingCurveError::CalculationOverflow)?;

//     // Суммируем компоненты (все scaled by PRECISION)
//     let total_cost_scaled = term1_scaled
//         .checked_add(term2_scaled).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_add(term3_scaled).ok_or(BondingCurveError::CalculationOverflow)?;
//     msg!("Total cost scaled by PRECISION: {}", total_cost_scaled);


//     // Переводим в N-Dollar lamports
//     // cost_lamports = total_cost_scaled * n_dollar_decimal_factor / PRECISION_FACTOR
//     let cost_lamports_u128 = total_cost_scaled
//         .checked_mul(n_dollar_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_div(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;

//      // Округляем вверх, чтобы пользователь заплатил чуть больше, если есть остаток
//      let cost_lamports_remainder = total_cost_scaled
//         .checked_mul(n_dollar_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_rem(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;

//      let final_cost_u128 = if cost_lamports_remainder > 0 {
//          cost_lamports_u128.checked_add(1).ok_or(BondingCurveError::CalculationOverflow)?
//      } else {
//          cost_lamports_u128
//      };


//     final_cost_u128.try_into().map_err(|_| BondingCurveError::CalculationOverflow.into())

// }

// // Расчет выручки от продажи dx токенов, когда x уже было продано
// // Revenue = m/2 * (x^2 - (x-dx)^2) + c*dx (все величины с децималами, результат в N-Dollar lamports)
// fn calculate_sell_revenue(
//     m_num: u128,
//     m_den: u128,
//     c_scaled: u128,
//     x: u64,      // tokens sold before this sell (with token decimals)
//     dx: u64,     // tokens to sell (with token decimals)
//     token_decimals: u8,
//     n_dollar_decimals: u8,
// ) -> Result<u64> {
//      msg!("Calculating sell revenue: m_num={}, m_den={}, c_scaled={}, x={}, dx={}", m_num, m_den, c_scaled, x, dx);
//     // Логика расчета похожа на calculate_buy_cost, но интегрируем от x-dx до x
//     // Revenue = m/2 * (x^2 - (x-dx)^2) + c*dx = m*x*dx - m/2*dx^2 + c*dx
//     let x_u128 = x as u128;
//     let dx_u128 = dx as u128;

//     let token_decimal_factor = 10u128.pow(token_decimals as u32);
//     let n_dollar_decimal_factor = 10u128.pow(n_dollar_decimals as u32);

//     // Переводим x и dx к масштабу без децималов для использования с m
//     let x_base = x_u128.checked_div(token_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?;
//     let dx_base_precise_num = dx_u128;
//     let dx_base_precise_den = token_decimal_factor;


//     // m * x_base * dx_base_precise
//     let term1_num = m_num
//         .checked_mul(x_base).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term1_den = m_den
//         .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term1_scaled = term1_num // scaled by PRECISION
//         .checked_div(term1_den).ok_or(BondingCurveError::CalculationOverflow)?;

//     // m/2 * dx_base_precise^2
//     let term2_num = m_num
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
//      let term2_den = m_den
//         .checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_mul(dx_base_precise_den).ok_or(BondingCurveError::CalculationOverflow)?;
//      let term2_scaled = term2_num // scaled by PRECISION
//          .checked_div(term2_den).ok_or(BondingCurveError::CalculationOverflow)?;

//     // c * dx_base_precise
//     let term3_num = c_scaled
//         .checked_mul(dx_base_precise_num).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term3_den = dx_base_precise_den
//         .checked_mul(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;
//     let term3_scaled = term3_num // scaled by PRECISION
//          .checked_div(term3_den).ok_or(BondingCurveError::CalculationOverflow)?;

//     // Суммируем/вычитаем компоненты (все scaled by PRECISION)
//     // Revenue = term1 - term2 + term3
//     let total_revenue_scaled = term1_scaled
//         .checked_sub(term2_scaled).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_add(term3_scaled).ok_or(BondingCurveError::CalculationOverflow)?;
//     msg!("Total revenue scaled by PRECISION: {}", total_revenue_scaled);


//     // Переводим в N-Dollar lamports
//     // revenue_lamports = total_revenue_scaled * n_dollar_decimal_factor / PRECISION_FACTOR
//      let revenue_lamports_u128 = total_revenue_scaled
//         .checked_mul(n_dollar_decimal_factor).ok_or(BondingCurveError::CalculationOverflow)?
//         .checked_div(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;

//      // Округляем вниз при продаже (в пользу протокола)
//      let final_revenue_u128 = revenue_lamports_u128;


//     final_revenue_u128.try_into().map_err(|_| BondingCurveError::CalculationOverflow.into())
// }


// // --- Аккаунты ---
// #[derive(Accounts)]
// pub struct InitializeCurve<'info> {
//     #[account(
//         init,
//         payer = authority,
//         space = 8 + BondingCurve::INIT_SPACE,
//         seeds = [b"bonding_curve", mint.key().as_ref()], // Сиды для PDA состояния
//         bump // Сохраняем канонический бамп
//     )]
//     pub bonding_curve: Account<'info, BondingCurve>,

//     pub mint: Account<'info, Mint>,
//     pub n_dollar_mint: Account<'info, Mint>,

//     #[account(
//         // Проверяем, что владелец - это ПРАВИЛЬНЫЙ PDA authority.
//         constraint = bonding_curve_token_account.owner == bonding_curve_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
//         constraint = bonding_curve_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
//     )]
//     pub bonding_curve_token_account: Account<'info, TokenAccount>,

//     #[account(
//         init,
//         payer = authority,
//         token::mint = n_dollar_mint,
//         token::authority = bonding_curve_authority, // Anchor проверит, что переданный bonding_curve_authority может быть authority для этого ATA (т.е. он PDA)
//         seeds = [b"n_dollar_treasury", mint.key().as_ref()],
//         bump
//     )]
//     pub n_dollar_treasury: Account<'info, TokenAccount>,

//     /// CHECK: PDA кривой. Его правильность неявно проверяется через token::authority для n_dollar_treasury
//     /// и через constraint owner для bonding_curve_token_account. Нет необходимости в seeds/bump здесь.
//     pub bonding_curve_authority: AccountInfo<'info>,

//     #[account(mut)]
//     pub authority: Signer<'info>,

//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct BuySell<'info> {
//     #[account(
//         mut, // Может изменяться при уточнении supply? Нет, только читаем
//         seeds = [b"bonding_curve", bonding_curve.mint.as_ref()], // Используем mint из состояния
//         bump = bonding_curve.bump, // Используем бамп из состояния
//         has_one = mint @ BondingCurveError::InvalidMintAccount,
//         has_one = n_dollar_mint @ BondingCurveError::InvalidMintAccount,
//         has_one = bonding_curve_token_account @ BondingCurveError::InvalidTokenAccount,
//         has_one = n_dollar_treasury @ BondingCurveError::InvalidTokenAccount,
//     )]
//     pub bonding_curve: Account<'info, BondingCurve>,

//     #[account(mut)] // Нужно для чтения децималов? Нет, можно не мут.
//     pub mint: Account<'info, Mint>,
//     #[account(mut)] // Нужно для чтения децималов? Нет, можно не мут.
//     pub n_dollar_mint: Account<'info, Mint>,

//     // ATA кривой для мем-токена (Откуда/куда идут токены)
//     #[account(mut)]
//     pub bonding_curve_token_account: Account<'info, TokenAccount>,

//     // ATA кривой для N-Dollar (Казна) (Куда/откуда идут N-Dollar)
//     #[account(mut)]
//     pub n_dollar_treasury: Account<'info, TokenAccount>,

//     // ATA пользователя для мем-токена
//     #[account(
//         mut,
//         constraint = user_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
//         constraint = user_token_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
//     )]
//     pub user_token_account: Account<'info, TokenAccount>,

//     // ATA пользователя для N-Dollar
//     #[account(
//         mut,
//         constraint = user_n_dollar_account.mint == n_dollar_mint.key() @ BondingCurveError::InvalidTokenAccount,
//         constraint = user_n_dollar_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
//     )]
//     pub user_n_dollar_account: Account<'info, TokenAccount>,

//     /// CHECK: PDA кривой, используется как authority для ATA казны и токенов.
//     #[account(
//         seeds = [b"bonding_curve", bonding_curve.mint.as_ref()],
//         bump = bonding_curve.bump
//     )]
//     pub bonding_curve_authority: AccountInfo<'info>,

//     #[account(mut)]
//     pub user_authority: Signer<'info>, // Пользователь, который покупает/продает

//     pub token_program: Program<'info, Token>,
// }


// // --- Состояние ---

// #[account]
// #[derive(InitSpace)] // InitSpace сам посчитает размер
// pub struct BondingCurve {
//     pub mint: Pubkey,                        // 32
//     pub n_dollar_mint: Pubkey,               // 32
//     pub bonding_curve_token_account: Pubkey, // 32
//     pub n_dollar_treasury: Pubkey,           // 32
//     pub initial_bonding_supply: u64,         // 8
//     // --- Атрибуты #[space] УБРАНЫ ---
//     pub slope_numerator: u128,               // 16
//     pub slope_denominator: u128,             // 16
//     pub intercept_scaled: u128,              // 16
//     // --- ------------------------ ---
//     pub token_decimals: u8,                  // 1
//     pub n_dollar_decimals: u8,               // 1
//     pub authority: Pubkey,                   // 32
//     pub is_initialized: bool,                // 1
//     pub bump: u8,                            // 1
// }


// // --- Ошибки ---
// #[error_code]
// pub enum BondingCurveError {
//     #[msg("Bonding curve account is not initialized.")]
//     NotInitialized,
//     #[msg("Input amount cannot be zero.")]
//     ZeroAmount,
//     #[msg("Calculation resulted in an overflow.")]
//     CalculationOverflow,
//     #[msg("Insufficient token liquidity on the bonding curve.")]
//     InsufficientLiquidity,
//     #[msg("Insufficient funds in user account.")]
//     InsufficientFunds,
//     #[msg("Insufficient N-Dollar in treasury to cover the sale.")]
//     InsufficientTreasury,
//     #[msg("Invalid mint account provided.")]
//     InvalidMintAccount,
//     #[msg("Invalid token account provided.")]
//     InvalidTokenAccount,
//     #[msg("Invalid token account owner.")]
//     InvalidTokenAccountOwner,
//      #[msg("Initial supply in bonding token account does not match expected amount.")]
//     IncorrectInitialSupply,
//     #[msg("Bonding curve token account is empty.")]
//     BondingAccountEmpty,
//     #[msg("Cannot sell more tokens than have been bought from the curve.")]
//     CannotSellMoreThanSold,
// }


// use anchor_lang::prelude::*;
// use anchor_spl::{
//     associated_token::AssociatedToken,
//     token::{self, Mint, Token, TokenAccount, Transfer},
// };
// use std::convert::TryInto;

// declare_id!("GvFsepxBQ2q8xZ3PYYDooMdnMBzWQKkpKavzT7vM83rZ"); // Replace with your actual program ID

// const PRECISION_FACTOR: u128 = 1_000_000_000_000; // 10^12 for scaling fixed-point math

// #[program]
// pub mod bonding_curve {
//     use super::*;

//     pub fn initialize_curve(ctx: Context<InitializeCurve>) -> Result<()> {
//         let curve = &mut ctx.accounts.bonding_curve;
//         let bonding_token_account = &ctx.accounts.bonding_curve_token_account;
//         let mint = &ctx.accounts.mint;
//         let n_dollar_mint = &ctx.accounts.n_dollar_mint;

//         // --- Basic Initialization ---
//         curve.is_initialized = true;
//         curve.authority = ctx.accounts.authority.key();
//         curve.mint = mint.key();
//         curve.n_dollar_mint = n_dollar_mint.key();
//         curve.bonding_curve_token_account = bonding_token_account.key();
//         curve.n_dollar_treasury = ctx.accounts.n_dollar_treasury.key(); // Storing the key of the newly created ATA
//         curve.bump = ctx.bumps.bonding_curve; // Store the canonical bump

//         // --- Read Decimals ---
//         curve.token_decimals = mint.decimals;
//         curve.n_dollar_decimals = n_dollar_mint.decimals;

//         // --- Read Initial Supply ---
//         // Reload account data if needed, though it should be current after distributeTokens CPI
//         // bonding_token_account.reload()?; // Uncomment if facing issues with stale data
//         curve.initial_bonding_supply = bonding_token_account.amount;
//         require!(
//             curve.initial_bonding_supply > 0,
//             BondingCurveError::BondingAccountEmpty
//         );

//         // --- Calculate Curve Parameters (Example: Linear Curve y = mx + c) ---
//         // y = price in N-Dollar per base token unit (scaled by PRECISION_FACTOR)
//         // x = supply of the token on the bonding curve

//         // Let's assume a simple model where the price goes from near 0 up to a target price
//         // when the initial supply is fully sold (or bought back).
//         // Target price example: 0.1 N-Dollar per token (at x=0, after full buyback)
//         // Max price example: 1 N-Dollar per token (at x = initial_bonding_supply)

//         let target_price_scaled: u128 = 100_000_000_000; // 0.1 * 10^12 (scaled intercept)
//         let max_price_scaled: u128 = 1_000_000_000_000; // 1.0 * 10^12 (scaled price at max supply)

//         let initial_supply_u128: u128 = curve.initial_bonding_supply.into();

//         // Calculate slope (m_scaled)
//         // m_scaled = (max_price_scaled - target_price_scaled) / initial_supply_u128
//         let price_diff_scaled = max_price_scaled
//             .checked_sub(target_price_scaled)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // To maintain precision, store slope as a fraction (numerator/denominator)
//         curve.slope_numerator = price_diff_scaled;
//         curve.slope_denominator = initial_supply_u128; // Denominator is the initial supply itself

//         // Intercept (c_scaled)
//         curve.intercept_scaled = target_price_scaled; // Price when supply is 0

//         msg!("Bonding curve initialized:");
//         msg!("  Initial Supply: {}", curve.initial_bonding_supply);
//         msg!("  Slope Numerator: {}", curve.slope_numerator);
//         msg!("  Slope Denominator: {}", curve.slope_denominator);
//         msg!("  Intercept Scaled: {}", curve.intercept_scaled);
//         msg!("  Token Decimals: {}", curve.token_decimals);
//         msg!("  N-Dollar Decimals: {}", curve.n_dollar_decimals);

//         Ok(())
//     }

//     pub fn buy(ctx: Context<BuySell>, amount_to_buy: u64) -> Result<()> {
//         let curve = &ctx.accounts.bonding_curve;
//         require!(curve.is_initialized, BondingCurveError::NotInitialized);
//         require!(amount_to_buy > 0, BondingCurveError::ZeroAmount);

//         // Reload data to get current balances
//         ctx.accounts.bonding_curve_token_account.reload()?;
//         ctx.accounts.user_n_dollar_account.reload()?;

//         let current_supply = ctx.accounts.bonding_curve_token_account.amount;
//         require!(
//             current_supply >= amount_to_buy,
//             BondingCurveError::InsufficientLiquidity
//         );

//         let final_supply = current_supply
//             .checked_sub(amount_to_buy)
//             .ok_or(BondingCurveError::CalculationOverflow)?; // Supply decreases when buying

//         // Calculate cost using integral of price function P(x) = m*x + c
//         // Cost = Integral[P(x) dx] from final_supply to current_supply
//         // Cost = [m/2 * x^2 + c*x] from final_supply to current_supply
//         // Cost = (m/2 * current_supply^2 + c*current_supply) - (m/2 * final_supply^2 + c*final_supply)
//         // Cost = m/2 * (current_supply^2 - final_supply^2) + c * (current_supply - final_supply)
//         // Cost = m/2 * (current_supply - final_supply)*(current_supply + final_supply) + c * amount_to_buy
//         // Cost = m/2 * amount_to_buy * (current_supply + final_supply) + c * amount_to_buy

//         let m_num = curve.slope_numerator;
//         let m_den = curve.slope_denominator;
//         let c_scaled = curve.intercept_scaled;
//         let amount_u128: u128 = amount_to_buy.into();
//         let current_supply_u128: u128 = current_supply.into();
//         let final_supply_u128: u128 = final_supply.into();

//         let term1_intermediate = amount_u128
//             .checked_mul(
//                 current_supply_u128
//                     .checked_add(final_supply_u128)
//                     .ok_or(BondingCurveError::CalculationOverflow)?,
//             )
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // term1 = (m_num / m_den / 2) * term1_intermediate
//         let term1 = m_num
//             .checked_mul(term1_intermediate)
//             .ok_or(BondingCurveError::CalculationOverflow)?
//             .checked_div(
//                 m_den
//                     .checked_mul(2)
//                     .ok_or(BondingCurveError::CalculationOverflow)?,
//             )
//             .ok_or(BondingCurveError::CalculationOverflow)?; // Potential loss of precision here if not perfectly divisible

//         let term2 = c_scaled
//             .checked_mul(amount_u128)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // cost_scaled is the cost in N-Dollar * 10^(N-Dollar Decimals) * 10^12 / 10^(Token Decimals)
//         // We need to divide by PRECISION_FACTOR (10^12) at the end
//         let cost_scaled = term1
//             .checked_add(term2)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // Convert scaled cost to N-Dollar lamports
//         // cost_in_n_dollar_lamports = cost_scaled / PRECISION_FACTOR
//         let cost_in_n_dollar_lamports_u128 = cost_scaled
//             .checked_div(PRECISION_FACTOR)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         // Convert u128 to u64 for transfer, check for overflow
//         let cost_in_n_dollar_lamports: u64 = cost_in_n_dollar_lamports_u128
//             .try_into()
//             .map_err(|_| BondingCurveError::CalculationOverflow)?;

//         msg!(
//             "Buying {} tokens. Calculated cost: {} N-Dollar lamports",
//             amount_to_buy,
//             cost_in_n_dollar_lamports
//         );

//         // Check user N-Dollar balance
//         require!(
//             ctx.accounts.user_n_dollar_account.amount >= cost_in_n_dollar_lamports,
//             BondingCurveError::InsufficientFunds
//         );

//         // --- Perform Transfers ---

//         // 1. Transfer N-Dollars from User to Treasury
//         let cpi_accounts_ndollar = Transfer {
//             from: ctx.accounts.user_n_dollar_account.to_account_info(),
//             to: ctx.accounts.n_dollar_treasury.to_account_info(),
//             authority: ctx.accounts.user_authority.to_account_info(), // User signs
//         };
//         let cpi_ctx_ndollar = CpiContext::new(
//             ctx.accounts.token_program.to_account_info(),
//             cpi_accounts_ndollar,
//         );
//         token::transfer(cpi_ctx_ndollar, cost_in_n_dollar_lamports)?;

//         // 2. Transfer Tokens from Curve to User
//         let bump = curve.bump; // Получаем бамп из состояния
//         let mint_key_bytes = curve.mint.to_bytes(); // Получаем байты ключа минта из состояния

//         // Создаем binding для bump, чтобы он жил дольше
//         let bump_slice = &[bump];

//         // Формируем сиды
//         let seeds = &[
//             b"bonding_curve".as_ref(),
//             &mint_key_bytes,
//             bump_slice,
//         ][..];
//         let signer_seeds = &[seeds];

//         let cpi_accounts_token = Transfer {
//             from: ctx.accounts.bonding_curve_token_account.to_account_info(),
//             to: ctx.accounts.user_token_account.to_account_info(),
//             authority: ctx.accounts.bonding_curve.to_account_info(), // PDA signs
//         };
//         let cpi_ctx_token = CpiContext::new_with_signer(
//             ctx.accounts.token_program.to_account_info(),
//             cpi_accounts_token,
//             signer_seeds, // Pass PDA seeds
//         );
//         token::transfer(cpi_ctx_token, amount_to_buy)?;

//         Ok(())
//     }

//     pub fn sell(ctx: Context<BuySell>, amount_to_sell: u64) -> Result<()> {
//         let curve = &ctx.accounts.bonding_curve;
//         require!(curve.is_initialized, BondingCurveError::NotInitialized);
//         require!(amount_to_sell > 0, BondingCurveError::ZeroAmount);

//         // Reload data to get current balances
//         ctx.accounts.bonding_curve_token_account.reload()?;
//         ctx.accounts.user_token_account.reload()?;
//         ctx.accounts.n_dollar_treasury.reload()?;

//         // Check user token balance
//         require!(
//             ctx.accounts.user_token_account.amount >= amount_to_sell,
//             BondingCurveError::InsufficientFunds
//         );

//         let current_supply = ctx.accounts.bonding_curve_token_account.amount;
//         let final_supply = current_supply
//             .checked_add(amount_to_sell)
//             .ok_or(BondingCurveError::CalculationOverflow)?; // Supply increases when selling

//         // Calculate proceeds using integral of price function P(x) = m*x + c
//         // Proceeds = Integral[P(x) dx] from current_supply to final_supply
//         // Proceeds = [m/2 * x^2 + c*x] from current_supply to final_supply
//         // Proceeds = (m/2 * final_supply^2 + c*final_supply) - (m/2 * current_supply^2 + c*current_supply)
//         // Proceeds = m/2 * (final_supply^2 - current_supply^2) + c * (final_supply - current_supply)
//         // Proceeds = m/2 * (final_supply - current_supply)*(final_supply + current_supply) + c * amount_to_sell
//         // Proceeds = m/2 * amount_to_sell * (final_supply + current_supply) + c * amount_to_sell

//         let m_num = curve.slope_numerator;
//         let m_den = curve.slope_denominator;
//         let c_scaled = curve.intercept_scaled;
//         let amount_u128: u128 = amount_to_sell.into();
//         let current_supply_u128: u128 = current_supply.into();
//         let final_supply_u128: u128 = final_supply.into();

//         let term1_intermediate = amount_u128
//             .checked_mul(
//                 final_supply_u128
//                     .checked_add(current_supply_u128)
//                     .ok_or(BondingCurveError::CalculationOverflow)?,
//             )
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         let term1 = m_num
//             .checked_mul(term1_intermediate)
//             .ok_or(BondingCurveError::CalculationOverflow)?
//             .checked_div(
//                 m_den
//                     .checked_mul(2)
//                     .ok_or(BondingCurveError::CalculationOverflow)?,
//             )
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         let term2 = c_scaled
//             .checked_mul(amount_u128)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         let proceeds_scaled = term1
//             .checked_add(term2)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         let proceeds_in_n_dollar_lamports_u128 = proceeds_scaled
//             .checked_div(PRECISION_FACTOR)
//             .ok_or(BondingCurveError::CalculationOverflow)?;

//         let proceeds_in_n_dollar_lamports: u64 = proceeds_in_n_dollar_lamports_u128
//             .try_into()
//             .map_err(|_| BondingCurveError::CalculationOverflow)?;

//         msg!(
//             "Selling {} tokens. Calculated proceeds: {} N-Dollar lamports",
//             amount_to_sell,
//             proceeds_in_n_dollar_lamports
//         );

//         // Check treasury N-Dollar balance
//         require!(
//             ctx.accounts.n_dollar_treasury.amount >= proceeds_in_n_dollar_lamports,
//             BondingCurveError::InsufficientTreasury
//         );

//         // --- Perform Transfers ---

//         // 1. Transfer Tokens from User to Curve
//         let cpi_accounts_token = Transfer {
//             from: ctx.accounts.user_token_account.to_account_info(),
//             to: ctx.accounts.bonding_curve_token_account.to_account_info(),
//             authority: ctx.accounts.user_authority.to_account_info(), // User signs
//         };
//         let cpi_ctx_token = CpiContext::new(
//             ctx.accounts.token_program.to_account_info(),
//             cpi_accounts_token,
//         );
//         token::transfer(cpi_ctx_token, amount_to_sell)?;

//         // 2. Transfer N-Dollars from Treasury to User
//         let bump = curve.bump; // Получаем бамп из состояния
//         let mint_key_bytes = curve.mint.to_bytes(); // Получаем байты ключа минта из состояния

//         // Создаем binding для bump, чтобы он жил дольше
//         let bump_slice = &[bump];

//         // Формируем сиды
//         let seeds = &[
//             b"bonding_curve".as_ref(),
//             &mint_key_bytes, // Используем байты ключа
//             bump_slice,      // Используем binding
//         ][..]; // Создаем срез из ссылок
//         let signer_seeds = &[seeds];

//         let cpi_accounts_ndollar = Transfer {
//             from: ctx.accounts.n_dollar_treasury.to_account_info(),
//             to: ctx.accounts.user_n_dollar_account.to_account_info(),
//             authority: ctx.accounts.bonding_curve.to_account_info(), // PDA signs
//         };
//         let cpi_ctx_ndollar = CpiContext::new_with_signer(
//             ctx.accounts.token_program.to_account_info(),
//             cpi_accounts_ndollar,
//             signer_seeds, // Pass PDA seeds
//         );
//         token::transfer(cpi_ctx_ndollar, proceeds_in_n_dollar_lamports)?;

//         Ok(())
//     }
// }

// // --- Аккаунты ---
// #[derive(Accounts)]
// pub struct InitializeCurve<'info> {
//     #[account(
//         init,
//         payer = authority,
//         space = 8 + BondingCurve::INIT_SPACE, // 8 для дискриминатора + размер структуры
//         seeds = [b"bonding_curve", mint.key().as_ref()],
//         bump
//     )]
//     pub bonding_curve: Account<'info, BondingCurve>,

//     pub mint: Account<'info, Mint>, // Пользовательский токен
//     pub n_dollar_mint: Account<'info, Mint>, // N-Dollar

//     #[account(
//         // Проверяем, что минт этого аккаунта совпадает с пользовательским токеном
//         constraint = bonding_curve_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
//         // Важно: Мы ожидаем, что distributeTokens уже перевел сюда токены.
//         // Проверка владельца здесь может быть избыточна, если мы доверяем процессу создания.
//         // Можно добавить проверку: constraint = bonding_curve_token_account.amount > 0,
//     )]
//     pub bonding_curve_token_account: Account<'info, TokenAccount>, // Сюда пришли 30%

//     #[account(
//         init, // Создаем ATA для казны N-Dollar
//         payer = authority,
//         associated_token::mint = n_dollar_mint,
//         // Владельцем (authority) этого ATA будет PDA bonding_curve
//         associated_token::authority = bonding_curve,
//     )]
//     pub n_dollar_treasury: Account<'info, TokenAccount>, // Казна N-Dollar (НЕТ seeds/bump!)

//     #[account(mut)]
//     pub authority: Signer<'info>, // Payer (пользователь)

//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub associated_token_program: Program<'info, AssociatedToken>, // Нужна для ATA
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct BuySell<'info> {
//     #[account(
//         mut, // Состояние может меняться (если отслеживаем объем торгов и т.д.)
//         seeds = [b"bonding_curve", mint.key().as_ref()], // Используем mint из переданного аккаунта
//         bump = bonding_curve.bump, // Используем сохраненный bump
//         // Проверяем, что ключи в состоянии соответствуют переданным аккаунтам
//         has_one = mint @ BondingCurveError::InvalidMintAccount,
//         has_one = n_dollar_mint @ BondingCurveError::InvalidMintAccount,
//         constraint = bonding_curve.bonding_curve_token_account == bonding_curve_token_account.key() @ BondingCurveError::InvalidTokenAccount,
//         constraint = bonding_curve.n_dollar_treasury == n_dollar_treasury.key() @ BondingCurveError::InvalidTokenAccount,
//     )]
//     pub bonding_curve: Account<'info, BondingCurve>,

//     // Можно не мутабельные, если читаем только decimals
//     pub mint: Account<'info, Mint>,
//     pub n_dollar_mint: Account<'info, Mint>,

//     #[account(
//         mut, // Баланс меняется
//         // Проверяем, что владелец - это PDA кривой (ключ которого == адрес bonding_curve)
//         constraint = bonding_curve_token_account.owner == bonding_curve.key() @ BondingCurveError::InvalidTokenAccountOwner,
//     )]
//     pub bonding_curve_token_account: Account<'info, TokenAccount>, // ATA кривой для мем-токена

//     #[account(
//         mut, // Баланс меняется
//         // Проверяем, что владелец - это PDA кривой
//         constraint = n_dollar_treasury.owner == bonding_curve.key() @ BondingCurveError::InvalidTokenAccountOwner,
//     )]
//     pub n_dollar_treasury: Account<'info, TokenAccount>, // ATA кривой для N-Dollar (Казна)

//     #[account(
//         mut, // Баланс меняется
//         constraint = user_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
//         constraint = user_token_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
//     )]
//     pub user_token_account: Account<'info, TokenAccount>, // ATA пользователя для мем-токена

//     #[account(
//         mut, // Баланс меняется
//         constraint = user_n_dollar_account.mint == n_dollar_mint.key() @ BondingCurveError::InvalidTokenAccount,
//         constraint = user_n_dollar_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
//     )]
//     pub user_n_dollar_account: Account<'info, TokenAccount>, // ATA пользователя для N-Dollar

//     // bonding_curve_authority как отдельный аккаунт больше не нужен

//     #[account(mut)]
//     pub user_authority: Signer<'info>, // Пользователь, который покупает/продает

//     pub token_program: Program<'info, Token>,
// }

// // --- Состояние ---

// #[account]
// #[derive(InitSpace)] // InitSpace сам посчитает размер
// pub struct BondingCurve {
//     pub authority: Pubkey,                   // 32 Создатель кривой / Payer инициализации
//     pub mint: Pubkey,                        // 32 Мем-токен
//     pub n_dollar_mint: Pubkey,               // 32 N-Dollar токен
//     pub bonding_curve_token_account: Pubkey, // 32 ATA кривой для мем-токена
//     pub n_dollar_treasury: Pubkey,           // 32 ATA кривой для N-Dollar (казна)
//     pub initial_bonding_supply: u64,         // 8 Начальное кол-во мем-токенов на кривой
//     pub slope_numerator: u128,               // 16 Числитель наклона (scaled)
//     pub slope_denominator: u128,             // 16 Знаменатель наклона (supply)
//     pub intercept_scaled: u128,              // 16 Пересечение с осью Y (scaled)
//     pub token_decimals: u8,                  // 1 Децималы мем-токена
//     pub n_dollar_decimals: u8,               // 1 Децималы N-Dollar
//     pub is_initialized: bool,                // 1 Флаг инициализации
//     pub bump: u8,                            // 1 Bump для PDA bonding_curve
// }

// // --- Ошибки ---
// #[error_code]
// pub enum BondingCurveError {
//     #[msg("Bonding curve account is not initialized.")]
//     NotInitialized,
//     #[msg("Input amount cannot be zero.")]
//     ZeroAmount,
//     #[msg("Calculation resulted in an overflow.")]
//     CalculationOverflow,
//     #[msg("Insufficient token liquidity on the bonding curve.")]
//     InsufficientLiquidity,
//     #[msg("Insufficient funds in user account.")]
//     InsufficientFunds,
//     #[msg("Insufficient N-Dollar in treasury to cover the sale.")]
//     InsufficientTreasury,
//     #[msg("Invalid mint account provided.")]
//     InvalidMintAccount,
//     #[msg("Invalid token account provided.")]
//     InvalidTokenAccount,
//     #[msg("Invalid token account owner.")]
//     InvalidTokenAccountOwner,
//     #[msg("Initial supply in bonding token account does not match expected amount.")]
//     IncorrectInitialSupply, // Не используется пока, но можно добавить
//     #[msg("Bonding curve token account is empty.")]
//     BondingAccountEmpty,
//     #[msg("Cannot sell more tokens than have been bought from the curve.")]
//     CannotSellMoreThanSold, // Пока не реализована логика отслеживания
// }



use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use std::convert::TryInto;

declare_id!("GvFsepxBQ2q8xZ3PYYDooMdnMBzWQKkpKavzT7vM83rZ");

const PRECISION_FACTOR: u128 = 1_000_000_000_000; // 10^12

// Helper function for ceiling division: ceil(a / b)
fn ceil_div(a: u128, b: u128) -> Result<u128> {
    if b == 0 {
        msg!("Error: Division by zero in ceil_div");
        return Err(BondingCurveError::CalculationOverflow.into()); // Or a specific division error
    }
    a.checked_add(b.checked_sub(1).ok_or(BondingCurveError::CalculationOverflow)?)
        .ok_or(BondingCurveError::CalculationOverflow)?
        .checked_div(b)
        .ok_or(BondingCurveError::CalculationOverflow.into())
}

// Helper function for floor division (standard integer division)
fn floor_div(a: u128, b: u128) -> Result<u128> {
    if b == 0 {
        msg!("Error: Division by zero in floor_div");
        return Err(BondingCurveError::CalculationOverflow.into()); // Or a specific division error
    }
    a.checked_div(b).ok_or(BondingCurveError::CalculationOverflow.into())
}


#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize_curve(ctx: Context<InitializeCurve>) -> Result<()> {
        // --- Existing initialization logic ---
        let curve = &mut ctx.accounts.bonding_curve;
        let bonding_token_account = &ctx.accounts.bonding_curve_token_account;
        let mint = &ctx.accounts.mint;
        let n_dollar_mint = &ctx.accounts.n_dollar_mint;

        curve.is_initialized = true;
        curve.authority = ctx.accounts.authority.key();
        curve.mint = mint.key();
        curve.n_dollar_mint = n_dollar_mint.key();
        curve.bonding_curve_token_account = bonding_token_account.key();
        curve.n_dollar_treasury = ctx.accounts.n_dollar_treasury.key();
        curve.bump = ctx.bumps.bonding_curve;
        curve.token_decimals = mint.decimals;
        curve.n_dollar_decimals = n_dollar_mint.decimals;
        curve.initial_bonding_supply = bonding_token_account.amount;
        require!(
            curve.initial_bonding_supply > 0,
            BondingCurveError::BondingAccountEmpty
        );

        // --- Example Curve Parameters (Linear) ---
        // Adjust these based on your desired price curve
        let target_price_scaled: u128 = 100_000_000_000; // 0.1 * 10^12 (intercept)
        let max_price_scaled: u128 = 1_000_000_000_000; // 1.0 * 10^12 (price at full depletion)

        let initial_supply_u128: u128 = curve.initial_bonding_supply.into();
        require!(initial_supply_u128 > 0, BondingCurveError::CalculationOverflow); // Should be caught earlier

        let price_diff_scaled = max_price_scaled
            .checked_sub(target_price_scaled)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        curve.slope_numerator = price_diff_scaled;     // scaled by PRECISION
        curve.slope_denominator = initial_supply_u128; // in lamports
        curve.intercept_scaled = target_price_scaled;  // scaled by PRECISION

        msg!("Bonding curve initialized:");
        msg!("  Initial Supply (lamports): {}", curve.initial_bonding_supply);
        msg!("  Slope Numerator (scaled): {}", curve.slope_numerator);
        msg!("  Slope Denominator (lamports): {}", curve.slope_denominator);
        msg!("  Intercept Scaled: {}", curve.intercept_scaled);

        Ok(())
    }

    // pub fn buy(ctx: Context<BuySell>, amount_to_buy: u64) -> Result<()> {
    //     msg!("Executing Buy for {} lamports", amount_to_buy);
    //     let curve = &ctx.accounts.bonding_curve;
    //     require!(curve.is_initialized, BondingCurveError::NotInitialized);
    //     require!(amount_to_buy > 0, BondingCurveError::ZeroAmount);

    //     // Reload data
    //     ctx.accounts.bonding_curve_token_account.reload()?;
    //     ctx.accounts.user_n_dollar_account.reload()?;

    //     let current_supply = ctx.accounts.bonding_curve_token_account.amount; // x1
    //     require!(
    //         current_supply >= amount_to_buy,
    //         BondingCurveError::InsufficientLiquidity
    //     );

    //     let final_supply = current_supply
    //         .checked_sub(amount_to_buy) // x0 = x1 - dx
    //         .ok_or(BondingCurveError::CalculationOverflow)?;

    //     let m_num = curve.slope_numerator;
    //     let m_den = curve.slope_denominator;
    //     let c_scaled = curve.intercept_scaled;
    //     let dx: u128 = amount_to_buy.into();
    //     let x1: u128 = current_supply.into();
    //     let x0: u128 = final_supply.into();

    //     // --- Calculate Term 1 (from slope m) ---
    //     // term1_lamports = [m_num * dx * (x1 + x0)] / [m_den * 2 * PRECISION_FACTOR]
    //     msg!("Calculating term 1 (slope component)...");

    //     let sum_supplies = x1.checked_add(x0).ok_or(BondingCurveError::CalculationOverflow)?;
    //     msg!("  x1 + x0 = {}", sum_supplies);

    //     // Calculate numerator: m_num * dx * sum_supplies
    //     // Do multiplication step-by-step to check overflow
    //     let temp_num1 = m_num.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
    //     let numerator1 = temp_num1.checked_mul(sum_supplies).ok_or(BondingCurveError::CalculationOverflow)?;
    //     msg!("  Numerator1 (m_num * dx * (x1+x0)) = {}", numerator1);


    //     // Calculate denominator: m_den * 2 * PRECISION_FACTOR
    //     let temp_den1 = m_den.checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?;
    //     let denominator1 = temp_den1.checked_mul(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;
    //      msg!("  Denominator1 (m_den * 2 * PRECISION_FACTOR) = {}", denominator1);


    //     // Calculate term1 using ceiling division (charge user more in case of fractions)
    //     let term1_lamports = ceil_div(numerator1, denominator1)?;
    //     msg!("  Term1 Lamports (Ceiling) = {}", term1_lamports);


    //     // --- Calculate Term 2 (from intercept c) ---
    //     // term2_lamports = [c_scaled * dx] / [PRECISION_FACTOR]
    //      msg!("Calculating term 2 (intercept component)...");

    //     let numerator2 = c_scaled.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
    //     let denominator2 = PRECISION_FACTOR;
    //     msg!("  Numerator2 (c_scaled * dx) = {}", numerator2);
    //     msg!("  Denominator2 (PRECISION_FACTOR) = {}", denominator2);

    //     // Calculate term2 using ceiling division
    //     let term2_lamports = ceil_div(numerator2, denominator2)?;
    //     msg!("  Term2 Lamports (Ceiling) = {}", term2_lamports);


    //     // --- Calculate Total Cost ---
    //     let total_cost_lamports_u128 = term1_lamports
    //         .checked_add(term2_lamports)
    //         .ok_or(BondingCurveError::CalculationOverflow)?;
    //     msg!("Total Cost (u128) = {}", total_cost_lamports_u128);


    //     // --- Convert to u64 for transfer ---
    //     let total_cost_lamports: u64 = total_cost_lamports_u128
    //         .try_into()
    //         .map_err(|_| {
    //             msg!("!!! Overflow: Final cost {} exceeds u64::MAX", total_cost_lamports_u128);
    //             BondingCurveError::CalculationOverflow
    //         })?;
    //     msg!("Final Cost (u64) = {}", total_cost_lamports);




    pub fn buy(ctx: Context<BuySell>, amount_to_buy: u64) -> Result<()> {
        msg!("Executing Buy for {} lamports", amount_to_buy);
        let curve = &ctx.accounts.bonding_curve;
        // ... (проверки и загрузка данных) ...

        let current_supply = ctx.accounts.bonding_curve_token_account.amount; // x1
        require!(current_supply >= amount_to_buy, BondingCurveError::InsufficientLiquidity);
        let final_supply = current_supply.checked_sub(amount_to_buy).ok_or(BondingCurveError::CalculationOverflow)?; // x0

        let m_num = curve.slope_numerator;
        let m_den = curve.slope_denominator;
        let c_scaled = curve.intercept_scaled;
        let dx: u128 = amount_to_buy.into();
        let x1: u128 = current_supply.into();
        let x0: u128 = final_supply.into();

        // --- Calculate Term 1 (from slope m) ---
        msg!("Calculating term 1 (slope component)...");

        // Рассчитываем m_num * dx
        let term1_part1 = m_num.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  m_num * dx = {}", term1_part1);

        // Рассчитываем m_den * 2
        let term1_part2 = m_den.checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  m_den * 2 = {}", term1_part2);

        // Вычисляем промежуточное отношение (с округлением вверх)
        // intermediate_ratio_scaled = ceil( (m_num * dx) / (m_den * 2) )
        // Это отношение все еще масштабировано PRECISION_FACTOR, т.к. m_num был масштабирован
        let intermediate_ratio_scaled = ceil_div(term1_part1, term1_part2)?;
        msg!("  Intermediate Ratio (scaled, ceiling) = {}", intermediate_ratio_scaled);

        // Рассчитываем сумму саплаев
        let sum_supplies = x1.checked_add(x0).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  x1 + x0 = {}", sum_supplies);

        // Теперь умножаем промежуточное отношение на sum_supplies
        // numerator_final = intermediate_ratio_scaled * sum_supplies
        let numerator_final = intermediate_ratio_scaled.checked_mul(sum_supplies).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  Numerator Final (Intermediate Ratio * Sum Supplies) = {}", numerator_final);

        // Финальное деление на PRECISION_FACTOR (с округлением вверх)
        let term1_lamports = ceil_div(numerator_final, PRECISION_FACTOR)?;
        msg!("  Term1 Lamports (Ceiling) = {}", term1_lamports);


        // --- Calculate Term 2 (from intercept c) ---
        msg!("Calculating term 2 (intercept component)...");
        let numerator2 = c_scaled.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
        let denominator2 = PRECISION_FACTOR;
        msg!("  Numerator2 (c_scaled * dx) = {}", numerator2);
        let term2_lamports = ceil_div(numerator2, denominator2)?;
        msg!("  Term2 Lamports (Ceiling) = {}", term2_lamports);


        // --- Calculate Total Cost ---
        let total_cost_lamports_u128 = term1_lamports
            .checked_add(term2_lamports)
            .ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("Total Cost (u128) = {}", total_cost_lamports_u128);

        // --- Convert to u64 ---
        let total_cost_lamports: u64 = total_cost_lamports_u128
            .try_into()
            // ... (обработка ошибки конвертации) ...
            .map_err(|_| {
                 msg!("!!! Overflow: Final cost {} exceeds u64::MAX", total_cost_lamports_u128);
                 BondingCurveError::CalculationOverflow
             })?;
        msg!("Final Cost (u64) = {}", total_cost_lamports);
        // Check user N-Dollar balance
        require!(
            ctx.accounts.user_n_dollar_account.amount >= total_cost_lamports,
            BondingCurveError::InsufficientFunds
        );

        // --- Perform Transfers ---
        msg!("Performing transfers...");
        // 1. N-Dollar User -> Treasury
        let cpi_accounts_ndollar = Transfer {
            from: ctx.accounts.user_n_dollar_account.to_account_info(),
            to: ctx.accounts.n_dollar_treasury.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_ndollar),
            total_cost_lamports
        )?;

        // 2. Token Curve -> User
        let bump = curve.bump;
        let mint_key = curve.mint.key();
        let seeds = &[
            b"bonding_curve".as_ref(),
            mint_key.as_ref(),
            &[bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts_token = Transfer {
            from: ctx.accounts.bonding_curve_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.bonding_curve.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts_token, signer_seeds),
            amount_to_buy // original u64 amount
        )?;

        msg!("Buy successful.");
        Ok(())
    }

    // pub fn sell(ctx: Context<BuySell>, amount_to_sell: u64) -> Result<()> {
    //     msg!("Executing Sell for {} lamports", amount_to_sell);
    //     let curve = &ctx.accounts.bonding_curve;
    //     require!(curve.is_initialized, BondingCurveError::NotInitialized);
    //     require!(amount_to_sell > 0, BondingCurveError::ZeroAmount);

    //     // Reload data
    //     ctx.accounts.bonding_curve_token_account.reload()?;
    //     ctx.accounts.user_token_account.reload()?;
    //     ctx.accounts.n_dollar_treasury.reload()?;

    //     // Check user token balance
    //     require!(
    //         ctx.accounts.user_token_account.amount >= amount_to_sell,
    //         BondingCurveError::InsufficientFunds
    //     );

    //     let current_supply = ctx.accounts.bonding_curve_token_account.amount; // x0
    //     let final_supply = current_supply
    //         .checked_add(amount_to_sell) // x1 = x0 + dx
    //         .ok_or(BondingCurveError::CalculationOverflow)?;

    //     let m_num = curve.slope_numerator;
    //     let m_den = curve.slope_denominator;
    //     let c_scaled = curve.intercept_scaled;
    //     let dx: u128 = amount_to_sell.into();
    //     let x0: u128 = current_supply.into();
    //     let x1: u128 = final_supply.into();

    //     // --- Calculate Term 1 (from slope m) ---
    //     // term1_lamports = [m_num * dx * (x1 + x0)] / [m_den * 2 * PRECISION_FACTOR]
    //     msg!("Calculating term 1 (slope component)...");

    //     let sum_supplies = x1.checked_add(x0).ok_or(BondingCurveError::CalculationOverflow)?;
    //      msg!("  x1 + x0 = {}", sum_supplies);

    //     // Calculate numerator: m_num * dx * sum_supplies
    //     let temp_num1 = m_num.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
    //     let numerator1 = temp_num1.checked_mul(sum_supplies).ok_or(BondingCurveError::CalculationOverflow)?;
    //     msg!("  Numerator1 (m_num * dx * (x1+x0)) = {}", numerator1);


    //     // Calculate denominator: m_den * 2 * PRECISION_FACTOR
    //     let temp_den1 = m_den.checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?;
    //     let denominator1 = temp_den1.checked_mul(PRECISION_FACTOR).ok_or(BondingCurveError::CalculationOverflow)?;
    //     msg!("  Denominator1 (m_den * 2 * PRECISION_FACTOR) = {}", denominator1);

    //     // Calculate term1 using floor division (give user less in case of fractions)
    //     let term1_lamports = floor_div(numerator1, denominator1)?;
    //     msg!("  Term1 Lamports (Floor) = {}", term1_lamports);


    //     // --- Calculate Term 2 (from intercept c) ---
    //     // term2_lamports = [c_scaled * dx] / [PRECISION_FACTOR]
    //     msg!("Calculating term 2 (intercept component)...");

    //     let numerator2 = c_scaled.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
    //     let denominator2 = PRECISION_FACTOR;
    //     msg!("  Numerator2 (c_scaled * dx) = {}", numerator2);
    //     msg!("  Denominator2 (PRECISION_FACTOR) = {}", denominator2);

    //     // Calculate term2 using floor division
    //     let term2_lamports = floor_div(numerator2, denominator2)?;
    //     msg!("  Term2 Lamports (Floor) = {}", term2_lamports);


    //     // --- Calculate Total Proceeds ---
    //     let total_proceeds_lamports_u128 = term1_lamports
    //         .checked_add(term2_lamports)
    //         .ok_or(BondingCurveError::CalculationOverflow)?;
    //     msg!("Total Proceeds (u128) = {}", total_proceeds_lamports_u128);


    //     // --- Convert to u64 for transfer ---
    //     let total_proceeds_lamports: u64 = total_proceeds_lamports_u128
    //         .try_into()
    //          .map_err(|_| {
    //              msg!("!!! Overflow: Final proceeds {} exceeds u64::MAX", total_proceeds_lamports_u128);
    //              BondingCurveError::CalculationOverflow
    //          })?;
    //     msg!("Final Proceeds (u64) = {}", total_proceeds_lamports);

    pub fn sell(ctx: Context<BuySell>, amount_to_sell: u64) -> Result<()> {
        msg!("Executing Sell for {} lamports", amount_to_sell);
        let curve = &ctx.accounts.bonding_curve;
        // ... (проверки и загрузка данных) ...

        let current_supply = ctx.accounts.bonding_curve_token_account.amount; // x0
        require!(ctx.accounts.user_token_account.amount >= amount_to_sell, BondingCurveError::InsufficientFunds);
        let final_supply = current_supply.checked_add(amount_to_sell).ok_or(BondingCurveError::CalculationOverflow)?; // x1

        let m_num = curve.slope_numerator;
        let m_den = curve.slope_denominator;
        let c_scaled = curve.intercept_scaled;
        let dx: u128 = amount_to_sell.into();
        let x0: u128 = current_supply.into();
        let x1: u128 = final_supply.into();

        // --- Calculate Term 1 (from slope m) ---
        msg!("Calculating term 1 (slope component)...");
        let term1_part1 = m_num.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
         msg!("  m_num * dx = {}", term1_part1);
        let term1_part2 = m_den.checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?;
         msg!("  m_den * 2 = {}", term1_part2);

        // Используем floor_div для продажи
        let intermediate_ratio_scaled = floor_div(term1_part1, term1_part2)?;
        msg!("  Intermediate Ratio (scaled, floor) = {}", intermediate_ratio_scaled);

        let sum_supplies = x1.checked_add(x0).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  x1 + x0 = {}", sum_supplies);

        let numerator_final = intermediate_ratio_scaled.checked_mul(sum_supplies).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  Numerator Final (Intermediate Ratio * Sum Supplies) = {}", numerator_final);

        // Используем floor_div для продажи
        let term1_lamports = floor_div(numerator_final, PRECISION_FACTOR)?;
        msg!("  Term1 Lamports (Floor) = {}", term1_lamports);

        // --- Calculate Term 2 (from intercept c) ---
        msg!("Calculating term 2 (intercept component)...");
        let numerator2 = c_scaled.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
         msg!("  Numerator2 (c_scaled * dx) = {}", numerator2);
        let denominator2 = PRECISION_FACTOR;
        // Используем floor_div для продажи
        let term2_lamports = floor_div(numerator2, denominator2)?;
        msg!("  Term2 Lamports (Floor) = {}", term2_lamports);

        // --- Calculate Total Proceeds ---
        let total_proceeds_lamports_u128 = term1_lamports
            .checked_add(term2_lamports)
            .ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("Total Proceeds (u128) = {}", total_proceeds_lamports_u128);

        // --- Convert to u64 ---
         let total_proceeds_lamports: u64 = total_proceeds_lamports_u128
            .try_into()
            // ... (обработка ошибки конвертации) ...
            .map_err(|_| {
                 msg!("!!! Overflow: Final proceeds {} exceeds u64::MAX", total_proceeds_lamports_u128);
                 BondingCurveError::CalculationOverflow
             })?;
        msg!("Final Proceeds (u64) = {}", total_proceeds_lamports);

        // Check treasury N-Dollar balance
        require!(
            ctx.accounts.n_dollar_treasury.amount >= total_proceeds_lamports,
            BondingCurveError::InsufficientTreasury
        );

        // --- Perform Transfers ---
        msg!("Performing transfers...");

        // 1. Token User -> Curve
        let cpi_accounts_token = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.bonding_curve_token_account.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        token::transfer(
             CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_token),
             amount_to_sell // original u64 amount
        )?;

        // 2. N-Dollar Treasury -> User
        let bump = curve.bump;
        let mint_key = curve.mint.key();
        let seeds = &[
            b"bonding_curve".as_ref(),
            mint_key.as_ref(),
            &[bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts_ndollar = Transfer {
            from: ctx.accounts.n_dollar_treasury.to_account_info(),
            to: ctx.accounts.user_n_dollar_account.to_account_info(),
            authority: ctx.accounts.bonding_curve.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts_ndollar, signer_seeds),
            total_proceeds_lamports
        )?;

        msg!("Sell successful.");
        Ok(())
    }
}


// --- Аккаунты (НЕ ТРОГАЕМ, как просили) ---
#[derive(Accounts)]
pub struct InitializeCurve<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + BondingCurve::INIT_SPACE,
        seeds = [b"bonding_curve", mint.key().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    pub mint: Account<'info, Mint>,
    pub n_dollar_mint: Account<'info, Mint>,
    #[account(
        constraint = bonding_curve_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = n_dollar_mint,
        associated_token::authority = bonding_curve,
    )]
    pub n_dollar_treasury: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuySell<'info> {
    #[account(
        mut,
        seeds = [b"bonding_curve", mint.key().as_ref()],
        bump = bonding_curve.bump,
        has_one = mint @ BondingCurveError::InvalidMintAccount,
        has_one = n_dollar_mint @ BondingCurveError::InvalidMintAccount,
        constraint = bonding_curve.bonding_curve_token_account == bonding_curve_token_account.key() @ BondingCurveError::InvalidTokenAccount,
        constraint = bonding_curve.n_dollar_treasury == n_dollar_treasury.key() @ BondingCurveError::InvalidTokenAccount,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    pub mint: Account<'info, Mint>,
    pub n_dollar_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = bonding_curve_token_account.owner == bonding_curve.key() @ BondingCurveError::InvalidTokenAccountOwner,
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = n_dollar_treasury.owner == bonding_curve.key() @ BondingCurveError::InvalidTokenAccountOwner,
    )]
    pub n_dollar_treasury: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_token_account.mint == mint.key() @ BondingCurveError::InvalidTokenAccount,
        constraint = user_token_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_n_dollar_account.mint == n_dollar_mint.key() @ BondingCurveError::InvalidTokenAccount,
        constraint = user_n_dollar_account.owner == user_authority.key() @ BondingCurveError::InvalidTokenAccountOwner,
    )]
    pub user_n_dollar_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// --- Состояние (НЕ ТРОГАЕМ) ---
#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub n_dollar_mint: Pubkey,
    pub bonding_curve_token_account: Pubkey,
    pub n_dollar_treasury: Pubkey,
    pub initial_bonding_supply: u64,
    pub slope_numerator: u128,
    pub slope_denominator: u128,
    pub intercept_scaled: u128,
    pub token_decimals: u8,
    pub n_dollar_decimals: u8,
    pub is_initialized: bool,
    pub bump: u8,
}

// --- Ошибки (НЕ ТРОГАЕМ) ---
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