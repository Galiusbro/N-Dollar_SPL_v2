// token_creator.rs
use anchor_lang::prelude::*;
use anchor_spl::{
associated_token::AssociatedToken,
token::{self, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
types::{DataV2, Creator},
ID as METADATA_PROGRAM_ID,
};

// Use the original declare_id
declare_id!("2vgQn1c2JPWGHYcjhBcdeXKCQCSWfs8gYn6CcNhMKwMG");

// --- Constants ---
// Rent costs need to be updated if we initialize more accounts here
// We now potentially initialize 3 ATAs: user, distributor, bonding_curve
const TOKEN_ACCOUNT_RENT: u64 = 2_039_280;
const MINT_RENT: u64 = 1_461_600; // Check current rent for Mint account
const METADATA_RENT: u64 = 5_616_000; // Check current rent for Metadata account
const TOKEN_INFO_RENT: u64 = 1_141_440; // Rent for our custom TokenInfo account (adjust based on actual size)

// Rent for accounts created by this instruction: Mint, Metadata, TokenInfo, Distributor ATA
const TOTAL_RENT_COST: u64 = MINT_RENT + METADATA_RENT + TOKEN_INFO_RENT + TOKEN_ACCOUNT_RENT;

const DECIMALS: u8 = 9;
const DECIMALS_FACTOR: u64 = 1_000_000_000; // 10^9
const MAX_TOTAL_SUPPLY: u64 = 100_000_000 * DECIMALS_FACTOR; // 1 billion tokens

const MAX_NAME_LENGTH: usize = 32;
const MAX_SYMBOL_LENGTH: usize = 10;
const MAX_URI_LENGTH: usize = 200;

#[event]
pub struct TokenCreated {
pub mint: Pubkey,
pub authority: Pubkey,
pub total_supply: u64,
pub distributor_authority: Pubkey,
pub distributor_token_account: Pubkey,
pub sol_used_for_rent: u64,
pub timestamp: i64,
}

#[program]
pub mod token_creator {
use super::*;
use anchor_lang::solana_program::program::invoke;

    pub fn create_user_token(
        ctx: Context<CreateUserToken>,
        name: String,
        symbol: String,
        uri: String,
        total_supply: u64,
    ) -> Result<()> {
        msg!("Starting token creation process...");
        msg!("Name: {}, Symbol: {}, URI: {}", name, symbol, uri);
        msg!("Total Supply: {}", total_supply);

        // --- Basic Input Validations ---
        require!(total_supply > 0, ErrorCode::InvalidSupply);
        require!(total_supply <= MAX_TOTAL_SUPPLY, ErrorCode::SupplyTooLarge);
        require!(name.len() <= MAX_NAME_LENGTH, ErrorCode::NameTooLong);
        require!(symbol.len() <= MAX_SYMBOL_LENGTH, ErrorCode::SymbolTooLong);
        require!(uri.len() <= MAX_URI_LENGTH, ErrorCode::UriTooLong);

        msg!("Basic validations passed");

        // Check if the user (payer) has enough SOL for rent
        let required_lamports = TOTAL_RENT_COST; // Use the calculated constant
        require!(
            ctx.accounts.authority.lamports() >= required_lamports,
            ErrorCode::InsufficientSolForRent
        );
        msg!("User SOL balance sufficient for rent ({} required)", required_lamports);

        // --- Create Metadata ---
        msg!("Creating token metadata...");
        let metadata_accounts_infos = [
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let creators = vec![
            Creator {
                address: ctx.accounts.authority.key(),
                verified: true,
                share: 100,
            }
        ];

        let data = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creators),
            collection: None,
            uses: None,
        };

        let ix = CreateMetadataAccountV3 {
            metadata: ctx.accounts.metadata.key(),
            mint: ctx.accounts.mint.key(),
            mint_authority: ctx.accounts.authority.key(),
            payer: ctx.accounts.authority.key(),
            update_authority: (ctx.accounts.authority.key(), true),
            system_program: ctx.accounts.system_program.key(),
            rent: Some(ctx.accounts.rent.key()),
        }.instruction(CreateMetadataAccountV3InstructionArgs {
            data,
            is_mutable: true,
            collection_details: None,
        });

        invoke(&ix, &metadata_accounts_infos)?;
        msg!("Metadata created successfully");

        // --- Initialize Token Info Account ---
        let token_info = &mut ctx.accounts.token_info;
        token_info.mint = ctx.accounts.mint.key();
        token_info.authority = ctx.accounts.authority.key();
        token_info.total_supply = total_supply;
        msg!("Token info account initialized");

        // Rent is paid automatically by the `init` macro for mint, metadata, token_info, and distributor_token_account
        let sol_used_for_rent = TOTAL_RENT_COST; // Use the constant value

        // --- Mint Tokens to Distributor's ATA ---
        msg!("Minting {} tokens to distributor ATA", total_supply);
        let cpi_accounts_mint = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.distributor_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program_mint = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_mint = CpiContext::new(cpi_program_mint, cpi_accounts_mint);
        token::mint_to(cpi_ctx_mint, total_supply)?;
        msg!("Minting complete");

        emit!(TokenCreated {
            mint: ctx.accounts.mint.key(),
            authority: ctx.accounts.authority.key(),
            total_supply,
            distributor_authority: ctx.accounts.distributor_authority.key(),
            distributor_token_account: ctx.accounts.distributor_token_account.key(),
            sol_used_for_rent,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!("Token creation process completed successfully using SOL for rent.");
        msg!("Total Supply: {}", total_supply);
        msg!("SOL Used for Rent: {}", sol_used_for_rent);

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid metadata account")]
    InvalidMetadataAccount,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Insufficient SOL balance in authority account for rent")]
    InsufficientSolForRent,
    #[msg("Invalid supply")]
    InvalidSupply,
    #[msg("Supply too large")]
    SupplyTooLarge,
    #[msg("Name too long")]
    NameTooLong,
    #[msg("Symbol too long")]
    SymbolTooLong,
    #[msg("URI too long")]
    UriTooLong,
    #[msg("Invalid mint address")]
    InvalidMint,
}


#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    uri: String,
    total_supply: u64,
)]
pub struct CreateUserToken<'info> {
// ---- Token Creation Accounts ----
#[account(
    init,
    payer = authority,
    mint::decimals = DECIMALS,
    mint::authority = authority.key(),
    mint::freeze_authority = authority.key(),
    // rent_exempt = enforce
)]
pub mint: Account<'info, Mint>,

    /// CHECK: Metadata account derived and checked by metaplex program
    #[account(
        mut,
        seeds = [
            b"metadata",
            METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref()
        ],
        bump,
        seeds::program = METADATA_PROGRAM_ID,
    )]
    pub metadata: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<TokenInfo>(),
        seeds = [b"token_info", mint.key().as_ref()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // ---- Liquidity Pool Accounts ----
    // #[account(
    //     mut,
    //     // Add constraints if the pool state struct is known, e.g., pool.is_initialized
    //     seeds = [b"pool".as_ref(), n_dollar_mint.key().as_ref()],
    //     bump,
    //     seeds::program = liquidity_pool_program.key() // Program ID of the pool
    // )]
    // pub liquidity_pool: Account<'info, liquidity_pool::Pool>, // Use actual Pool type

    // #[account(mut, constraint = pool_n_dollar_account.mint == n_dollar_mint.key())]
    // // Add constraint for owner if known, e.g. pool_n_dollar_account.owner == liquidity_pool.key()
    // pub pool_n_dollar_account: Box<Account<'info, TokenAccount>>, // Use Box for potentially large account
    // /// CHECK: Native SOL vault for the pool. Check seeds/address if possible.
    // #[account(mut)]
    // // If pool_sol_account is a PDA:
    // // seeds = [b"sol_vault".as_ref(), liquidity_pool.key().as_ref()], bump, seeds::program = liquidity_pool_program.key()
    // pub pool_sol_account: AccountInfo<'info>,

    // #[account(
    //     mut,
    //     constraint = user_n_dollar_account.mint == n_dollar_mint.key(),
    //     constraint = user_n_dollar_account.owner == authority.key(),
    // )]
    // pub user_n_dollar_account: Account<'info, TokenAccount>,

    // pub n_dollar_mint: Account<'info, Mint>,
    // pub sol_mint: Account<'info, Mint>, // Removed

    // ---- Distribution Accounts ----
    /// CHECK: PDA for the distributor program, acts as authority for its ATA.
    #[account(
        seeds = [b"distributor".as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = token_distributor::ID
    )]
    pub distributor_authority: AccountInfo<'info>,

    #[account(
        init_if_needed, // Initialize distributor's ATA if it doesn't exist
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = distributor_authority, // PDA is the authority
    )]
    pub distributor_token_account: Account<'info, TokenAccount>,

    // #[account(
    //     init_if_needed, // Initialize user's ATA for the *new* token
    //     payer = authority,
    //     associated_token::mint = mint,
    //     associated_token::authority = authority, // User is the authority
    // )]
    // pub user_token_account: Account<'info, TokenAccount>, // User's ATA for the *newly created* token

    // /// CHECK: PDA for the bonding curve program, acts as authority for its ATA.
    //  #[account(
    //     seeds = [b"bonding_curve".as_ref(), mint.key().as_ref()],
    //     bump,
    //     seeds::program = bonding_curve::ID
    // )]
    // pub bonding_curve_authority: AccountInfo<'info>,

    // #[account(
    //     init_if_needed, // Initialize bonding curve's ATA if it doesn't exist
    //     payer = authority,
    //     associated_token::mint = mint,
    //     associated_token::authority = bonding_curve_authority, // Bonding curve PDA is the authority
    // )]
    // pub bonding_curve_token_account: Account<'info, TokenAccount>,

    // ---- Programs ----
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Address verified by constraint
    #[account(address = METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,

    // Program for the liquidity pool CPI - REMOVED
    // pub liquidity_pool_program: Program<'info, liquidity_pool::program::LiquidityPool>, // Use actual Program type
    // Program for the token distributor CPI
    // pub token_distributor_program: Program<'info, token_distributor::program::TokenDistributor>, // Use actual Program type

    // Program for the bonding curve (needed for PDA derivation) - not called via CPI here
    // pub bonding_curve_program: Program<'info, bonding_curve::program::BondingCurve>, // Use actual Program type
}

#[account]
#[derive(InitSpace)] // Use InitSpace for automatic space calculation
pub struct TokenInfo {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub total_supply: u64,
}

// Remove impl TokenInfo { SIZE } as InitSpace handles it.