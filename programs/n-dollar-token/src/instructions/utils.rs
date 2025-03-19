use anchor_lang::prelude::*;
use crate::errors::NDollarError;
use crate::state::AdminAccount;

/// Проверяет, имеет ли аккаунт достаточно подписей для мультиподписной операции
pub fn verify_multisig<'info>(
    admin_account: &Account<'info, AdminAccount>,
    authority: &Signer<'info>,
    additional_signer1: &Option<Signer<'info>>,
    additional_signer2: &Option<Signer<'info>>,
) -> Result<()> {
    // Проверка основного авторизованного пользователя
    require!(
        admin_account.authority == authority.key(),
        NDollarError::UnauthorizedAccess
    );
    
    // Проверка минимального количества подписей
    let mut valid_signatures = 1; // Основной авторизованный пользователь уже подписал
    
    // Проверяем дополнительных подписантов
    if let Some(signer1) = additional_signer1 {
        if admin_account.authorized_signers.iter().any(|s| s.is_some() && s.unwrap() == signer1.key()) {
            valid_signatures += 1;
        }
    }
    
    if let Some(signer2) = additional_signer2 {
        if admin_account.authorized_signers.iter().any(|s| s.is_some() && s.unwrap() == signer2.key()) {
            valid_signatures += 1;
        }
    }
    
    // Убеждаемся, что есть достаточное количество подписей
    require!(
        valid_signatures >= admin_account.min_required_signers as usize,
        NDollarError::InsufficientSigners
    );
    
    Ok(())
}

/// Проверяет авторизацию программы через admin_control
pub fn verify_admin_control_authorization<'info>(
    admin_config: &Option<AccountInfo<'info>>,
    admin_control_program: &Option<AccountInfo<'info>>,
) -> Result<()> {
    if admin_config.is_some() && admin_control_program.is_some() {
        // Проверка, что текущая программа авторизована в admin_control
        let program_id = crate::ID;
        let is_authorized = admin_control::admin_cpi::verify_program_authorization(
            &admin_config.as_ref().unwrap().to_account_info(),
            &program_id,
            &admin_control_program.as_ref().unwrap().to_account_info(),
        )?;
        
        require!(is_authorized, NDollarError::UnauthorizedAccess);
    }
    
    Ok(())
}

/// Проверяет защиту от манипуляций временем
pub fn verify_time_manipulation(
    admin_account: &Account<'_, AdminAccount>,
    current_time: i64,
    current_slot: u64,
) -> Result<()> {
    // 1. Проверка, что текущее время больше, чем последнее время блока
    require!(current_time >= admin_account.last_block_time, NDollarError::TimeManipulationDetected);
    
    // 2. Проверка последовательности блоков (номер блока должен увеличиваться)
    require!(current_slot > admin_account.last_block_height, NDollarError::TimeManipulationDetected);
    
    // 3. Проверка согласованности времени и блоков
    // Среднее время блока в Solana ~0.4 секунды, разница между блоками не должна быть слишком большой
    let expected_block_time_diff = (current_slot - admin_account.last_block_height) / 2; // Предполагаем, что 2 блока в секунду
    let actual_time_diff = (current_time - admin_account.last_block_time) as u64;
    
    // Проверяем, что разница времени не слишком сильно отличается от ожидаемой (с 50% допуском)
    require!(
        actual_time_diff <= expected_block_time_diff * 3 / 2, 
        NDollarError::TimeManipulationDetected
    );
    
    Ok(())
}
