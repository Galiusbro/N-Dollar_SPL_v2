use anchor_lang::prelude::*;
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3};
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use admin_control::admin_cpi;
use crate::contexts::InitializeNDollar;
use crate::constants::*;

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

    msg!("N-Dollar Token успешно инициализирован");
    Ok(())
}
