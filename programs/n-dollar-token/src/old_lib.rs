// use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn, FreezeAccount, ThawAccount};
// use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata};
// use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
// use anchor_lang::solana_program::pubkey::Pubkey;
// // Импортируем модуль admin_control для авторизации
// use admin_control::admin_cpi;

// declare_id!("2MiMzfH55kTC9tNM8LQFLzC8GbduTMTe5SyTadBi33cL");

// #[program]
// pub mod n_dollar_token {
//     use super::*;

//     /// Инициализирует токен N-Dollar
//     pub fn initialize_n_dollar(
//         ctx: Context<InitializeNDollar>,
//         name: String,
//         symbol: String,
//         uri: String,
//         _decimals: u8,
//     ) -> Result<()> {
//         // Инициализация минта токена
//         let _mint = &ctx.accounts.mint;
        
//         // Установка метаданных через MetaplexMetadata
//         let metadata_accounts = CreateMetadataAccountsV3 {
//             metadata: ctx.accounts.metadata.to_account_info(),
//             mint: ctx.accounts.mint.to_account_info(),
//             mint_authority: ctx.accounts.authority.to_account_info(),
//             payer: ctx.accounts.authority.to_account_info(),
//             update_authority: ctx.accounts.authority.to_account_info(),
//             system_program: ctx.accounts.system_program.to_account_info(),
//             rent: ctx.accounts.rent.to_account_info(),
//         };

//         let token_data = DataV2 {
//             name,
//             symbol,
//             uri,
//             seller_fee_basis_points: 0,
//             creators: None,
//             collection: None,
//             uses: None,
//         };

//         create_metadata_accounts_v3(
//             CpiContext::new(ctx.accounts.metadata_program.to_account_info(), metadata_accounts),
//             token_data,
//             true,  // is_mutable
//             true,  // update_authority_is_signer
//             None,  // collection_details
//         )?;

//         // Инициализация контрольного аккаунта
//         let admin_account = &mut ctx.accounts.admin_account;
//         admin_account.authority = ctx.accounts.authority.key();
//         admin_account.mint = ctx.accounts.mint.key();
//         admin_account.last_mint_time = Clock::get()?.unix_timestamp;
//         admin_account.total_supply = 0;
//         admin_account.bump = ctx.bumps.admin_account;
        
//         // Инициализация новых полей
//         admin_account.authorized_signers = [None, None, None];
//         admin_account.last_block_time = Clock::get()?.unix_timestamp;
//         admin_account.last_block_height = Clock::get()?.slot;
//         admin_account.min_required_signers = 1; // По умолчанию требуется только один подписант (можно изменить позже)

//         // Регистрируем адрес минта N-Dollar в admin_control, если admin_config передан
//         if ctx.accounts.admin_config.is_some() && ctx.accounts.admin_control_program.is_some() {
//             // Инициализируем N-Dollar в admin_control через CPI
//             let admin_cpi_accounts = admin_cpi::account::InitializeNDollar {
//                 authority: ctx.accounts.authority.to_account_info(),
//                 admin_config: ctx.accounts.admin_config.as_ref().unwrap().to_account_info(),
//                 ndollar_mint: ctx.accounts.mint.to_account_info(),
//             };
            
//             admin_cpi::direct_cpi::initialize_ndollar(
//                 ctx.accounts.admin_control_program.as_ref().unwrap().to_account_info(),
//                 admin_cpi_accounts,
//             )?;
            
//             msg!("N-Dollar зарегистрирован в admin_control");
//         }

//         msg!("N-Dollar Token успешно инициализирован");
//         Ok(())
//     }

//     /// Минтинг токенов согласно расписанию
//     pub fn mint_supply(ctx: Context<MintSupply>, amount: u64) -> Result<()> {
//         // Проверка авторизации через admin_control, если admin_config передан
//         if ctx.accounts.admin_config.is_some() && ctx.accounts.admin_control_program.is_some() {
//             // Проверка, что текущая программа авторизована в admin_control
//             let program_id = crate::ID;
//             let is_authorized = admin_cpi::verify_program_authorization(
//                 &ctx.accounts.admin_config.as_ref().unwrap().to_account_info(),
//                 &program_id,
//                 &ctx.accounts.admin_control_program.as_ref().unwrap().to_account_info(),
//             )?;
            
//             require!(is_authorized, NDollarError::UnauthorizedAccess);
//         }
        
