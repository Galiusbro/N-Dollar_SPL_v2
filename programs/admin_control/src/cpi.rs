use crate::*;
use anchor_lang::solana_program;

/// Модуль определения типов и структур для CPI
pub mod account {
    use super::*;
    
    /// Определяет структуру для передачи аккаунтов инструкции initialize_admin
    #[derive(Accounts)]
    pub struct InitializeAdmin<'info> {
        pub authority: Signer<'info>,
        
        /// CHECK: Это PDA аккаунт, который будет инициализирован или проверен в вызываемой программе.
        pub admin_config: AccountInfo<'info>,
        
        /// CHECK: Это системная программа, которая необходима для инициализации аккаунтов.
        pub system_program: AccountInfo<'info>,
    }
    
    /// Определяет структуру для передачи аккаунтов инструкции initialize_ndollar
    #[derive(Accounts)]
    pub struct InitializeNDollar<'info> {
        pub authority: Signer<'info>,
        
        /// CHECK: Это PDA аккаунт admin_config, который будет проверен вызываемой программой.
        pub admin_config: AccountInfo<'info>,
        
        /// CHECK: Это mint-аккаунт, адрес которого будет сохранен в admin_config.
        pub ndollar_mint: AccountInfo<'info>,
    }
    
    /// Определяет структуру для передачи аккаунтов инструкции update_fees
    #[derive(Accounts)]
    pub struct UpdateFees<'info> {
        pub authority: Signer<'info>,
        
        /// CHECK: Это PDA аккаунт admin_config, который будет проверен вызываемой программой.
        pub admin_config: AccountInfo<'info>,
    }
    
    /// Определяет структуру для передачи аккаунтов инструкции authorize_program
    #[derive(Accounts)]
    pub struct AuthorizeProgram<'info> {
        pub authority: Signer<'info>,
        
        /// CHECK: Это PDA аккаунт admin_config, который будет проверен вызываемой программой.
        pub admin_config: AccountInfo<'info>,
    }
}

/// Проверяет, авторизована ли указанная программа в admin_control
pub fn verify_program_authorization(
    admin_config: &AccountInfo,
    program_id: &Pubkey,
    admin_control_program: &AccountInfo,
) -> Result<bool> {
    // Проверяем, что admin_config принадлежит программе admin_control
    require!(
        admin_config.owner == admin_control_program.key,
        ErrorCode::UnauthorizedAccess
    );
    
    // Получаем данные admin_config
    let admin_config_data = AdminConfig::try_deserialize(&mut &admin_config.data.borrow()[..])?;
    
    // Проверяем, содержится ли program_id в списке авторизованных программ
    let authorized = admin_config_data.authorized_programs.contains(program_id);
    
    // Или проверяем, является ли program_id одной из основных программ экосистемы
    let is_system_program = 
        *program_id == admin_config_data.bonding_curve_program ||
        *program_id == admin_config_data.genesis_program ||
        *program_id == admin_config_data.referral_system_program ||
        *program_id == admin_config_data.trading_exchange_program ||
        *program_id == admin_config_data.liquidity_manager_program;
    
    Ok(authorized || is_system_program)
}

/// Получает информацию о комиссиях из admin_config
pub fn get_fee_basis_points(
    admin_config: &AccountInfo,
    admin_control_program: &AccountInfo,
) -> Result<u16> {
    // Проверяем, что admin_config принадлежит программе admin_control
    require!(
        admin_config.owner == admin_control_program.key,
        ErrorCode::UnauthorizedAccess
    );
    
    // Получаем данные admin_config
    let admin_config_data = AdminConfig::try_deserialize(&mut &admin_config.data.borrow()[..])?;
    
    Ok(admin_config_data.fee_basis_points)
}

/// Проверяет, инициализирован ли указанный модуль
pub fn is_module_initialized(
    admin_config: &AccountInfo,
    module_bit: u8,
    admin_control_program: &AccountInfo,
) -> Result<bool> {
    // Проверяем, что admin_config принадлежит программе admin_control
    require!(
        admin_config.owner == admin_control_program.key,
        ErrorCode::UnauthorizedAccess
    );
    
    // Получаем данные admin_config
    let admin_config_data = AdminConfig::try_deserialize(&mut &admin_config.data.borrow()[..])?;
    
    Ok(admin_config_data.initialized_modules & module_bit != 0)
}

/// Получение PDA admin_config по authority
pub fn derive_admin_config_address(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"admin_config".as_ref(), authority.as_ref()],
        program_id
    )
}

/// Модуль для удобного вызова CPI
pub mod cpi {
    use super::*;
    
    /// Выполняет CPI вызов к инструкции initialize_admin
    pub fn initialize_admin(
        program: AccountInfo<'_>,
        accounts: account::InitializeAdmin<'_>,
    ) -> Result<()> {
        // Создаем дискриминатор для инструкции
        let disc = anchor_lang::solana_program::hash::hash("global:initialize_admin".as_bytes());
        let initialize_admin_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Определяем аккаунты для инструкции
        let ix_accounts = vec![
            AccountMeta::new(accounts.authority.key(), true),
            AccountMeta::new(accounts.admin_config.key(), false),
            AccountMeta::new_readonly(accounts.system_program.key(), false),
        ];
        
        // Создаем инструкцию
        let ix = solana_program::instruction::Instruction {
            program_id: program.key(),
            accounts: ix_accounts,
            data: initialize_admin_discriminator,
        };
        
        // Выполняем инструкцию
        solana_program::program::invoke(
            &ix,
            &[
                accounts.authority.to_account_info(),
                accounts.admin_config,
                accounts.system_program,
            ],
        )?;
        
        Ok(())
    }
    
