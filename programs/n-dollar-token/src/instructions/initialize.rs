use anchor_lang::prelude::*;
// use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3};
// use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use anchor_spl::token::{self, MintTo};
use admin_control::admin_cpi;
use crate::contexts::InitializeNDollar;
use crate::constants::*;
use crate::errors::NDollarError;

/// Инициализирует токен N-Dollar
pub fn initialize_n_dollar(
    ctx: Context<InitializeNDollar>,
    name: String,
    symbol: String,
    uri: String,
    _decimals: u8,
) -> Result<()> {
    // Инициализация минта токена
    let _mint = &ctx.accounts.mint;
    
    /* Временно отключено для тестирования
    // Установка метаданных через MetaplexMetadata
    let metadata_accounts = CreateMetadataAccountsV3 {
        metadata: ctx.accounts.metadata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        mint_authority: ctx.accounts.authority.to_account_info(),
        payer: ctx.accounts.authority.to_account_info(),
        update_authority: ctx.accounts.authority.to_account_info(),
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
    */

    // Инициализация контрольного аккаунта
    let admin_account = &mut ctx.accounts.admin_account;
    admin_account.authority = ctx.accounts.authority.key();
    admin_account.mint = ctx.accounts.mint.key();
    admin_account.last_mint_time = Clock::get()?.unix_timestamp;
    admin_account.total_supply = 0;
    admin_account.bump = ctx.bumps.admin_account;
    
    // Инициализация новых полей
    admin_account.authorized_signers = [None, None, None];
    admin_account.last_block_time = Clock::get()?.unix_timestamp;
    admin_account.last_block_height = Clock::get()?.slot;
    admin_account.min_required_signers = DEFAULT_MIN_SIGNERS;
    admin_account.current_mint_week = 1; // Первая неделя в расписании

    // Регистрируем адрес минта N-Dollar в admin_control, если admin_config передан
    if ctx.accounts.admin_config.is_some() && ctx.accounts.admin_control_program.is_some() {
        // Инициализируем N-Dollar в admin_control через CPI
        let admin_cpi_accounts = admin_cpi::account::InitializeNDollar {
            authority: ctx.accounts.authority.to_account_info(),
            admin_config: ctx.accounts.admin_config.as_ref().unwrap().to_account_info(),
            ndollar_mint: ctx.accounts.mint.to_account_info(),
        };
        
        admin_cpi::direct_cpi::initialize_ndollar(
            ctx.accounts.admin_control_program.as_ref().unwrap().to_account_info(),
            admin_cpi_accounts,
        )?;
        
        msg!("N-Dollar зарегистрирован в admin_control");
    }

    // Расчет распределения токенов между админом и пулом ликвидности
    let liquidity_amount = INITIAL_MINT_AMOUNT
        .checked_mul(LIQUIDITY_POOL_PERCENTAGE as u64)
        .and_then(|v| v.checked_div(100))
        .ok_or(NDollarError::ArithmeticError)?;
    
    let admin_amount = INITIAL_MINT_AMOUNT
        .checked_mul(ADMIN_RESERVE_PERCENTAGE as u64)
        .and_then(|v| v.checked_div(100))
        .ok_or(NDollarError::ArithmeticError)?;
    
    // Проверка, что суммы корректно рассчитаны
    let total_minted = liquidity_amount
        .checked_add(admin_amount)
        .ok_or(NDollarError::ArithmeticError)?;
    
    require!(
        total_minted == INITIAL_MINT_AMOUNT,
        NDollarError::ArithmeticError
    );

    // Автоматический минт начального количества токенов при инициализации
    let seeds = &[
        b"admin_account".as_ref(),
        &admin_account.mint.to_bytes(),
        &[admin_account.bump],
    ];
    let signer = &[&seeds[..]];
    
    // Минтим токены администратору (резервная часть)
    if admin_amount > 0 {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.admin_token_account.to_account_info(),
            authority: admin_account.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, admin_amount)?;
        msg!("Минт администратору: {}", admin_amount);
    }
    
    // Минтим токены в пул ликвидности
    if liquidity_amount > 0 && ctx.accounts.liquidity_pool_account.is_some() {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.liquidity_pool_account.as_ref().unwrap().to_account_info(),
            authority: admin_account.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, liquidity_amount)?;
        msg!("Минт в пул ликвидности: {}", liquidity_amount);
    } else if liquidity_amount > 0 {
        // Если аккаунт пула ликвидности не предоставлен, минтим все администратору
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.admin_token_account.to_account_info(),
            authority: admin_account.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, liquidity_amount)?;
        msg!("Аккаунт пула ликвидности не предоставлен, минт направлен администратору: {}", liquidity_amount);
    }
    
    // Обновляем информацию о минте
    admin_account.total_supply = INITIAL_MINT_AMOUNT;
    
    msg!("N-Dollar Token успешно инициализирован и выпущено {} токенов", INITIAL_MINT_AMOUNT);
    Ok(())
}