//         let admin_account = &mut ctx.accounts.admin_account;
//         let current_time = Clock::get()?.unix_timestamp;
//         let current_slot = Clock::get()?.slot;
        
//         // Проверка прошла ли неделя с последнего минта
//         // 7 дней * 24 часа * 60 минут * 60 секунд = 604800 секунд
//         let time_since_last_mint = current_time - admin_account.last_mint_time;
//         require!(time_since_last_mint >= 604800, NDollarError::TooEarlyToMint);
        
//         // Защита от атак на время
//         // 1. Проверка, что текущее время больше, чем последнее время блока
//         require!(current_time >= admin_account.last_block_time, NDollarError::TimeManipulationDetected);
        
//         // 2. Проверка последовательности блоков (номер блока должен увеличиваться)
//         require!(current_slot > admin_account.last_block_height, NDollarError::TimeManipulationDetected);
        
//         // 3. Проверка согласованности времени и блоков
//         // Среднее время блока в Solana ~0.4 секунды, разница между блоками не должна быть слишком большой
//         let expected_block_time_diff = (current_slot - admin_account.last_block_height) / 2; // Предполагаем, что 2 блока в секунду
//         let actual_time_diff = (current_time - admin_account.last_block_time) as u64;
        
//         // Проверяем, что разница времени не слишком сильно отличается от ожидаемой (с 50% допуском)
//         require!(
//             actual_time_diff <= expected_block_time_diff * 3 / 2, 
//             NDollarError::TimeManipulationDetected
//         );
        
//         // Только авторизованный пользователь может минтить
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );

//         // Минтим токены
//         let seeds = &[
//             b"admin_account".as_ref(),
//             &admin_account.mint.to_bytes(),
//             &[admin_account.bump],
//         ];
//         let signer = &[&seeds[..]];
        
//         let cpi_accounts = MintTo {
//             mint: ctx.accounts.mint.to_account_info(),
//             to: ctx.accounts.token_account.to_account_info(),
//             authority: admin_account.to_account_info(),
//         };
        
//         let cpi_program = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
//         token::mint_to(cpi_ctx, amount)?;
        
//         // Обновляем информацию о последнем минте
//         admin_account.last_mint_time = current_time;
//         admin_account.last_block_time = current_time;
//         admin_account.last_block_height = current_slot;
//         admin_account.total_supply += amount;
        
//         msg!("Минт выполнен успешно, добавлено: {}", amount);
//         Ok(())
//     }

//     /// Сжигание токенов (административная функция)
//     pub fn burn_tokens(ctx: Context<AdminFunction>, amount: u64) -> Result<()> {
//         let admin_account = &ctx.accounts.admin_account;
        
//         // Только авторизованный пользователь может сжигать токены
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );

//         // Сжигаем токены
//         let cpi_accounts = Burn {
//             mint: ctx.accounts.mint.to_account_info(),
//             from: ctx.accounts.token_account.to_account_info(),
//             authority: ctx.accounts.authority.to_account_info(),
//         };
        
//         let cpi_program = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
//         token::burn(cpi_ctx, amount)?;
        
//         msg!("Токены успешно сожжены, количество: {}", amount);
//         Ok(())
//     }

//     /// Заморозка аккаунта (административная функция)
//     pub fn freeze_account(ctx: Context<AdminFunctionWithMultisig>) -> Result<()> {
//         let admin_account = &ctx.accounts.admin_account;
        
//         // Проверка основного авторизованного пользователя
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );
        
//         // Проверка минимального количества подписей
//         let mut valid_signatures = 1; // Основной авторизованный пользователь уже подписал
        
//         // Проверяем дополнительных подписантов
//         if let Some(signer1) = &ctx.accounts.additional_signer1 {
//             if admin_account.authorized_signers.iter().any(|s| s.is_some() && s.unwrap() == signer1.key()) {
//                 valid_signatures += 1;
//             }
//         }
        
//         if let Some(signer2) = &ctx.accounts.additional_signer2 {
//             if admin_account.authorized_signers.iter().any(|s| s.is_some() && s.unwrap() == signer2.key()) {
//                 valid_signatures += 1;
//             }
//         }
        
//         // Убеждаемся, что есть достаточное количество подписей
//         require!(
//             valid_signatures >= admin_account.min_required_signers as usize,
//             NDollarError::InsufficientSigners
//         );

//         // Замораживаем аккаунт
//         let cpi_accounts = FreezeAccount {
//             account: ctx.accounts.token_account.to_account_info(),
//             mint: ctx.accounts.mint.to_account_info(),
//             authority: ctx.accounts.authority.to_account_info(),
//         };
        
