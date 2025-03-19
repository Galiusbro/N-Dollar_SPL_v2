use anchor_lang::prelude::*;
use crate::contexts::AdminFunction;
use crate::errors::NDollarError;
use crate::constants::MAX_MIN_SIGNERS;

/// Добавление авторизованного подписанта
pub fn add_authorized_signer(ctx: Context<AdminFunction>, new_signer: Pubkey) -> Result<()> {
    let admin_account = &mut ctx.accounts.admin_account;
    
    // Только основной авторизованный пользователь может добавлять новые ключи
    require!(
        admin_account.authority == ctx.accounts.authority.key(),
        NDollarError::UnauthorizedAccess
    );
    
    // Находим пустой слот для нового подписанта
    let mut added = false;
    for signer_slot in admin_account.authorized_signers.iter_mut() {
        if signer_slot.is_none() {
            *signer_slot = Some(new_signer);
            added = true;
            break;
        }
    }
    
    require!(added, NDollarError::UnauthorizedAccess); // Если нет места для нового подписанта
    
    msg!("Авторизованный подписант добавлен");
    Ok(())
}

/// Удаление авторизованного подписанта
pub fn remove_authorized_signer(ctx: Context<AdminFunction>, signer_to_remove: Pubkey) -> Result<()> {
    let admin_account = &mut ctx.accounts.admin_account;
    
    // Только основной авторизованный пользователь может удалять ключи
    require!(
        admin_account.authority == ctx.accounts.authority.key(),
        NDollarError::UnauthorizedAccess
    );
    
    // Находим и удаляем подписанта
    let mut removed = false;
    for signer_slot in admin_account.authorized_signers.iter_mut() {
        if let Some(key) = signer_slot {
            if *key == signer_to_remove {
                *signer_slot = None;
                removed = true;
                break;
            }
        }
    }
    
    require!(removed, NDollarError::UnauthorizedAccess); // Если подписант не найден
    
    msg!("Авторизованный подписант удален");
    Ok(())
}

/// Установка минимального количества подписантов
pub fn set_min_required_signers(ctx: Context<AdminFunction>, min_signers: u8) -> Result<()> {
    let admin_account = &mut ctx.accounts.admin_account;
    
    // Только основной авторизованный пользователь может менять настройки
    require!(
        admin_account.authority == ctx.accounts.authority.key(),
        NDollarError::UnauthorizedAccess
    );
    
    // Проверяем, что требуемое количество не превышает максимально возможное
    require!(min_signers <= MAX_MIN_SIGNERS, NDollarError::UnauthorizedAccess);
    
    admin_account.min_required_signers = min_signers;
    
    msg!("Установлено минимальное количество подписантов: {}", min_signers);
    Ok(())
}
