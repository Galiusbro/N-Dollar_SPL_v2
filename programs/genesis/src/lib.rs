use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata};
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use anchor_lang::solana_program::pubkey::Pubkey;

declare_id!("9FtGUvKzUZ9MFKYWnj4hsNLomFVopjGtznp52pMEWY8D");

#[program]
pub mod genesis {
    use super::*;

    /// Создание нового мемкоина
    pub fn create_coin(
        ctx: Context<CreateCoin>,
        name: String,
        symbol: String,
        uri: String,
        ndollar_payment: u64,
    ) -> Result<()> {
        // Сначала берем плату в N-Dollar токенах
        let creator = &ctx.accounts.creator;
        let ndollar_token_account = &ctx.accounts.ndollar_token_account;
        let fees_account = &ctx.accounts.fees_account;
        
        // Переводим N-Dollar в качестве оплаты за создание монеты
        let transfer_instruction = anchor_spl::token::Transfer {
            from: ndollar_token_account.to_account_info(),
            to: fees_account.to_account_info(),
            authority: creator.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, ndollar_payment)?;
        
        // Создаем минт нового мемкоина
        let coin_data = &mut ctx.accounts.coin_data;
        coin_data.creator = creator.key();
        coin_data.mint = ctx.accounts.mint.key();
        coin_data.name = name.clone();
        coin_data.symbol = symbol.clone();
        coin_data.creation_time = Clock::get()?.unix_timestamp;
        coin_data.total_supply = 0;
        coin_data.referral_link_active = false;
        coin_data.bump = ctx.bumps.coin_data;
        
        // Установка метаданных через MetaplexMetadata
        let metadata_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.mint_authority.to_account_info(),
            payer: creator.to_account_info(),
            update_authority: ctx.accounts.mint_authority.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let token_data = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(
            CpiContext::new(ctx.accounts.metadata_program.to_account_info(), metadata_accounts),
            token_data,
            true,  // is_mutable
            true,  // update_authority_is_signer
            None,  // collection_details
        )?;
        
        // Сразу минтим начальное предложение токенов (10% от максимального предложения) создателю
        let decimals = 9; // Используем стандартные 9 десятичных знаков
        let max_supply: u64 = 1_000_000_000 * 10u64.pow(decimals as u32); // Например, 1 миллиард токенов
        let initial_supply = max_supply / 10; // 10% от максимального предложения
        
        let seeds = &[
            b"coin_data".as_ref(),
            &ctx.accounts.mint.key().to_bytes(),
            &[coin_data.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.creator_token_account.to_account_info(),
            authority: coin_data.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, initial_supply)?;
        
        // Обновляем информацию о токенах
        coin_data.total_supply = initial_supply;
        
        // CPI вызов для инициализации бондинговой кривой (тут будет вызов контракта bonding-curve)
        // Это будет добавлено позже после реализации bonding-curve модуля
        
        msg!("Мемкоин успешно создан: {}", coin_data.symbol);
        Ok(())
    }

    /// Опция для основателя купить дополнительные токены (до 10% от общего предложения)
    pub fn purchase_founder_option(
        ctx: Context<PurchaseFounderOption>,
        amount: u64,
        ndollar_payment: u64,
    ) -> Result<()> {
        let coin_data = &ctx.accounts.coin_data;
        let creator = &ctx.accounts.creator;
        
        // Проверка, что вызывающий является создателем монеты
        require!(
            coin_data.creator == creator.key(),
            GenesisError::NotCoinCreator
        );
        
        // Проверка, что запрашиваемая сумма не превышает 10% от максимального предложения
        let max_supply: u64 = 1_000_000_000 * 10u64.pow(9); // 1 миллиард токенов
        let max_founder_allocation = max_supply / 10; // 10% от максимального предложения
        let current_allocation = coin_data.total_supply;
        
        require!(
            current_allocation + amount <= max_founder_allocation,
            GenesisError::ExceedsFounderAllocation
        );
        
        // Переводим N-Dollar в качестве оплаты
        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.ndollar_token_account.to_account_info(),
            to: ctx.accounts.fees_account.to_account_info(),
            authority: creator.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, ndollar_payment)?;
        
        // Минтим дополнительные токены создателю
        let seeds = &[
            b"coin_data".as_ref(),
            &ctx.accounts.mint.key().to_bytes(),
            &[coin_data.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.creator_token_account.to_account_info(),
            authority: coin_data.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, amount)?;
        
        // Обновляем информацию о токенах в coin_data
        let coin_data = &mut ctx.accounts.coin_data;
        coin_data.total_supply += amount;
        
        msg!("Основатель приобрел дополнительные токены: {}", amount);
        Ok(())
    }

    /// Генерация уникальной реферальной ссылки
    pub fn generate_referral_link(
        ctx: Context<GenerateReferralLink>,
    ) -> Result<()> {
        let coin_data = &mut ctx.accounts.coin_data;
        let creator = &ctx.accounts.creator;
        
        // Проверка, что вызывающий является создателем монеты
        require!(
            coin_data.creator == creator.key(),
            GenesisError::NotCoinCreator
        );
        
        // Проверка, что реферальная ссылка еще не активирована
        require!(
            !coin_data.referral_link_active,
            GenesisError::ReferralLinkAlreadyActive
        );
        
        // Инициализируем реферальный аккаунт
        let referral_data = &mut ctx.accounts.referral_data;
        referral_data.coin_mint = coin_data.mint;
        referral_data.creator = creator.key();
        referral_data.creation_time = Clock::get()?.unix_timestamp;
        referral_data.referred_users = 0;
        referral_data.total_rewards = 0;
        referral_data.bump = ctx.bumps.referral_data;
        
        // Отмечаем, что реферальная ссылка активирована
        coin_data.referral_link_active = true;
        
        // CPI для инициализации в модуле реферальной системы можно будет добавить позже
        
        msg!("Реферальная ссылка сгенерирована для монеты: {}", coin_data.symbol);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String, ndollar_payment: u64)]
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
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Authority для метаданных
    #[account(mut)]
    pub mint_authority: UncheckedAccount<'info>,
    
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
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: Это токен ассоциированная программа
    pub associated_token_program: UncheckedAccount<'info>,
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PurchaseFounderOption<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
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
        constraint = ndollar_token_account.owner == creator.key()
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
    pub creator: Signer<'info>,
    