//         let cpi_program = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
//         token::freeze_account(cpi_ctx)?;
        
//         msg!("Аккаунт заморожен успешно");
//         Ok(())
//     }

//     /// Разморозка аккаунта (административная функция)
//     pub fn thaw_account(ctx: Context<AdminFunctionWithMultisig>) -> Result<()> {
//         let admin_account = &ctx.accounts.admin_account;
        
//         // Проверка основного авторизованного пользователя
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );
        
//         // Проверка минимального количества подписей
//         let mut valid_signatures = 1; // Основной авторизованный пользователь уже подписал
        
//         // Проверяем дополнительных подписантов
//         if let Some(signer1) = &ctx.accounts.additional_signer1 {
//             if admin_account.authorized_signers.iter().any(|s| s.is_some() && s.unwrap() == signer1.key()) {
//                 valid_signatures += 1;
//             }
//         }
        
//         if let Some(signer2) = &ctx.accounts.additional_signer2 {
//             if admin_account.authorized_signers.iter().any(|s| s.is_some() && s.unwrap() == signer2.key()) {
//                 valid_signatures += 1;
//             }
//         }
        
//         // Убеждаемся, что есть достаточное количество подписей
//         require!(
//             valid_signatures >= admin_account.min_required_signers as usize,
//             NDollarError::InsufficientSigners
//         );

//         // Размораживаем аккаунт
//         let cpi_accounts = ThawAccount {
//             account: ctx.accounts.token_account.to_account_info(),
//             mint: ctx.accounts.mint.to_account_info(),
//             authority: ctx.accounts.authority.to_account_info(),
//         };
        
//         let cpi_program = ctx.accounts.token_program.to_account_info();
//         let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
//         token::thaw_account(cpi_ctx)?;
        
//         msg!("Аккаунт разморожен успешно");
//         Ok(())
//     }

//     /// Обновление метаданных токена (административная функция)
//     pub fn update_metadata(
//         ctx: Context<UpdateMetadata>,
//         name: String,
//         symbol: String,
//         uri: String,
//     ) -> Result<()> {
//         let admin_account = &ctx.accounts.admin_account;
        
//         // Только авторизованный пользователь может обновлять метаданные
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );

//         // Обновление метаданных через CPI к токен-метадата программе
//         let seeds = &[
//             b"admin_account".as_ref(),
//             &admin_account.mint.to_bytes(),
//             &[admin_account.bump],
//         ];
//         let signer = &[&seeds[..]];

//         // Создаем дискриминатор инструкции update_metadata_accounts_v2
//         let disc = anchor_lang::solana_program::hash::hash("global:update_metadata_accounts_v2".as_bytes());
//         let update_metadata_discriminator = disc.to_bytes()[..8].to_vec();
        
//         // Подготавливаем данные для инструкции
//         let mut ix_data = update_metadata_discriminator;
        
//         // Добавляем данные в формате, который ожидает метаплекс:
//         // Option<Data> - Some(Data) с полями name, symbol, uri
//         ix_data.push(1); // Some(data)
        
//         // Сериализуем название
//         let name_bytes = name.as_bytes();
//         ix_data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
//         ix_data.extend_from_slice(name_bytes);
        
//         // Сериализуем символ
//         let symbol_bytes = symbol.as_bytes();
//         ix_data.extend_from_slice(&(symbol_bytes.len() as u32).to_le_bytes());
//         ix_data.extend_from_slice(symbol_bytes);
        
//         // Сериализуем URI
//         let uri_bytes = uri.as_bytes();
//         ix_data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
//         ix_data.extend_from_slice(uri_bytes);
        
//         // Seller fee basis points (0)
//         ix_data.extend_from_slice(&0u16.to_le_bytes());
        
//         // Creators - None
//         ix_data.push(0);
        
//         // Collection - None
//         ix_data.push(0);
        
//         // Uses - None
//         ix_data.push(0);
        
//         // Option<bool> для поля primary_sale_happened - None
//         ix_data.push(0);
        
//         // Option<bool> для поля is_mutable - None
//         ix_data.push(0);
        
//         // Определяем аккаунты для инструкции
//         let ix_accounts = vec![
//             AccountMeta::new(ctx.accounts.metadata.key(), false),
//             AccountMeta::new_readonly(admin_account.to_account_info().key(), true),
//         ];
        
