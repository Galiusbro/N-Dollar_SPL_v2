use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, MintTo},
    associated_token::AssociatedToken,
};
use mpl_token_metadata::{
    instructions::CreateMetadataAccountV3,
    types::DataV2,
    ID as METADATA_PROGRAM_ID,
};

// Импорт интерфейса пула ликвидности
use liquidity_pool;

declare_id!("2vgQn1c2JPWGHYcjhBcdeXKCQCSWfs8gYn6CcNhMKwMG");

// Константы для расчета комиссий и рент-экземпта
const TOKEN_MINT_RENT: u64 = 1_461_600;
const METADATA_RENT: u64 = 5_616_000;
const TOKEN_ACCOUNT_RENT: u64 = 2_039_280;
const ACCOUNT_RENT: u64 = 1_141_440;
const TOTAL_RENT_COST: u64 = TOKEN_MINT_RENT + METADATA_RENT + TOKEN_ACCOUNT_RENT + ACCOUNT_RENT;

// Константы для токенов
const DECIMALS: u8 = 9;
const DECIMALS_FACTOR: u64 = 1_000_000_000; // 10^9
const MAX_TOTAL_SUPPLY: u64 = 1_000_000_000 * DECIMALS_FACTOR; // 1 миллиард токенов

// Константы для метадаты
const MAX_NAME_LENGTH: usize = 32;
const MAX_SYMBOL_LENGTH: usize = 10;
const MAX_URI_LENGTH: usize = 200;