    /// Выполняет CPI вызов к инструкции initialize_ndollar
    pub fn initialize_ndollar(
        program: AccountInfo<'_>,
        accounts: account::InitializeNDollar<'_>,
    ) -> Result<()> {
        // Создаем дискриминатор для инструкции
        let disc = anchor_lang::solana_program::hash::hash("global:initialize_ndollar".as_bytes());
        let initialize_ndollar_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Определяем аккаунты для инструкции
        let ix_accounts = vec![
            AccountMeta::new(accounts.authority.key(), true),
            AccountMeta::new(accounts.admin_config.key(), false),
            AccountMeta::new_readonly(accounts.ndollar_mint.key(), false),
        ];
        
        // Создаем инструкцию
        let ix = solana_program::instruction::Instruction {
            program_id: program.key(),
            accounts: ix_accounts,
            data: initialize_ndollar_discriminator,
        };
        
        // Выполняем инструкцию
        solana_program::program::invoke(
            &ix,
            &[
                accounts.authority.to_account_info(),
                accounts.admin_config,
                accounts.ndollar_mint,
            ],
        )?;
        
        Ok(())
    }
    
    /// Выполняет CPI вызов к инструкции update_fees
    pub fn update_fees(
        program: AccountInfo<'_>,
        accounts: account::UpdateFees<'_>,
        fee_basis_points: u16,
    ) -> Result<()> {
        // Создаем дискриминатор для инструкции
        let disc = anchor_lang::solana_program::hash::hash("global:update_fees".as_bytes());
        let update_fees_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Подготавливаем данные инструкции
        let mut ix_data = update_fees_discriminator;
        ix_data.extend_from_slice(&fee_basis_points.to_le_bytes());
        
        // Определяем аккаунты для инструкции
        let ix_accounts = vec![
            AccountMeta::new(accounts.authority.key(), true),
            AccountMeta::new(accounts.admin_config.key(), false),
        ];
        
        // Создаем инструкцию
        let ix = solana_program::instruction::Instruction {
            program_id: program.key(),
            accounts: ix_accounts,
            data: ix_data,
        };
        
        // Выполняем инструкцию
        solana_program::program::invoke(
            &ix,
            &[
                accounts.authority.to_account_info(),
                accounts.admin_config,
            ],
        )?;
        
        Ok(())
    }
    
    /// Выполняет CPI вызов к инструкции authorize_program
    pub fn authorize_program(
        program: AccountInfo<'_>,
        accounts: account::AuthorizeProgram<'_>,
        program_id_to_authorize: Pubkey,
    ) -> Result<()> {
        // Создаем дискриминатор для инструкции
        let disc = anchor_lang::solana_program::hash::hash("global:authorize_program".as_bytes());
        let authorize_program_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Подготавливаем данные инструкции
        let mut ix_data = authorize_program_discriminator;
        ix_data.extend_from_slice(&program_id_to_authorize.to_bytes());
        
        // Определяем аккаунты для инструкции
        let ix_accounts = vec![
            AccountMeta::new(accounts.authority.key(), true),
            AccountMeta::new(accounts.admin_config.key(), false),
        ];
        
        // Создаем инструкцию
        let ix = solana_program::instruction::Instruction {
            program_id: program.key(),
            accounts: ix_accounts,
            data: ix_data,
        };
        
        // Выполняем инструкцию
        solana_program::program::invoke(
            &ix,
            &[
                accounts.authority.to_account_info(),
                accounts.admin_config,
            ],
        )?;
        
        Ok(())
    }
    
    /// Выполняет CPI вызов с подписью PDA
    pub fn update_fees_with_pda_signature(
        program: AccountInfo<'_>,
        accounts: account::UpdateFees<'_>,
        fee_basis_points: u16,
        seeds: &[&[u8]],
    ) -> Result<()> {
        // Создаем дискриминатор для инструкции
        let disc = anchor_lang::solana_program::hash::hash("global:update_fees".as_bytes());
        let update_fees_discriminator = disc.to_bytes()[..8].to_vec();
        
        // Подготавливаем данные инструкции
        let mut ix_data = update_fees_discriminator;
        ix_data.extend_from_slice(&fee_basis_points.to_le_bytes());
        
        // Определяем аккаунты для инструкции
        let ix_accounts = vec![
            AccountMeta::new(accounts.authority.key(), true), // PDA как подписант
            AccountMeta::new(accounts.admin_config.key(), false),
        ];
        
        // Создаем инструкцию
        let ix = solana_program::instruction::Instruction {
            program_id: program.key(),
            accounts: ix_accounts,
            data: ix_data,
        };
        
        // Выполняем инструкцию с подписью PDA
        solana_program::program::invoke_signed(
            &ix,
            &[
                accounts.authority.to_account_info(),
                accounts.admin_config,
            ],
            &[seeds],
        )?;
        
        Ok(())
    }
} 