//         // Создаем инструкцию
//         let ix = anchor_lang::solana_program::instruction::Instruction {
//             program_id: ctx.accounts.metadata_program.key(),
//             accounts: ix_accounts,
//             data: ix_data,
//         };
        
//         // Выполняем инструкцию с PDA подписью
//         anchor_lang::solana_program::program::invoke_signed(
//             &ix,
//             &[
//                 ctx.accounts.metadata.to_account_info(),
//                 admin_account.to_account_info(),
//             ],
//             signer,
//         )?;
        
//         msg!("Метаданные обновлены успешно");
//         Ok(())
//     }

//     /// Добавление авторизованного подписанта
//     pub fn add_authorized_signer(ctx: Context<AdminFunction>, new_signer: Pubkey) -> Result<()> {
//         let admin_account = &mut ctx.accounts.admin_account;
        
//         // Только основной авторизованный пользователь может добавлять новые ключи
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );
        
//         // Находим пустой слот для нового подписанта
//         let mut added = false;
//         for signer_slot in admin_account.authorized_signers.iter_mut() {
//             if signer_slot.is_none() {
//                 *signer_slot = Some(new_signer);
//                 added = true;
//                 break;
//             }
//         }
        
//         require!(added, NDollarError::UnauthorizedAccess); // Если нет места для нового подписанта
        
//         msg!("Авторизованный подписант добавлен");
//         Ok(())
//     }

//     /// Удаление авторизованного подписанта
//     pub fn remove_authorized_signer(ctx: Context<AdminFunction>, signer_to_remove: Pubkey) -> Result<()> {
//         let admin_account = &mut ctx.accounts.admin_account;
        
//         // Только основной авторизованный пользователь может удалять ключи
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );
        
//         // Находим и удаляем подписанта
//         let mut removed = false;
//         for signer_slot in admin_account.authorized_signers.iter_mut() {
//             if let Some(key) = signer_slot {
//                 if *key == signer_to_remove {
//                     *signer_slot = None;
//                     removed = true;
//                     break;
//                 }
//             }
//         }
        
//         require!(removed, NDollarError::UnauthorizedAccess); // Если подписант не найден
        
//         msg!("Авторизованный подписант удален");
//         Ok(())
//     }

//     /// Установка минимального количества подписантов
//     pub fn set_min_required_signers(ctx: Context<AdminFunction>, min_signers: u8) -> Result<()> {
//         let admin_account = &mut ctx.accounts.admin_account;
        
//         // Только основной авторизованный пользователь может менять настройки
//         require!(
//             admin_account.authority == ctx.accounts.authority.key(),
//             NDollarError::UnauthorizedAccess
//         );
        
//         // Проверяем, что требуемое количество не превышает максимально возможное
//         require!(min_signers <= 3, NDollarError::UnauthorizedAccess);
        
//         admin_account.min_required_signers = min_signers;
        
//         msg!("Установлено минимальное количество подписантов: {}", min_signers);
//         Ok(())
//     }
// }

// #[derive(Accounts)]
// pub struct InitializeNDollar<'info> {
//     #[account(mut)]
//     pub authority: Signer<'info>,
    
//     #[account(
//         init,
//         payer = authority,
//         mint::decimals = 9,
//         mint::authority = admin_account,
//     )]
//     pub mint: Account<'info, Mint>,
    
//     /// CHECK: Аккаунт метаданных, который будет инициализирован через CPI
//     #[account(mut)]
//     pub metadata: AccountInfo<'info>,
    
//     #[account(
//         init,
//         payer = authority,
//         seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
//         bump,
//         space = 8 + AdminAccount::SPACE
//     )]
//     pub admin_account: Account<'info, AdminAccount>,
    
//     /// Опциональный admin_config аккаунт из программы admin_control
//     /// Используется для регистрации минта N-Dollar в admin_control
//     /// CHECK: Этот аккаунт проверяется внутри CPI вызова
//     #[account(mut)]
//     pub admin_config: Option<AccountInfo<'info>>,
    
//     /// Опциональная программа admin_control для CPI вызовов
//     /// CHECK: ID программы admin_control
//     pub admin_control_program: Option<AccountInfo<'info>>,
    
//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     /// CHECK: Метаплекс программа метаданных
//     pub metadata_program: Program<'info, Metadata>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct MintSupply<'info> {
//     #[account(mut)]
//     pub authority: Signer<'info>,
    
//     #[account(
//         mut,
//         constraint = mint.key() == admin_account.mint
//     )]
//     pub mint: Account<'info, Mint>,
    
