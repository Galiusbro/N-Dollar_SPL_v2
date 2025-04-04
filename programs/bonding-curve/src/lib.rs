use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use std::convert::TryInto;

declare_id!("GvFsepxBQ2q8xZ3PYYDooMdnMBzWQKkpKavzT7vM83rZ");

const PRECISION_FACTOR: u128 = 1_000_000_000_000; // 10^12

// --- Curve specification constants from the document ---
const TOTAL_BONDING_TOKENS_FOR_CURVE: u64 = 30_000_000;
const START_PRICE_NUMERATOR: u128 = 5;
const START_PRICE_DENOMINATOR: u128 = 100_000; // 0.00005
const TARGET_PRICE_NUMERATOR: u128 = 1;
const TARGET_PRICE_DENOMINATOR: u128 = 1; // 1.0

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
        msg!("Initializing Bonding Curve (Linear: 0.00005 -> 1 N$ over 30M tokens)...");
        let curve = &mut ctx.accounts.bonding_curve;
        let bonding_token_account = &ctx.accounts.bonding_curve_token_account;
        let mint = &ctx.accounts.mint;
        let n_dollar_mint = &ctx.accounts.n_dollar_mint;

        // --- Basic Initialization ---
        curve.is_initialized = true;
        curve.authority = ctx.accounts.authority.key();
        curve.mint = mint.key();
        curve.n_dollar_mint = n_dollar_mint.key();
        curve.bonding_curve_token_account = bonding_token_account.key();
        curve.n_dollar_treasury = ctx.accounts.n_dollar_treasury.key();
        curve.bump = ctx.bumps.bonding_curve;
        curve.token_decimals = mint.decimals;
        curve.n_dollar_decimals = n_dollar_mint.decimals;

        // --- Calculate and Verify Initial Supply for the Curve ---
        let token_decimal_factor_u64 = 10u64.pow(curve.token_decimals as u32);
        let supply_target_lamports = TOTAL_BONDING_TOKENS_FOR_CURVE
            .checked_mul(token_decimal_factor_u64)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        let current_balance_on_curve = bonding_token_account.amount;
        msg!("Expected initial supply (lamports based on 30M tokens): {}", supply_target_lamports);
        msg!("Actual balance in bonding token account: {}", current_balance_on_curve);

        require!(
            current_balance_on_curve == supply_target_lamports,
            BondingCurveError::IncorrectInitialSupply
        );

        curve.initial_bonding_supply = current_balance_on_curve; // Store the verified supply
        require!(
            curve.initial_bonding_supply > 0,
            BondingCurveError::BondingAccountEmpty // Should be caught by previous check, but good sanity check
        );

        // --- Calculate Linear Curve Parameters P(y) = m*y + c ---
        // y = tokens sold (0 to initial_bonding_supply)
        // c = start_price (at y=0)
        // P(initial_supply) = target_price (at y=initial_supply)
        // m = (target_price - start_price) / initial_supply

        let initial_supply_u128: u128 = curve.initial_bonding_supply.into();

        // Calculate c_scaled (start price * PRECISION)
        let start_price_scaled = START_PRICE_NUMERATOR
            .checked_mul(PRECISION_FACTOR)
            .ok_or(BondingCurveError::CalculationOverflow)?
            .checked_div(START_PRICE_DENOMINATOR)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        // Calculate P(Y_max)_scaled (target price * PRECISION)
        let target_price_scaled = TARGET_PRICE_NUMERATOR
            .checked_mul(PRECISION_FACTOR)
            .ok_or(BondingCurveError::CalculationOverflow)?
            .checked_div(TARGET_PRICE_DENOMINATOR)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        // Calculate slope numerator (target_price - start_price) * PRECISION
        let price_diff_scaled = target_price_scaled
            .checked_sub(start_price_scaled)
            .ok_or(BondingCurveError::CalculationOverflow)?;

        // --- Store Curve Parameters ---
        curve.slope_numerator = price_diff_scaled;     // (P_target - P_start) * PRECISION
        curve.slope_denominator = initial_supply_u128; // Y_max (30M tokens in lamports)
        curve.intercept_scaled = start_price_scaled;  // P_start * PRECISION

        msg!("Bonding curve initialized (Price based on SOLD tokens, target 30M):");
        msg!("  Initial Supply (lamports, Y_max): {}", curve.initial_bonding_supply);
        msg!("  Start Price (scaled, c): {}", curve.intercept_scaled); // P(0)
        msg!("  Target Price (scaled, P(Y_max)): {}", target_price_scaled);
        msg!("  Slope Numerator (m * Y_max, scaled): {}", curve.slope_numerator);
        msg!("  Slope Denominator (Y_max, lamports): {}", curve.slope_denominator);

        Ok(())
    }

    pub fn buy(ctx: Context<BuySell>, amount_to_buy: u64) -> Result<()> {
        msg!("Executing Buy for {} lamports", amount_to_buy);
        let curve = &ctx.accounts.bonding_curve;
        require!(curve.is_initialized, BondingCurveError::NotInitialized);
        require!(amount_to_buy > 0, BondingCurveError::ZeroAmount);

        // Reload data
        ctx.accounts.bonding_curve_token_account.reload()?;
        ctx.accounts.user_n_dollar_account.reload()?;

        let current_supply = ctx.accounts.bonding_curve_token_account.amount; // x1
        require!(
            current_supply >= amount_to_buy,
            BondingCurveError::InsufficientLiquidity
        );

        let final_supply = current_supply
            .checked_sub(amount_to_buy) // x0 = x1 - dx
            .ok_or(BondingCurveError::CalculationOverflow)?;

        let initial_supply = curve.initial_bonding_supply;
        let dx: u128 = amount_to_buy.into();
        let _x1: u128 = current_supply.into();
        let _x0: u128 = final_supply.into();

        // Calculate tokens sold before (y0) and after (y1) the buy
        // y = initial_supply - x
        let y0 = initial_supply.checked_sub(current_supply) // initial - x1
                 .ok_or(BondingCurveError::CalculationOverflow)?;
        let y1 = initial_supply.checked_sub(final_supply)   // initial - x0 = y0 + dx
                 .ok_or(BondingCurveError::CalculationOverflow)?;

        msg!("  Tokens Sold Before (y0): {}", y0);
        msg!("  Tokens Sold After (y1): {}", y1);
        msg!("  Amount Bought (dx): {}", dx);

        // Calculate cost using integral of P(y) = m*y + c
        // Cost = Integral[P(y) dy] from y0 to y1
        // Cost = [m/2 * y^2 + c*y] from y0 to y1
        // Cost = m/2 * (y1^2 - y0^2) + c * (y1 - y0)
        // Cost = m/2 * dx * (y1 + y0) + c * dx

        let m_num = curve.slope_numerator;   // target_price - start_price (scaled)
        let m_den = curve.slope_denominator; // initial_supply (lamports)
        let c_scaled = curve.intercept_scaled; // start_price (scaled)

        // --- Calculate Term 1 (from slope m) ---
        // term1_lamports = ceil( [m/2 * dx * (y1 + y0)] / PRECISION_FACTOR )
        // Rearranged to avoid overflow:
        // term1_lamports = ceil( [ ceil(m_num * dx / (m_den * 2)) * (y1 + y0) ] / PRECISION_FACTOR )
        msg!("Calculating term 1 (slope component)...");

        let term1_intermediate_num = m_num.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  m_num * dx = {}", term1_intermediate_num);
        let term1_intermediate_den = m_den.checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  m_den * 2 = {}", term1_intermediate_den);

        // Calculate ratio scaled by PRECISION, using ceiling for buy
        let intermediate_ratio_scaled = ceil_div(term1_intermediate_num, term1_intermediate_den)?;
        msg!("  Intermediate Ratio (scaled, ceiling) = {}", intermediate_ratio_scaled);

        // Convert y0 and y1 to u128 BEFORE adding
        let y0_u128: u128 = y0.into();
        let y1_u128: u128 = y1.into();
        let sum_y = y1_u128.checked_add(y0_u128).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  y1 + y0 = {}", sum_y);

        // Multiply ratio by sum_y (now both are u128)
        let term1_final_num = intermediate_ratio_scaled.checked_mul(sum_y).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  Numerator Final (Intermediate Ratio * Sum y) = {}", term1_final_num);

        // Divide by PRECISION, using ceiling for buy
        let term1_lamports = ceil_div(term1_final_num, PRECISION_FACTOR)?;
        msg!("  Term1 Lamports (Ceiling) = {}", term1_lamports);

        // --- Calculate Term 2 (from intercept c) ---
        // term2_lamports = ceil( [c_scaled * dx] / PRECISION_FACTOR )
         msg!("Calculating term 2 (intercept component)...");
        let term2_num = c_scaled.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
         msg!("  Numerator2 (c_scaled * dx) = {}", term2_num);
        // Divide by PRECISION, using ceiling for buy
        let term2_lamports = ceil_div(term2_num, PRECISION_FACTOR)?;
        msg!("  Term2 Lamports (Ceiling) = {}", term2_lamports);

        // --- Calculate Total Cost ---
        let total_cost_lamports_u128 = term1_lamports
            .checked_add(term2_lamports)
            .ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("Total Cost (u128) = {}", total_cost_lamports_u128);

        // --- Convert to u64 ---
        let total_cost_lamports: u64 = total_cost_lamports_u128
            .try_into()
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

    pub fn sell(ctx: Context<BuySell>, amount_to_sell: u64) -> Result<()> {
        msg!("Executing Sell for {} lamports", amount_to_sell);
        let curve = &ctx.accounts.bonding_curve;
        require!(curve.is_initialized, BondingCurveError::NotInitialized);
        require!(amount_to_sell > 0, BondingCurveError::ZeroAmount);

        // Reload data
        ctx.accounts.bonding_curve_token_account.reload()?;
        ctx.accounts.user_token_account.reload()?;
        ctx.accounts.n_dollar_treasury.reload()?;

        // Check user token balance
        require!(
            ctx.accounts.user_token_account.amount >= amount_to_sell,
            BondingCurveError::InsufficientFunds
        );

        let current_supply = ctx.accounts.bonding_curve_token_account.amount; // x0
        let final_supply = current_supply
            .checked_add(amount_to_sell) // x1 = x0 + dx
            .ok_or(BondingCurveError::CalculationOverflow)?;

        let initial_supply = curve.initial_bonding_supply;
        let dx: u128 = amount_to_sell.into();
        let _x0: u128 = current_supply.into();
        let _x1: u128 = final_supply.into();

        // Calculate tokens sold before (y1) and after (y0) the sell
        // y = initial_supply - x
        let y1 = initial_supply.checked_sub(current_supply) // initial - x0
                 .ok_or(BondingCurveError::CalculationOverflow)?;
        let y0 = initial_supply.checked_sub(final_supply)   // initial - x1 = y1 - dx
                 .ok_or(BondingCurveError::CalculationOverflow)?;

        msg!("  Tokens Sold Before (y1): {}", y1); // Note: y1 > y0 here
        msg!("  Tokens Sold After (y0): {}", y0);
        msg!("  Amount Sold (dx): {}", dx);


        // Calculate proceeds using integral of P(y) = m*y + c
        // Proceeds = Integral[P(y) dy] from y0 to y1
        // Proceeds = [m/2 * y^2 + c*y] from y0 to y1
        // Proceeds = m/2 * dx * (y1 + y0) + c * dx (Same formula as buy cost)

        let m_num = curve.slope_numerator;   // target_price - start_price (scaled)
        let m_den = curve.slope_denominator; // initial_supply (lamports)
        let c_scaled = curve.intercept_scaled; // start_price (scaled)

        // --- Calculate Term 1 (from slope m) ---
        // term1_lamports = floor( [m/2 * dx * (y1 + y0)] / PRECISION_FACTOR )
        // Rearranged to avoid overflow:
        // term1_lamports = floor( [ floor(m_num * dx / (m_den * 2)) * (y1 + y0) ] / PRECISION_FACTOR )
        msg!("Calculating term 1 (slope component)...");

        let term1_intermediate_num = m_num.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  m_num * dx = {}", term1_intermediate_num);
        let term1_intermediate_den = m_den.checked_mul(2).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  m_den * 2 = {}", term1_intermediate_den);

        // Calculate ratio scaled by PRECISION, using floor for sell
        let intermediate_ratio_scaled = floor_div(term1_intermediate_num, term1_intermediate_den)?;
        msg!("  Intermediate Ratio (scaled, floor) = {}", intermediate_ratio_scaled);

        // Convert y0 and y1 to u128 BEFORE adding
        let y0_u128: u128 = y0.into();
        let y1_u128: u128 = y1.into();
        let sum_y = y1_u128.checked_add(y0_u128).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  y1 + y0 = {}", sum_y);

        // Multiply ratio by sum_y (now both are u128)
        let term1_final_num = intermediate_ratio_scaled.checked_mul(sum_y).ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("  Numerator Final (Intermediate Ratio * Sum y) = {}", term1_final_num);

        // Divide by PRECISION, using floor for sell
        let term1_lamports = floor_div(term1_final_num, PRECISION_FACTOR)?;
        msg!("  Term1 Lamports (Floor) = {}", term1_lamports);

        // --- Calculate Term 2 (from intercept c) ---
        // term2_lamports = floor( [c_scaled * dx] / PRECISION_FACTOR )
         msg!("Calculating term 2 (intercept component)...");
        let term2_num = c_scaled.checked_mul(dx).ok_or(BondingCurveError::CalculationOverflow)?;
         msg!("  Numerator2 (c_scaled * dx) = {}", term2_num);
        // Divide by PRECISION, using floor for sell
        let term2_lamports = floor_div(term2_num, PRECISION_FACTOR)?;
        msg!("  Term2 Lamports (Floor) = {}", term2_lamports);


        // --- Calculate Total Proceeds ---
        let total_proceeds_lamports_u128 = term1_lamports
            .checked_add(term2_lamports)
            .ok_or(BondingCurveError::CalculationOverflow)?;
        msg!("Total Proceeds (u128) = {}", total_proceeds_lamports_u128);

        // --- Convert to u64 ---
         let total_proceeds_lamports: u64 = total_proceeds_lamports_u128
            .try_into()
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


// --- Accounts (DO NOT TOUCH, as requested) ---
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

// --- State (DO NOT TOUCH) ---
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

// --- Errors (DO NOT TOUCH) ---
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
    #[msg("Initial supply in bonding token account does not match expected amount (30M tokens).")]
    IncorrectInitialSupply,
    #[msg("Bonding curve token account is empty or has incorrect initial supply.")]
    BondingAccountEmpty,
    #[msg("Cannot sell more tokens than have been bought from the curve.")]
    CannotSellMoreThanSold,
}