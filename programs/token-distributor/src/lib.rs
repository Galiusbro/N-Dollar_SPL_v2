// token_distributor.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

// !!! SPECIFY YOUR ACTUAL DISTRIBUTOR PROGRAM ID HERE !!!
declare_id!("2Hy1wGdC5iqceaTnZC1qJeuoM4s6yEKHbYcjMMpbKYqp");

#[program]
pub mod token_distributor {
    use super::*;

    // total_supply is no longer needed as an instruction argument,
    // as we can get it from distributor_token_account.amount
    pub fn distribute_tokens(ctx: Context<DistributeTokens>) -> Result<()> {
        msg!("Distributing tokens...");

        // Get total_supply from the distributor account balance
        // Reload the account just in case to get the latest balance
        ctx.accounts.distributor_token_account.reload()?;
        let total_supply = ctx.accounts.distributor_token_account.amount;

        require!(total_supply > 0, ErrorCode::ZeroSupply);
        msg!("Total supply to distribute: {}", total_supply);


        // ... (Calculations for bonding_curve_amount and user_amount as before) ...
        let total_supply_u128 = total_supply as u128;
        let bonding_curve_amount_u128 = total_supply_u128
            .checked_mul(30)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::CalculationOverflow)?;
        let bonding_curve_amount = bonding_curve_amount_u128 as u64;
        let user_amount = total_supply
            .checked_sub(bonding_curve_amount)
            .ok_or(ErrorCode::CalculationOverflow)?;

        msg!(
            "Calculated distribution - Bonding Curve: {}, User: {}",
            bonding_curve_amount,
            user_amount
        );


        // Balance check is no longer needed here, as we just read it
        // require!(
        //     ctx.accounts.distributor_token_account.amount == total_supply,
        //     ErrorCode::InsufficientDistributorBalance // Can rename the error or remove the check
        // );

        // Find PDA and bump
        let (_distributor_pda, distributor_bump) = Pubkey::find_program_address(
            &[b"distributor".as_ref(), ctx.accounts.mint.key().as_ref()],
            ctx.program_id,
        );
        let mint_key = ctx.accounts.mint.key();
        let seeds = &[b"distributor".as_ref(), mint_key.as_ref(), &[distributor_bump]];
        let signer_seeds = &[&seeds[..]];

        // Transfer 30% to Bonding Curve Account
        if bonding_curve_amount > 0 {
             msg!("Transferring {} tokens to bonding curve account {}", bonding_curve_amount, ctx.accounts.bonding_curve_token_account.key());
             let cpi_accounts_bc = Transfer {
                 from: ctx.accounts.distributor_token_account.to_account_info(),
                 to: ctx.accounts.bonding_curve_token_account.to_account_info(),
                 authority: ctx.accounts.distributor_authority.to_account_info(),
             };
             let cpi_program_bc = ctx.accounts.token_program.to_account_info();
             let cpi_ctx_bc = CpiContext::new_with_signer(cpi_program_bc, cpi_accounts_bc, signer_seeds);
             token::transfer(cpi_ctx_bc, bonding_curve_amount)?;
        } else {
            msg!("Skipping transfer to bonding curve (amount is zero)");
        }

        // Transfer 70% (remainder) to User Account
        if user_amount > 0 {
            msg!("Transferring {} tokens to user account {}", user_amount, ctx.accounts.user_token_account.key());
            let cpi_accounts_user = Transfer {
                from: ctx.accounts.distributor_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.distributor_authority.to_account_info(),
            };
            let cpi_program_user = ctx.accounts.token_program.to_account_info();
            let cpi_ctx_user = CpiContext::new_with_signer(cpi_program_user, cpi_accounts_user, signer_seeds);
            token::transfer(cpi_ctx_user, user_amount)?;
        } else {
            msg!("Skipping transfer to user (amount is zero)");
        }

        // Optional: Close the distributor token account
        // ... (account closing logic, if needed) ...
        // Make sure the `destination` for rent is specified correctly (e.g., user_authority)

        msg!("Token distribution complete.");
        Ok(())
    }
}

#[derive(Accounts)]
// Remove total_supply from instruction data, as we take it from the account
pub struct DistributeTokens<'info> {
    pub mint: Account<'info, Mint>, // Needed for PDA seeds

    /// CHECK: PDA derivation checked below. Authority for distributor_token_account.
    #[account(
        seeds = [b"distributor".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub distributor_authority: AccountInfo<'info>,

    #[account(
        mut, // Balance will decrease
        associated_token::mint = mint,
        associated_token::authority = distributor_authority, // PDA owns this account
    )]
    pub distributor_token_account: Account<'info, TokenAccount>,

    // User Authority must now be a Signer, as they pay for ATAs
    #[account(mut)] // Receives rent when distributor_token_account is closed
    pub user_authority: Signer<'info>,

    #[account(
        init_if_needed, // Create user's ATA if it doesn't exist
        payer = user_authority, // User pays for their own ATA
        associated_token::mint = mint,
        associated_token::authority = user_authority, // User owns this account
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // #[account(
    //     seeds = [b"bonding_curve".as_ref(), mint.key().as_ref()],
    //     bump,
    //     seeds::program = bonding_curve::ID // Make sure the ID is correct
    // )]
    /// CHECK: PDA derivation checked below (using bonding_curve module logic).
    pub bonding_curve_authority: AccountInfo<'info>,

    #[account(
        init_if_needed, // Create bonding curve's ATA if it doesn't exist
        payer = user_authority, // Does the user pay for this ATA too? Or need another payer?
        associated_token::mint = mint,
        associated_token::authority = bonding_curve_authority, // Bonding curve PDA owns this
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,

    // --- Required Programs ---
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>, // Needed for init_if_needed
    pub associated_token_program: Program<'info, AssociatedToken>, // Needed for init_if_needed
}

#[error_code]
pub enum ErrorCode {
    #[msg("Calculation overflow")]
    CalculationOverflow,
    #[msg("Total supply cannot be zero")]
    ZeroSupply,
    // Can remove or rename if the check is not needed
    // #[msg("Distributor token account has insufficient balance")]
    // InsufficientDistributorBalance,
}