//     #[account(
//         mut,
//         seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
//         bump = admin_account.bump
//     )]
//     pub admin_account: Account<'info, AdminAccount>,
    
//     #[account(
//         mut,
//         constraint = token_account.mint == mint.key()
//     )]
//     pub token_account: Account<'info, TokenAccount>,
    
//     /// Опциональный admin_config аккаунт из программы admin_control для проверки авторизации
//     /// CHECK: Этот аккаунт проверяется внутри CPI вызова
//     pub admin_config: Option<AccountInfo<'info>>,
    
//     /// Опциональная программа admin_control для CPI вызовов
//     /// CHECK: ID программы admin_control
//     pub admin_control_program: Option<AccountInfo<'info>>,
    
//     pub token_program: Program<'info, Token>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// pub struct AdminFunction<'info> {
//     #[account(mut)]
//     pub authority: Signer<'info>,
    
//     #[account(
//         mut,
//         constraint = mint.key() == admin_account.mint
//     )]
//     pub mint: Account<'info, Mint>,
    
//     #[account(
//         seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
//         bump = admin_account.bump,
//         constraint = admin_account.authority == authority.key()
//     )]
//     pub admin_account: Account<'info, AdminAccount>,
    
//     #[account(
//         mut,
//         constraint = token_account.mint == mint.key()
//     )]
//     pub token_account: Account<'info, TokenAccount>,
    
//     pub token_program: Program<'info, Token>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// pub struct UpdateMetadata<'info> {
//     #[account(mut)]
//     pub authority: Signer<'info>,
    
//     #[account(
//         constraint = mint.key() == admin_account.mint
//     )]
//     pub mint: Account<'info, Mint>,
    
//     #[account(
//         seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
//         bump = admin_account.bump,
//         constraint = admin_account.authority == authority.key()
//     )]
//     pub admin_account: Account<'info, AdminAccount>,
    
//     /// CHECK: Аккаунт метаданных, который будет обновлен через CPI
//     #[account(mut)]
//     pub metadata: AccountInfo<'info>,
    
//     /// CHECK: Метаплекс программа метаданных
//     pub metadata_program: Program<'info, Metadata>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// pub struct AdminFunctionWithMultisig<'info> {
//     #[account(mut)]
//     pub authority: Signer<'info>,
    
//     #[account(
//         mut,
//         constraint = mint.key() == admin_account.mint
//     )]
//     pub mint: Account<'info, Mint>,
    
//     #[account(
//         seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
//         bump = admin_account.bump,
//         constraint = admin_account.authority == authority.key()
//     )]
//     pub admin_account: Account<'info, AdminAccount>,
    
//     #[account(
//         mut,
//         constraint = token_account.mint == mint.key()
//     )]
//     pub token_account: Account<'info, TokenAccount>,
    
//     // Дополнительные подписанты (опциональные)
//     pub additional_signer1: Option<Signer<'info>>,
//     pub additional_signer2: Option<Signer<'info>>,
    
//     pub token_program: Program<'info, Token>,
//     pub system_program: Program<'info, System>,
// }

// #[account]
// pub struct AdminAccount {
//     pub authority: Pubkey,
//     pub mint: Pubkey,
//     pub last_mint_time: i64,
//     pub total_supply: u64,
//     pub bump: u8,
//     // Добавляем поле для хранения дополнительных авторизованных ключей
//     pub authorized_signers: [Option<Pubkey>; 3],  // Массив дополнительных подписантов
//     // Добавляем поле для защиты от атак на время
//     pub last_block_time: i64,  // Последний блок, когда был выполнен минт
//     pub last_block_height: u64, // Высота последнего блока
//     pub min_required_signers: u8, // Минимальное количество подписантов для чувствительных операций
// }

// impl AdminAccount {
//     pub const SPACE: usize = 32 + 32 + 8 + 8 + 1 + ((32 + 1) * 3) + 8 + 8 + 1;
// }

// #[error_code]
// pub enum NDollarError {
//     #[msg("Несанкционированный доступ")]
//     UnauthorizedAccess,
//     #[msg("Слишком рано для минтинга, должна пройти неделя между минтами")]
//     TooEarlyToMint,
//     #[msg("Обнаружена атака на время, несоответствие в данных блока")]
//     TimeManipulationDetected,
//     #[msg("Недостаточное количество подтверждений для критической операции")]
//     InsufficientSigners,
// }
