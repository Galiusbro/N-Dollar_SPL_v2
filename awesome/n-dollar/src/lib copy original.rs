use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use mpl_token_metadata::{
    instructions::CreateMetadataAccountV3,
    types::DataV2,
    ID as METADATA_PROGRAM_ID,
};

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