#[event]
pub struct TokenCreated {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub total_supply: u64,
    pub n_dollar_spent: u64,
    pub sol_used: u64,
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
        decimals: u8,
        total_supply: u64,
        n_dollar_amount: u64, // Количество N-долларов для обмена
    ) -> Result<()> {
        require!(decimals == DECIMALS, ErrorCode::InvalidDecimals);
        require!(total_supply > 0, ErrorCode::InvalidSupply);
        require!(total_supply <= MAX_TOTAL_SUPPLY, ErrorCode::SupplyTooLarge);
        require!(name.len() <= MAX_NAME_LENGTH, ErrorCode::NameTooLong);
        require!(symbol.len() <= MAX_SYMBOL_LENGTH, ErrorCode::SymbolTooLong);
        require!(uri.len() <= MAX_URI_LENGTH, ErrorCode::UriTooLong);

        require!(ctx.accounts.n_dollar_mint.key() != ctx.accounts.sol_mint.key(), ErrorCode::SameTokenMints);
        require!(!ctx.accounts.n_dollar_mint.key().eq(&Pubkey::default()), ErrorCode::InvalidMint);
        require!(!ctx.accounts.sol_mint.key().eq(&Pubkey::default()), ErrorCode::InvalidMint);

        require!(
            ctx.accounts.pool_n_dollar_account.key() != ctx.accounts.pool_sol_account.key() &&
            ctx.accounts.pool_n_dollar_account.key() != ctx.accounts.user_n_dollar_account.key() &&
            ctx.accounts.pool_sol_account.key() != ctx.accounts.user_n_dollar_account.key(),
            ErrorCode::DuplicateTokenAccounts
        );

        require!(
            ctx.accounts.user_n_dollar_account.amount >= n_dollar_amount,
            ErrorCode::InsufficientNDollarBalance
        );

        require!(
            n_dollar_amount >= TOTAL_RENT_COST,
            ErrorCode::InsufficientNDollarAmount
        );

        require!(
            ctx.accounts.pool_sol_account.lamports() >= TOTAL_RENT_COST,
            ErrorCode::InsufficientPoolSolBalance
        );

        let (pool_pda, _pool_bump) = Pubkey::find_program_address(
            &[b"pool".as_ref(), ctx.accounts.n_dollar_mint.key().as_ref()],
            ctx.accounts.liquidity_pool_program.key
        );
        require!(pool_pda == ctx.accounts.liquidity_pool.key(), ErrorCode::InvalidPoolAccount);

        // Сначала обмениваем N-доллары на SOL через пул ликвидности
        {
            let cpi_program = ctx.accounts.liquidity_pool_program.to_account_info();
            let cpi_accounts = liquidity_pool::cpi::accounts::Swap {
                pool: ctx.accounts.liquidity_pool.to_account_info(),
                ndollar_mint: ctx.accounts.n_dollar_mint.to_account_info(),
                ndollar_vault: ctx.accounts.pool_n_dollar_account.to_account_info(),
                sol_vault: ctx.accounts.pool_sol_account.to_account_info(),
                user: ctx.accounts.authority.to_account_info(),
                user_ndollar: ctx.accounts.user_n_dollar_account.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };

            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            liquidity_pool::cpi::swap_ndollar_to_sol(cpi_ctx, n_dollar_amount)?;
        }

        // Проверяем, что получили достаточно SOL
        let user_sol_balance = ctx.accounts.authority.to_account_info().lamports();
        require!(
            user_sol_balance >= TOTAL_RENT_COST,
            ErrorCode::InsufficientSolBalance
        );

        // Создание метаданных токена
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

        let token_info = &mut ctx.accounts.token_info;
        token_info.mint = ctx.accounts.mint.key();
        token_info.authority = ctx.accounts.authority.key();
        token_info.total_supply = total_supply;

        // Минтим токены на аккаунт владельца
        mint_tokens(
            &ctx.accounts.mint,
            &ctx.accounts.token_account.to_account_info(),
            &ctx.accounts.authority,
            &ctx.accounts.token_program,
            total_supply,
            decimals,
        )?;

        // Возврат избыточного SOL обратно в пул
        let current_balance = ctx.accounts.authority.to_account_info().lamports();
        let mut sol_used = current_balance;

        if current_balance > TOTAL_RENT_COST {
            let excess_sol = current_balance - TOTAL_RENT_COST;

            let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.authority.key(),
                &ctx.accounts.pool_sol_account.key(),
                excess_sol,
            );

            anchor_lang::solana_program::program::invoke(
                &transfer_ix,
                &[
                    ctx.accounts.authority.to_account_info(),
                    ctx.accounts.pool_sol_account.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            sol_used = TOTAL_RENT_COST;
        }

        emit!(TokenCreated {
            mint: ctx.accounts.mint.key(),
            authority: ctx.accounts.authority.key(),
            total_supply,
            n_dollar_spent: n_dollar_amount,
            sol_used,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!("Токен успешно создан");
        msg!("Общее количество: {}", total_supply);
        msg!("Потрачено N-долларов: {}", n_dollar_amount);
        msg!("Использовано SOL: {}", sol_used);

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid metadata account")]
    InvalidMetadataAccount,
    #[msg("Invalid pool account")]
    InvalidPoolAccount,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Insufficient N-Dollar balance")]
    InsufficientNDollarBalance,
    #[msg("Insufficient N-Dollar amount for rent")]
    InsufficientNDollarAmount,
    #[msg("Insufficient SOL balance after swap")]
    InsufficientSolBalance,
    #[msg("Insufficient SOL balance in pool")]
    InsufficientPoolSolBalance,
    #[msg("Invalid decimals")]
    InvalidDecimals,
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
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Invalid mint address")]
    InvalidMint,
    #[msg("N-Dollar and SOL mints cannot be the same")]
    SameTokenMints,
    #[msg("Token accounts must be unique")]
    DuplicateTokenAccounts,
}

// Вспомогательная функция для минта токенов
fn mint_tokens<'info>(
    mint: &Account<'info, Mint>,
    destination: &AccountInfo<'info>,
    authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
    _decimals: u8,
) -> Result<()> {
    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        to: destination.clone(),
        authority: authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
    anchor_spl::token::mint_to(cpi_ctx, amount)?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    uri: String,
    decimals: u8,
    total_supply: u64,
    n_dollar_amount: u64
)]
pub struct CreateUserToken<'info> {
    #[account(
        init, 
        payer = authority, 
        mint::decimals = DECIMALS, 
        mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Метадата проверяется через PDA
    #[account(
        mut,
        seeds = [
            b"metadata",
            METADATA_PROGRAM_ID.as_ref(),
            mint.key().as_ref()
        ],
        bump,
        seeds::program = METADATA_PROGRAM_ID
    )]
    pub metadata: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + TokenInfo::SIZE,
        seeds = [b"token_info", mint.key().as_ref()],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,

    // Аккаунты для взаимодействия с пулом ликвидности
    #[account(
        mut,
        seeds = [b"pool".as_ref(), n_dollar_mint.key().as_ref()],
        bump,
        seeds::program = liquidity_pool_program.key()
    )]
    pub liquidity_pool: Account<'info, liquidity_pool::Pool>,

    #[account(
        mut,
        constraint = pool_n_dollar_account.mint == n_dollar_mint.key(),
        constraint = pool_n_dollar_account.owner == liquidity_pool.key(),
    )]
    pub pool_n_dollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a native SOL vault
    #[account(
        mut,
        seeds = [b"sol_vault".as_ref(), liquidity_pool.key().as_ref()],
        bump,
        seeds::program = liquidity_pool_program.key()
    )]
    pub pool_sol_account: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = user_n_dollar_account.mint == n_dollar_mint.key(),
        constraint = user_n_dollar_account.owner == authority.key(),
    )]
    pub user_n_dollar_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a native SOL account
    // #[account(mut)]
    // pub user_sol_account: AccountInfo<'info>,
    
    pub n_dollar_mint: Account<'info, Mint>,
    pub sol_mint: Account<'info, Mint>,

    pub liquidity_pool_program: Program<'info, liquidity_pool::program::LiquidityPool>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Проверяется через адрес
    #[account(address = METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,
}

#[account]
pub struct TokenInfo {
    pub mint: Pubkey,         // Адрес минта токена
    pub authority: Pubkey,    // Владелец токена
    pub total_supply: u64,    // Общее количество токенов
}

impl TokenInfo {
    pub const SIZE: usize = 32 + 32 + 8; // Pubkey + Pubkey + u64
}