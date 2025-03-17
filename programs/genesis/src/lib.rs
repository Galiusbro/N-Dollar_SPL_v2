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
        admin: Option<Pubkey>,
    ) -> Result<()> {
        // Проверка длины и содержимого имени и символа
        require!(name.len() >= 3, GenesisError::NameTooShort);
        require!(name.len() <= 40, GenesisError::NameTooLong);
        require!(symbol.len() >= 2, GenesisError::SymbolTooShort);
        require!(symbol.len() <= 8, GenesisError::SymbolTooLong);
        
        // Проверка на допустимые символы (буквы, цифры, пробелы и некоторые специальные символы)
        let valid_chars = |s: &str| -> bool {
            s.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.:;,?!()[]{}\"'".contains(c))
        };
        
        require!(valid_chars(&name), GenesisError::InvalidCharacters);
        require!(valid_chars(&symbol), GenesisError::InvalidCharacters);
        
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
        
        // Устанавливаем администратора
        // Если передан, используем его, иначе создатель становится администратором
        coin_data.admin = match admin {
            Some(admin_pubkey) => admin_pubkey,
            None => creator.key(),
        };
        
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
        
        // CPI вызов для инициализации бондинговой кривой
        let power: u8 = 2; // Стандартный показатель степени для кривой
        let initial_price: u64 = 5_000_000; // Начальная цена 0.00005 N-Dollar (с учетом 9 десятичных знаков)
        let fee_percent: u16 = 50; // 0.5% комиссия (в базисных пунктах)
        
        // Рассчитываем дискриминатор для инструкции initialize_bonding_curve
        let disc = anchor_lang::solana_program::hash::hash("global:initialize_bonding_curve".as_bytes());
        let initialize_bonding_curve_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Готовим данные для инструкции
        let mut ix_data = initialize_bonding_curve_discriminator;
        ix_data.extend_from_slice(&ctx.accounts.mint.key().to_bytes()); // coin_mint
        ix_data.extend_from_slice(&initial_price.to_le_bytes()); // initial_price
        
        // Добавляем опциональные параметры как Some(value)
        ix_data.push(1); // Для Some(power)
        ix_data.extend_from_slice(&power.to_le_bytes());
        
        ix_data.push(1); // Для Some(fee_percent)
        ix_data.extend_from_slice(&fee_percent.to_le_bytes());
        
        // Определяем аккаунты для CPI вызова
        let ix_accounts = vec![
            AccountMeta::new(ctx.accounts.creator.key(), true), // creator (signer)
            AccountMeta::new(ctx.accounts.bonding_curve.key(), false), // bonding_curve PDA
            AccountMeta::new(ctx.accounts.mint.key(), false), // coin_mint
            AccountMeta::new(ctx.accounts.ndollar_mint.key(), false), // ndollar_mint
            AccountMeta::new(ctx.accounts.liquidity_pool.key(), false), // liquidity_pool
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false), // token_program
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false), // system_program
            AccountMeta::new_readonly(ctx.accounts.rent.key(), false), // rent
        ];
        
        // Создаем инструкцию
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.bonding_curve_program.key(),
            accounts: ix_accounts,
            data: ix_data,
        };
        
        // Выполняем инструкцию
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.bonding_curve.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.ndollar_mint.to_account_info(),
                ctx.accounts.liquidity_pool.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;
        
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
        let admin = &ctx.accounts.admin;
        let creator = ctx.accounts.creator.key();
        
        // Проверка, что вызывающий является администратором монеты
        require!(
            coin_data.admin == admin.key(),
            GenesisError::NotCoinAdmin
        );
        
        // Проверка, что токены покупаются для создателя
        require!(
            coin_data.creator == creator,
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
            authority: admin.to_account_info(),
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
        let authority = &ctx.accounts.authority;
        
        // Проверка, что вызывающий является администратором монеты
        require!(
            coin_data.admin == authority.key(),
            GenesisError::NotCoinAdmin
        );
        
        // Проверка, что реферальная ссылка еще не активирована
        require!(
            !coin_data.referral_link_active,
            GenesisError::ReferralLinkAlreadyActive
        );
        
        // Инициализируем реферальный аккаунт
        let referral_data = &mut ctx.accounts.referral_data;
        referral_data.coin_mint = coin_data.mint;
        referral_data.creator = coin_data.creator;
        referral_data.creation_time = Clock::get()?.unix_timestamp;
        referral_data.referred_users = 0;
        referral_data.total_rewards = 0;
        referral_data.bump = ctx.bumps.referral_data;
        
        // Отмечаем, что реферальная ссылка активирована
        coin_data.referral_link_active = true;
        
        // CPI для инициализации в модуле реферальной системы
        // Рассчитываем дискриминатор для инструкции initialize_referral_system
        let disc = anchor_lang::solana_program::hash::hash("global:initialize_referral_system".as_bytes());
        let initialize_referral_system_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Готовим данные для инструкции
        let mut ix_data = initialize_referral_system_discriminator;
        ix_data.extend_from_slice(&ctx.accounts.mint.key().to_bytes()); // coin_mint
        
        // Определяем аккаунты для CPI вызова
        let ix_accounts = vec![
            AccountMeta::new(ctx.accounts.authority.key(), true), // authority (signer)
            AccountMeta::new(ctx.accounts.referral_system.key(), false), // referral_system PDA
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false), // system_program
            AccountMeta::new_readonly(ctx.accounts.rent.key(), false), // rent
        ];
        
        // Создаем инструкцию
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.referral_system_program.key(),
            accounts: ix_accounts,
            data: ix_data,
        };
        
        // Выполняем инструкцию
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.referral_system.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;
        
        msg!("Реферальная ссылка сгенерирована для монеты: {}", coin_data.symbol);
        Ok(())
    }

    /// Передача административных прав другому пользователю
    pub fn transfer_admin_rights(
        ctx: Context<TransferAdminRights>,
        new_admin: Pubkey,
    ) -> Result<()> {
        let coin_data = &mut ctx.accounts.coin_data;
        let current_admin = &ctx.accounts.current_admin;
        
        // Проверка, что вызывающий является текущим администратором
        require!(
            coin_data.admin == current_admin.key(),
            GenesisError::NotCoinAdmin
        );
        
        // Обновляем администратора
        coin_data.admin = new_admin;
        
        msg!("Права администратора переданы новому пользователю: {}", new_admin);
        Ok(())
    }
}

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

#[account]
pub struct CoinData {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub creation_time: i64,
    pub total_supply: u64,
    pub referral_link_active: bool,
    pub admin: Pubkey,
    pub bump: u8,
}

impl CoinData {
    // Размер для хранения строковых полей (название и символ) может быть динамическим
    // но мы выделим немного больше чтобы быть уверенными
    pub const SPACE: usize = 32 + 32 + 50 + 10 + 8 + 8 + 1 + 32 + 1;
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
    #[msg("Название токена слишком короткое (минимум 3 символа)")]
    NameTooShort,
    #[msg("Название токена слишком длинное (максимум 40 символов)")]
    NameTooLong,
    #[msg("Символ токена слишком короткий (минимум 2 символа)")]
    SymbolTooShort,
    #[msg("Символ токена слишком длинный (максимум 8 символов)")]
    SymbolTooLong,
    #[msg("Название или символ содержат недопустимые символы")]
    InvalidCharacters,
    #[msg("Вы не являетесь администратором этой монеты")]
    NotCoinAdmin,
}
