// token_distributor.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

// !!! REPLACE WITH YOUR ACTUAL TOKEN DISTRIBUTOR PROGRAM ID !!!
declare_id!("2Hy1wGdC5iqceaTnZC1qJeuoM4s6yEKHbYcjMMpbKYqp");

#[program]
pub mod token_distributor {
    use super::*;

    pub fn distribute_tokens(ctx: Context<DistributeTokens>, total_supply: u64) -> Result<()> {
        msg!("Distributing tokens...");
        require!(total_supply > 0, ErrorCode::ZeroSupply);

        // Calculate distribution amounts
        // Use u128 for intermediate calcs to prevent overflow
        let total_supply_u128 = total_supply as u128;
        let bonding_curve_amount_u128 = total_supply_u128
            .checked_mul(30)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::CalculationOverflow)?;

        let bonding_curve_amount = bonding_curve_amount_u128 as u64;

        // Assign the remainder to the user to avoid dust due to integer division
        let user_amount = total_supply
            .checked_sub(bonding_curve_amount)
            .ok_or(ErrorCode::CalculationOverflow)?;

        msg!(
            "Total Supply: {}, Bonding Curve: {}, User: {}",
            total_supply,
            bonding_curve_amount,
            user_amount
        );

        // Verify the distributor token account has enough balance
        // Reload account data to ensure we have the latest balance after minting
        ctx.accounts.distributor_token_account.reload()?;
        require!(
            ctx.accounts.distributor_token_account.amount == total_supply,
            ErrorCode::InsufficientDistributorBalance
        );

        // Find PDA bump
        let (_distributor_pda, distributor_bump) = Pubkey::find_program_address(
            &[b"distributor".as_ref(), ctx.accounts.mint.key().as_ref()],
            ctx.program_id,
        );

        // Create signer seeds for the PDA
        let mint_key = ctx.accounts.mint.key();
        let seeds = &[b"distributor".as_ref(), mint_key.as_ref(), &[distributor_bump]];
        let signer_seeds = &[&seeds[..]];

        // Transfer 30% to Bonding Curve Account
        if bonding_curve_amount > 0 {
            msg!(
                "Transferring {} tokens to bonding curve account {}",
                bonding_curve_amount,
                ctx.accounts.bonding_curve_token_account.key()
            );
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
             msg!(
                "Transferring {} tokens to user account {}",
                user_amount,
                ctx.accounts.user_token_account.key()
            );
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

        // Optional: Close the distributor token account if it's now empty and rent should be reclaimed.
        // This requires the `distributor_authority` PDA to also be the close authority.
        // Requires careful consideration of rent destination (e.g., back to original user).
        // Example (add `CloseAccount` to imports and `close_account` to `token::transfer` context if needed):
        // let ca_accounts = CloseAccount {
        //     account: ctx.accounts.distributor_token_account.to_account_info(),
        //     destination: ctx.accounts.user_authority.to_account_info(), // Send rent to user
        //     authority: ctx.accounts.distributor_authority.to_account_info(),
        // };
        // let ca_ctx = CpiContext::new_with_signer(
        //     ctx.accounts.token_program.to_account_info(),
        //     ca_accounts,
        //     signer_seeds
        // );
        // token::close_account(ca_ctx)?;
        // msg!("Distributor token account closed.");


        msg!("Token distribution complete.");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(total_supply: u64)]
pub struct DistributeTokens<'info> {
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA derivation checked below. Acts as authority.
    #[account(
        seeds = [b"distributor".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub distributor_authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = distributor_authority, // PDA owns this account
    )]
    pub distributor_token_account: Account<'info, TokenAccount>,

    /// CHECK: Original creator, receives 70%
    #[account(mut)] // Receives rent if distributor ATA is closed
    pub user_authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user_authority, // User owns this account
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA derivation checked below (using bonding_curve module logic).
    #[account(
        seeds = [b"bonding_curve".as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = bonding_curve::ID
    )]
    pub bonding_curve_authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve_authority, // Bonding curve PDA owns this
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>, // Needed if creating accounts implicitly? No, should be created before.
    pub associated_token_program: Program<'info, AssociatedToken>, // Needed if creating accounts implicitly? No.
}

#[error_code]
pub enum ErrorCode {
    #[msg("Calculation overflow")]
    CalculationOverflow,
    #[msg("Total supply cannot be zero")]
    ZeroSupply,
    #[msg("Distributor token account has insufficient balance")]
    InsufficientDistributorBalance,
}
