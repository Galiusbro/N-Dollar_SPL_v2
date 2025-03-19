use anchor_lang::prelude::*;
use crate::contexts::UpdateMetadata;
use crate::errors::NDollarError;

/// Обновление метаданных токена (административная функция)
pub fn update_metadata(
    ctx: Context<UpdateMetadata>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let admin_account = &ctx.accounts.admin_account;
    
    // Только авторизованный пользователь может обновлять метаданные
    require!(
        admin_account.authority == ctx.accounts.authority.key(),
        NDollarError::UnauthorizedAccess
    );

    // Обновление метаданных через CPI к токен-метадата программе
    let seeds = &[
        b"admin_account".as_ref(),
        &admin_account.mint.to_bytes(),
        &[admin_account.bump],
    ];
    let signer = &[&seeds[..]];

    // Создаем дискриминатор инструкции update_metadata_accounts_v2
    let disc = anchor_lang::solana_program::hash::hash("global:update_metadata_accounts_v2".as_bytes());
    let update_metadata_discriminator = disc.to_bytes()[..8].to_vec();
    
    // Подготавливаем данные для инструкции
    let mut ix_data = update_metadata_discriminator;
    
    // Добавляем данные в формате, который ожидает метаплекс:
    // Option<Data> - Some(Data) с полями name, symbol, uri
    ix_data.push(1); // Some(data)
    
    // Сериализуем название
    let name_bytes = name.as_bytes();
    ix_data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    ix_data.extend_from_slice(name_bytes);
    
    // Сериализуем символ
    let symbol_bytes = symbol.as_bytes();
    ix_data.extend_from_slice(&(symbol_bytes.len() as u32).to_le_bytes());
    ix_data.extend_from_slice(symbol_bytes);
    
    // Сериализуем URI
    let uri_bytes = uri.as_bytes();
    ix_data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
    ix_data.extend_from_slice(uri_bytes);
    
    // Seller fee basis points (0)
    ix_data.extend_from_slice(&0u16.to_le_bytes());
    
    // Creators - None
    ix_data.push(0);
    
    // Collection - None
    ix_data.push(0);
    
    // Uses - None
    ix_data.push(0);
    
    // Option<bool> для поля primary_sale_happened - None
    ix_data.push(0);
    
    // Option<bool> для поля is_mutable - None
    ix_data.push(0);
    
    // Определяем аккаунты для инструкции
    let ix_accounts = vec![
        AccountMeta::new(ctx.accounts.metadata.key(), false),
        AccountMeta::new_readonly(admin_account.to_account_info().key(), true),
    ];
    
    // Создаем инструкцию
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.accounts.metadata_program.key(),
        accounts: ix_accounts,
        data: ix_data,
    };
    
    // Выполняем инструкцию с PDA подписью
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.metadata.to_account_info(),
            admin_account.to_account_info(),
        ],
        signer,
    )?;
    
    msg!("Метаданные обновлены успешно");
    Ok(())
}
