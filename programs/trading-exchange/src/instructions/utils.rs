use anchor_lang::prelude::*;
use admin_control::admin_cpi;
use crate::errors::TradingError;
use crate::constants::DEFAULT_FEE_PERCENTAGE;

/// Проверяет авторизацию через admin_control
pub fn verify_admin_control_authorization<'info>(
    admin_config: &Option<AccountInfo<'info>>,
    admin_control_program: &Option<AccountInfo<'info>>,
) -> Result<()> {
    if admin_config.is_some() && admin_control_program.is_some() {
        // Проверка, что текущая программа авторизована в admin_control
        let program_id = crate::ID;
        let is_authorized = admin_cpi::verify_program_authorization(
            &admin_config.as_ref().unwrap().to_account_info(),
            &program_id,
            &admin_control_program.as_ref().unwrap().to_account_info(),
        )?;
        
        require!(is_authorized, TradingError::UnauthorizedAccess);
    }
    
    Ok(())
}

/// Получает процент комиссии из admin_control или возвращает значение по умолчанию
pub fn get_fee_percentage<'info>(
    admin_config: &Option<AccountInfo<'info>>,
    admin_control_program: &Option<AccountInfo<'info>>,
) -> Result<u64> {
    if admin_config.is_some() && admin_control_program.is_some() {
        // Получаем комиссию из admin_config
        let fee_basis_points = admin_cpi::get_fee_basis_points(
            &admin_config.as_ref().unwrap().to_account_info(),
            &admin_control_program.as_ref().unwrap().to_account_info(),
        )?;
        
        // Преобразуем из базисных пунктов (1/100 процента) в проценты
        Ok(fee_basis_points as u64 / 100)
    } else {
        Ok(DEFAULT_FEE_PERCENTAGE)
    }
}

/// Создает CPI инструкцию для вызова внешней программы
pub fn create_cpi_instruction(
    program_id: Pubkey,
    accounts: Vec<AccountMeta>,
    discriminator: &[u8],
    data: Option<Vec<u8>>,
) -> anchor_lang::solana_program::instruction::Instruction {
    // Подготовка данных инструкции с дискриминатором
    let disc = anchor_lang::solana_program::hash::hash(discriminator);
    let mut ix_data = disc.to_bytes()[..8].to_vec();
    
    // Добавляем дополнительные данные, если они есть
    if let Some(mut additional_data) = data {
        ix_data.append(&mut additional_data);
    }
    
    anchor_lang::solana_program::instruction::Instruction {
        program_id,
        accounts,
        data: ix_data,
    }
}
