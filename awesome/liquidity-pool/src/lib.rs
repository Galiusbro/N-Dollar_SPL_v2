use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer},
    associated_token::AssociatedToken,
};

declare_id!("B24yupzEDjF7Z9frnDG16uAwH1ZYfB57kuzh8jwDsL83");

#[program]
pub mod liquidity_pool {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        bump: u8,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.ndollar_mint = ctx.accounts.ndollar_mint.key();
        pool.ndollar_vault = ctx.accounts.ndollar_vault.key();
        pool.sol_vault = ctx.accounts.sol_vault.key();
        pool.bump = bump;

        msg!("Liquidity pool initialized successfully");
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        ndollar_amount: u64,
        sol_amount: u64,
    ) -> Result<()> {
        // Transfer N-dollar tokens from provider to pool
        let ndollar_transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ndollar.to_account_info(),
                to: ctx.accounts.ndollar_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        anchor_spl::token::transfer(ndollar_transfer_ctx, ndollar_amount)?;

        // Transfer SOL from provider to pool
        let sol_transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.sol_vault.key(),
            sol_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &sol_transfer_ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.sol_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        msg!("Liquidity added: {} N-Dollar, {} SOL", ndollar_amount, sol_amount);
        Ok(())
    }

    pub fn swap_sol_to_ndollar(
        ctx: Context<Swap>,
        sol_amount: u64,
    ) -> Result<()> {
        // Calculate the N-Dollar amount based on the pool's liquidity ratio
        let ndollar_vault_balance = ctx.accounts.ndollar_vault.amount;
        let sol_vault_balance = ctx.accounts.sol_vault.lamports();
        
        // Simple formula: amount_out = (amount_in * balance_out) / balance_in
        // In real DEX, you would apply a price impact, fees, etc.
        let ndollar_amount = (sol_amount as u128)
            .checked_mul(ndollar_vault_balance as u128)
            .unwrap()
            .checked_div(sol_vault_balance as u128)
            .unwrap() as u64;

        // Transfer SOL to the pool
        let sol_transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.sol_vault.key(),
            sol_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &sol_transfer_ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.sol_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Transfer N-Dollar tokens from pool to user
        let mint_key = ctx.accounts.ndollar_mint.key();
        let pool_seeds = &[
            b"pool".as_ref(),
            mint_key.as_ref(),
            &[ctx.accounts.pool.bump],
        ];
        let pool_signer = &[&pool_seeds[..]];

        let ndollar_transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.ndollar_vault.to_account_info(),
                to: ctx.accounts.user_ndollar.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            pool_signer,
        );
        anchor_spl::token::transfer(ndollar_transfer_ctx, ndollar_amount)?;

        msg!("Swapped {} SOL for {} N-Dollar", sol_amount, ndollar_amount);
        Ok(())
    }

    pub fn swap_ndollar_to_sol(
        ctx: Context<Swap>,
        ndollar_amount: u64,
    ) -> Result<()> {
        // Calculate the SOL amount based on the pool's liquidity ratio
        let ndollar_vault_balance = ctx.accounts.ndollar_vault.amount;
        let sol_vault_balance = ctx.accounts.sol_vault.lamports();
        
        // Simple formula: amount_out = (amount_in * balance_out) / balance_in
        let sol_amount = (ndollar_amount as u128)
            .checked_mul(sol_vault_balance as u128)
            .unwrap()
            .checked_div(ndollar_vault_balance as u128)
            .unwrap() as u64;

        // Transfer N-Dollar tokens to the pool
        let ndollar_transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ndollar.to_account_info(),
                to: ctx.accounts.ndollar_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        anchor_spl::token::transfer(ndollar_transfer_ctx, ndollar_amount)?;

        // Find the SOL vault PDA and bump
        let pool_key = ctx.accounts.pool.key();
        let (_, sol_vault_bump) = Pubkey::find_program_address(
            &[b"sol_vault".as_ref(), pool_key.as_ref()],
            ctx.program_id,
        );

        let sol_vault_seeds = &[
            b"sol_vault".as_ref(),
            pool_key.as_ref(),
            &[sol_vault_bump],
        ];
        let sol_vault_signer = &[&sol_vault_seeds[..]];

        // Transfer SOL using system program
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.sol_vault.key(),
            &ctx.accounts.user.key(),
            sol_amount,
        );

        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.sol_vault.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            sol_vault_signer,
        )?;

        msg!("Swapped {} N-Dollar for {} SOL", ndollar_amount, sol_amount);
        Ok(())
    }
}

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub ndollar_mint: Pubkey,
    pub ndollar_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 1,
        seeds = [b"pool".as_ref(), ndollar_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        associated_token::mint = ndollar_mint,
        associated_token::authority = pool,
    )]
    pub ndollar_vault: Account<'info, TokenAccount>,

    /// CHECK: This is a native SOL vault
    #[account(
        seeds = [b"sol_vault".as_ref(), pool.key().as_ref()],
        bump,
    )]
    pub sol_vault: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    
    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = ndollar_vault.mint == ndollar_mint.key(),
        constraint = ndollar_vault.owner == pool.key(),
    )]
    pub ndollar_vault: Account<'info, TokenAccount>,
    
    /// CHECK: This is a native SOL vault
    #[account(
        mut,
        seeds = [b"sol_vault".as_ref(), pool.key().as_ref()],
        bump,
    )]
    pub sol_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_ndollar.mint == ndollar_mint.key(),
        constraint = user_ndollar.owner == user.key(),
    )]
    pub user_ndollar: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    
    pub ndollar_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = ndollar_vault.mint == ndollar_mint.key(),
        constraint = ndollar_vault.owner == pool.key(),
    )]
    pub ndollar_vault: Account<'info, TokenAccount>,
    
    /// CHECK: This is a native SOL vault
    #[account(
        mut,
        seeds = [b"sol_vault".as_ref(), pool.key().as_ref()],
        bump,
    )]
    pub sol_vault: AccountInfo<'info>,
    
    /// CHECK: This is a native SOL vault
    #[account(mut)]
    // pub user: AccountInfo<'info>,
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_ndollar.mint == ndollar_mint.key(),
        constraint = user_ndollar.owner == user.key(),
    )]
    pub user_ndollar: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
