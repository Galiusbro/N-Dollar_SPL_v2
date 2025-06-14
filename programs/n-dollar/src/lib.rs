use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use mpl_token_metadata::{
    instructions::CreateMetadataAccountV3,
    types::DataV2,
    ID as METADATA_PROGRAM_ID,
};

use liquidity_pool::cpi::accounts::InitializePool;

declare_id!("3Mdve11qmHuVZVe9YgCzA1d3hcjyamm2Jiz3VfHSJgEQ");

#[program]
pub mod n_dollar {
    use super::*;
    use anchor_lang::solana_program::program::invoke;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let metadata_accounts = CreateMetadataAccountV3 {
            metadata: ctx.accounts.metadata.key(),
            mint: ctx.accounts.mint.key(),
            mint_authority: ctx.accounts.authority.key(),
            payer: ctx.accounts.authority.key(),
            update_authority: (ctx.accounts.authority.key(), true),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        };

        let data = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        invoke(
            &metadata_accounts.instruction(
                mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
                    data,
                    is_mutable: true,
                    collection_details: None,
                },
            ),
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
            ],
        )?;

        msg!("Token created successfully");
        Ok(())
    }

    pub fn initialize_liquidity_pool(
        ctx: Context<InitializeLiquidityPool>,
    ) -> Result<()> {
        // Calculate the PDA bump for the pool account
        let (_, bump) = Pubkey::find_program_address(
            &[b"pool".as_ref(), ctx.accounts.mint.key().as_ref()],
            &ctx.accounts.liquidity_pool_program.key(),
        );

        // Initialize the liquidity pool via CPI
        let cpi_accounts = InitializePool {
            pool: ctx.accounts.pool.to_account_info(),
            ndollar_mint: ctx.accounts.mint.to_account_info(),
            ndollar_vault: ctx.accounts.ndollar_vault.to_account_info(),
            sol_vault: ctx.accounts.sol_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_program = ctx.accounts.liquidity_pool_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        liquidity_pool::cpi::initialize_pool(cpi_ctx, bump)?;

        // Mint 108,000,000 tokens to the liquidity pool's vault
        let mint_amount = 108_000_000 * 10u64.pow(9); // Adjust for 9 decimals
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.ndollar_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        anchor_spl::token::mint_to(cpi_ctx, mint_amount)?;
        
        msg!("Liquidity pool initialized with 108,000,000 N-Dollar tokens");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(
        init, 
        payer = authority, 
        mint::decimals = 9, 
        mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: metadata account
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Metaplex Token Metadata Program
    #[account(address = METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: This is the pool account that will be initialized
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    
    /// CHECK: This is the N-Dollar vault of the pool
    #[account(mut)]
    pub ndollar_vault: AccountInfo<'info>,
    
    /// CHECK: This is the SOL vault of the pool
    #[account(mut)]
    pub sol_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    
    /// CHECK: This is the liquidity pool program
    pub liquidity_pool_program: AccountInfo<'info>,
}