    #[account(
        constraint = mint.key() == coin_data.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"coin_data".as_ref(), mint.key().as_ref()],
        bump = coin_data.bump,
        constraint = coin_data.creator == creator.key()
    )]
    pub coin_data: Account<'info, CoinData>,
    
    #[account(
        init,
        payer = creator,
        seeds = [b"referral_data".as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + ReferralData::SPACE
    )]
    pub referral_data: Account<'info, ReferralData>,
    
    pub system_program: Program<'info, System>,
}

#[account]
pub struct CoinData {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub creation_time: i64,
    pub total_supply: u64,
    pub referral_link_active: bool,
    pub bump: u8,
}

impl CoinData {
    // Размер для хранения строковых полей (название и символ) может быть динамическим
    // но мы выделим немного больше чтобы быть уверенными
    pub const SPACE: usize = 32 + 32 + 50 + 10 + 8 + 8 + 1 + 1;
}

#[account]
pub struct ReferralData {
    pub coin_mint: Pubkey,
    pub creator: Pubkey,
    pub creation_time: i64,
    pub referred_users: u64,
    pub total_rewards: u64,
    pub bump: u8,
}

impl ReferralData {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 8 + 1;
}

#[error_code]
pub enum GenesisError {
    #[msg("Вы не являетесь создателем этой монеты")]
    NotCoinCreator,
    #[msg("Превышена максимальная аллокация для основателя (10%)")]
    ExceedsFounderAllocation,
    #[msg("Реферальная ссылка уже активна")]
    ReferralLinkAlreadyActive,
}
