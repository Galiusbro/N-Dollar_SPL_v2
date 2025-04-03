// referral_program/src/lib.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

declare_id!("DMQh8Evpe3y4DzAWxx1rhLuGpnZGDvFSPLJvD9deQQfX");

#[program]
pub mod referral_program {
    use super::*;

    pub fn process_referral(ctx: Context<ProcessReferral>) -> Result<()> {
        msg!("Processing referral...");

        // Check if this referral has already been processed
        let referral_status = &ctx.accounts.referral_status;
        require!(!referral_status.processed, ErrorCode::AlreadyProcessed);

        // Get token decimals to calculate the reward
        let token_decimals = ctx.accounts.mint.decimals;
        let reward_amount = 1 * 10u64.pow(token_decimals as u32); // 1 token with decimals
        msg!("Reward amount (lamports): {}", reward_amount);

        // Check if there are enough funds in the treasury
        let required_total_reward = reward_amount
            .checked_mul(2) // Reward for referrer and referee
            .ok_or(ErrorCode::CalculationOverflow)?;
        require!(
            ctx.accounts.referral_treasury.amount >= required_total_reward,
            ErrorCode::InsufficientTreasuryBalance
        );

        // Get bump for the treasury authority PDA
        let mint_key = ctx.accounts.mint.key();
        let (_authority_pda, authority_bump) = Pubkey::find_program_address(
            &[b"referral".as_ref(), mint_key.as_ref()],
            ctx.program_id,
        );

        // Create signer seeds
        let authority_seeds = &[
            b"referral".as_ref(),
            mint_key.as_ref(),
            &[authority_bump],
        ];
        let signer = &[&authority_seeds[..]];

        // 1. Pay reward to the referrer
        msg!(
            "Paying reward to referrer: {}",
            ctx.accounts.referrer_token_account.key()
        );
        let cpi_accounts_referrer = Transfer {
            from: ctx.accounts.referral_treasury.to_account_info(),
            to: ctx.accounts.referrer_token_account.to_account_info(),
            authority: ctx.accounts.referral_authority.to_account_info(), // PDA signs
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_referrer =
            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts_referrer, signer);
        token::transfer(cpi_ctx_referrer, reward_amount)?;

        // 2. Pay reward to the referee
        msg!(
            "Paying reward to referee: {}",
            ctx.accounts.referee_token_account.key()
        );
        let cpi_accounts_referee = Transfer {
            from: ctx.accounts.referral_treasury.to_account_info(),
            to: ctx.accounts.referee_token_account.to_account_info(),
            authority: ctx.accounts.referral_authority.to_account_info(), // PDA signs
        };
        let cpi_ctx_referee =
            CpiContext::new_with_signer(cpi_program, cpi_accounts_referee, signer);
        token::transfer(cpi_ctx_referee, reward_amount)?;

        // Mark referral as processed
        let referral_status = &mut ctx.accounts.referral_status;
        referral_status.processed = true;
        referral_status.referrer = ctx.accounts.referrer.key();
        referral_status.referee = ctx.accounts.referee.key();
        referral_status.mint = ctx.accounts.mint.key();
        msg!(
            "Referral processed for referee: {}",
            ctx.accounts.referee.key()
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ProcessReferral<'info> {
    // --- Accounts for validation and payouts ---
    pub mint: Account<'info, Mint>, // Token used for reward

    /// CHECK: PDA owning the treasury. Verified via seeds + program_id.
    #[account(
        seeds = [b"referral".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub referral_authority: AccountInfo<'info>,

    #[account(
        mut, // Funds are debited from here
        associated_token::mint = mint,
        associated_token::authority = referral_authority, // Verify treasury owner
        constraint = referral_treasury.amount > 0 @ ErrorCode::InsufficientTreasuryBalance // Extra check
    )]
    pub referral_treasury: Account<'info, TokenAccount>, // Referral program treasury

    /// CHECK: Referrer - just an address, nothing to check except not equal to referee?
    #[account(
         constraint = referrer.key() != referee.key() @ ErrorCode::ReferrerCannotBeReferee
    )]
    pub referrer: AccountInfo<'info>, // Who invited

    /// CHECK: Referee - new user. Verified via referral_status.
    pub referee: AccountInfo<'info>, // Who was invited

    // Referrer's ATA to receive the reward
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = referrer, // Ensure this is the referrer's ATA
    )]
    pub referrer_token_account: Account<'info, TokenAccount>,

    // Referee's ATA to receive the reward
    // May not exist, hence init_if_needed
    #[account(
        init_if_needed,
        payer = payer, // Who pays for ATA creation if it doesn't exist
        // mut,
        associated_token::mint = mint,
        associated_token::authority = referee, // Ensure this is the referee's ATA
    )]
    pub referee_token_account: Account<'info, TokenAccount>,

    // --- Account for tracking status ---
    #[account(
        init, // Created on first processing
        payer = payer,
        space = 8 + ReferralStatus::INIT_SPACE,
        seeds = [b"status".as_ref(), mint.key().as_ref(), referee.key().as_ref()], // Unique for mint+referee
        bump
    )]
    pub referral_status: Account<'info, ReferralStatus>,

    // --- System Accounts ---
    #[account(mut)]
    pub payer: Signer<'info>, // Who pays for creating status and referee_token_account
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// State for tracking processed referrals
#[account]
#[derive(InitSpace)]
pub struct ReferralStatus {
    pub processed: bool,     // 1
    pub referrer: Pubkey,    // 32
    pub referee: Pubkey,     // 32
    pub mint: Pubkey,        // 32
}

#[error_code]
pub enum ErrorCode {
    #[msg("Referral already processed for this referee and mint.")]
    AlreadyProcessed,
    #[msg("Insufficient balance in treasury for rewards.")]
    InsufficientTreasuryBalance,
    #[msg("Calculation overflow.")]
    CalculationOverflow,
    #[msg("Referrer cannot be the same as the referee.")]
    ReferrerCannotBeReferee,
}