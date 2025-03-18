use anchor_lang::prelude::*;

use crate::errors::GenesisError;
use crate::instructions::contexts::TransferAdminRights;

pub fn handler(
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