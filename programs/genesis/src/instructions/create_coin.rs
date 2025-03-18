use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3};
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::solana_program::instruction::{Instruction, AccountMeta};
use anchor_lang::solana_program::program::invoke;

use crate::errors::GenesisError;
use crate::instructions::contexts::CreateCoin;
use crate::constants::*;
use crate::utils::*;

pub fn handler(
    ctx: Context<CreateCoin>,
    name: String,
    symbol: String,
    uri: String,
    ndollar_payment: u64,
    admin: Option<Pubkey>,
) -> Result<()> {
    // Проверка длины и содержимого имени и символа
    require!(name.len() >= MIN_NAME_LENGTH, GenesisError::NameTooShort);
    require!(name.len() <= MAX_NAME_LENGTH, GenesisError::NameTooLong);
    require!(symbol.len() >= MIN_SYMBOL_LENGTH, GenesisError::SymbolTooShort);
    require!(symbol.len() <= MAX_SYMBOL_LENGTH, GenesisError::SymbolTooLong);
    
    // Проверка на допустимые символы
    require!(validate_string(&name), GenesisError::InvalidCharacters);
    require!(validate_string(&symbol), GenesisError::InvalidCharacters);
    
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
    
    // Сразу минтим начальное предложение токенов создателю
    let initial_supply = calculate_initial_supply();
    
    // Получаем байты ключа минта для seeds
    let mint = ctx.accounts.mint.key();
    let bump = &coin_data.bump;
    let seeds = get_coin_data_seeds(&mint, bump);
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
    let power: u8 = BONDING_CURVE_POWER;
    let initial_price: u64 = INITIAL_PRICE;
    let fee_percent: u16 = FEE_PERCENT;
    
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
    let ix = Instruction {
        program_id: ctx.accounts.bonding_curve_program.key(),
        accounts: ix_accounts,
        data: ix_data,
    };
    
    // Выполняем инструкцию
    invoke(
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