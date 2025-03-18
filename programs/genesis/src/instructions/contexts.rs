use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::metadata::Metadata;
use crate::state::*;

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String, ndollar_payment: u64, admin: Option<Pubkey>)]
pub struct CreateCoin<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = coin_data,
    )]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: Аккаунт метаданных, инициализируется через CPI
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    
    /// CHECK: Authority для метаданных
    #[account(mut)]
    pub mint_authority: AccountInfo<'info>,
    
    #[account(
        init,
        payer = creator,
        seeds = [b"coin_data".as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + CoinData::SPACE
    )]
    pub coin_data: Account<'info, CoinData>,
    
    #[account(
        mut,
        constraint = ndollar_token_account.owner == creator.key()
    )]
    pub ndollar_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub fees_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    /// Mint токена N-Dollar
    pub ndollar_mint: Account<'info, Mint>,
    
    /// CHECK: Это PDA аккаунт bonding_curve, который будет инициализирован через CPI
    #[account(mut)]
    pub bonding_curve: AccountInfo<'info>,
    
    /// CHECK: Аккаунт ликвидности для бондинговой кривой
    #[account(mut)]
    pub liquidity_pool: AccountInfo<'info>,
    
    /// CHECK: Программа Bonding Curve
    pub bonding_curve_program: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: Это токен ассоциированная программа
    pub associated_token_program: AccountInfo<'info>,
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PurchaseFounderOption<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: Адрес создателя монеты
    pub creator: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = mint.key() == coin_data.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"coin_data".as_ref(), mint.key().as_ref()],
        bump = coin_data.bump
    )]
    pub coin_data: Account<'info, CoinData>,
    
    #[account(
        mut,
        constraint = ndollar_token_account.owner == admin.key()
    )]
    pub ndollar_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub fees_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = creator_token_account.mint == mint.key(),
        constraint = creator_token_account.owner == creator.key()
    )]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GenerateReferralLink<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        constraint = mint.key() == coin_data.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"coin_data".as_ref(), mint.key().as_ref()],
        bump = coin_data.bump
    )]
    pub coin_data: Account<'info, CoinData>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"referral_data".as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + ReferralData::SPACE
    )]
    pub referral_data: Account<'info, ReferralData>,
    
    /// CHECK: Это PDA аккаунт referral_system, который будет инициализирован через CPI
    #[account(mut)]
    pub referral_system: AccountInfo<'info>,
    
    /// CHECK: Программа Referral System
    pub referral_system_program: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TransferAdminRights<'info> {
    #[account(mut)]
    pub current_admin: Signer<'info>,
    
    #[account(
        constraint = mint.key() == coin_data.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"coin_data".as_ref(), mint.key().as_ref()],
        bump = coin_data.bump
    )]
    pub coin_data: Account<'info, CoinData>,
    
    pub system_program: Program<'info, System>,
}