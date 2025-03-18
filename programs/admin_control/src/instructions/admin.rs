use anchor_lang::prelude::*;
use crate::contexts::{UpdateFees, AuthorizeProgram, RevokeProgram, UpgradeAdminConfig};
use crate::ErrorCode;

/// Обновление комиссионных ставок в админской конфигурации
pub fn update_fees(ctx: Context<UpdateFees>, fee_basis_points: u16) -> Result<()> {
    // Проверка, что комиссия не превышает 10%
    require!(
        fee_basis_points <= 1000, 
        ErrorCode::FeeTooHigh
    );
    
    let admin_config = &mut ctx.accounts.admin_config;
    admin_config.fee_basis_points = fee_basis_points;
    
    msg!("Fee updated to {} basis points", fee_basis_points);
    Ok(())
}

/// Авторизация новой программы для взаимодействия с экосистемой
pub fn authorize_program(ctx: Context<AuthorizeProgram>, program_id: Pubkey) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    
    // Проверяем, не добавлена ли уже эта программа
    for i in 0..admin_config.authorized_programs.len() {
        if admin_config.authorized_programs[i] == program_id {
            return Err(ErrorCode::ProgramAlreadyAuthorized.into());
        }
        
        // Если нашли пустой слот, добавляем новую программу
        if admin_config.authorized_programs[i] == Pubkey::default() {
            admin_config.authorized_programs[i] = program_id;
            msg!("Program {} is now authorized", program_id);
            return Ok(());
        }
    }
    
    // Если не нашли свободных слотов
    Err(ErrorCode::TooManyAuthorizedPrograms.into())
}

/// Отзыв авторизации у программы
pub fn revoke_program_authorization(ctx: Context<RevokeProgram>, program_id: Pubkey) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    
    for i in 0..admin_config.authorized_programs.len() {
        if admin_config.authorized_programs[i] == program_id {
            admin_config.authorized_programs[i] = Pubkey::default();
            msg!("Program {} authorization revoked", program_id);
            return Ok(());
        }
    }
    
    Err(ErrorCode::ProgramNotAuthorized.into())
}

/// Обновление версии структуры AdminConfig при необходимости
pub fn upgrade_admin_config(ctx: Context<UpgradeAdminConfig>) -> Result<()> {
    let admin_config = &mut ctx.accounts.admin_config;
    
    // Проверяем, что текущая версия ниже чем последняя
    require!(
        admin_config.version < crate::state::AdminConfig::CURRENT_VERSION,
        ErrorCode::AlreadyUpgraded
    );
    
    // Обновляем версию
    admin_config.version = crate::state::AdminConfig::CURRENT_VERSION;
    
    // Здесь можно добавить логику миграции данных между версиями
    
    msg!("Admin Config upgraded to version {}", admin_config.version);
    Ok(())
} 