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

declare_id!("E3ZAgCnCpX38ktRUFgpjsMp3xJT8qzuXerWy88zY7wfL");

#[program]
pub mod n_dollar {
    use super::*;
    use anchor_lang::solana_program::program::invoke;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        decimals: u8,
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

        // Инициализируем состояние расписания эмиссии
        let schedule = &mut ctx.accounts.mint_schedule;
        schedule.mint = ctx.accounts.mint.key();
        schedule.current_week = 1;

        // CPI-вызов liquidity_pool
        let cpi_program = ctx.accounts.liquidity_pool_program.to_account_info();
        let cpi_accounts = liquidity_pool::cpi::accounts::InitializeNDollarAccount {
            pool_account: ctx.accounts.pool_account.to_account_info(),
            pool_authority: ctx.accounts.pool_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            initializer: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        liquidity_pool::cpi::initialize_n_dollar_account(cpi_ctx)?;

        msg!("Liquidity pool initialized via CPI!");

        // Минтим токены 1-й недели сразу при создании
        mint_to_pool(
            &ctx.accounts.mint,
            &ctx.accounts.pool_account.to_account_info(),
            &ctx.accounts.authority,
            &ctx.accounts.token_program,
            108_000_000,
            decimals,
        )?;

        msg!("Token created and week 1 tokens minted.");

        // Инициализация SOL аккаунта через CPI
        let cpi_program = ctx.accounts.liquidity_pool_program.to_account_info();
        let cpi_accounts = liquidity_pool::cpi::accounts::InitializeSolAccount {
            sol_account: ctx.accounts.sol_account.to_account_info(),
            pool_authority: ctx.accounts.pool_authority.to_account_info(),
            initializer: ctx.accounts.authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        liquidity_pool::cpi::initialize_sol_account(cpi_ctx)?;

        // Перевод SOL используя системную программу
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.sol_account.key(),
            500_000_000, // 0.5 SOL
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.sol_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        msg!("Transferred 0.5 SOL to sol_account");

        Ok(())
    }

    pub fn mint_scheduled_tokens(ctx: Context<MintScheduledTokens>, decimals: u8) -> Result<()> {
        let schedule = &mut ctx.accounts.mint_schedule;

        // Эмиссия по неделям
        let amount = match schedule.current_week {
            1 => 54_000_000_000,
            2 => 108_000_000_000,
            3 => 369_000_000_000,
            _ => return Err(error!(ErrorCode::ScheduleComplete)),
        };

        mint_to_pool(
            &ctx.accounts.mint,
            &ctx.accounts.pool_account.to_account_info(),
            &ctx.accounts.authority,
            &ctx.accounts.token_program,
            amount,
            decimals,
        )?;

        schedule.current_week += 1;
        msg!("Week {} minted: {}", schedule.current_week, amount);

        Ok(())
    }
}

fn mint_to_pool<'info>(
    mint: &Account<'info, Mint>,
    destination: &AccountInfo<'info>,
    authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        to: destination.clone(),
        authority: authority.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
    anchor_spl::token::mint_to(cpi_ctx, amount * 10u64.pow(decimals as u32))?;
    Ok(())
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(
        init, payer = authority, mint::decimals = 9, mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: metadata account
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + MintSchedule::SIZE,
        seeds = [b"mint_schedule", mint.key().as_ref()],
        bump
    )]
    pub mint_schedule: Account<'info, MintSchedule>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: PDA pool_account for liquidity pool
    #[account(mut)]
    pub pool_account: AccountInfo<'info>,

    /// CHECK: PDA sol_account для liquidity pool
    #[account(
        mut,
        seeds = [b"sol_account", mint.key().as_ref()],
        bump,
        seeds::program = liquidity_pool_program.key()
    )]
    pub sol_account: AccountInfo<'info>,

    /// CHECK: PDA pool_authority for liquidity pool
    #[account(seeds = [b"pool_authority"], bump, seeds::program = liquidity_pool_program.key())]
    pub pool_authority: AccountInfo<'info>,

    pub liquidity_pool_program: Program<'info, liquidity_pool::program::LiquidityPool>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Metaplex Token Metadata Program
    #[account(address = METADATA_PROGRAM_ID)]
    pub token_metadata_program: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct MintScheduledTokens<'info> {
    #[account(mut, has_one = mint)]
    pub mint_schedule: Account<'info, MintSchedule>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub pool_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct MintSchedule {
    pub mint: Pubkey,
    pub current_week: u8,
}

impl MintSchedule {
    const SIZE: usize = 32 + 1; // Pubkey + u8
}

#[error_code]
pub enum ErrorCode {
    #[msg("All scheduled mints completed.")]
    ScheduleComplete,
}
