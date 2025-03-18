use anchor_lang::prelude::*;
use crate::errors::BondingCurveError;
use admin_control::admin_cpi::verify_program_authorization;
//get_fee_basis_points

/// Вспомогательная функция для проверки авторизации программы через admin_control
pub fn verify_program_auth<'info>(
    admin_config: &AccountInfo<'info>,
    admin_control_program: &AccountInfo<'info>
) -> Result<()> {
    // Проверяем, что текущая программа авторизована
    let is_authorized = verify_program_authorization(
        admin_config,
        &crate::ID,
        admin_control_program,
    )?;
    
    require!(is_authorized, BondingCurveError::UnauthorizedAccess);
    Ok(())
}