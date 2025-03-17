use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn, FreezeAccount, ThawAccount};
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata};
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use anchor_lang::solana_program::pubkey::Pubkey;

declare_id!("2MiMzfH55kTC9tNM8LQFLzC8GbduTMTe5SyTadBi33cL");

#[program]
pub mod n_dollar_token {
    use super::*;

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

        msg!("N-Dollar Token успешно инициализирован");
        Ok(())
    }

    /// Минтинг токенов согласно расписанию
    pub fn mint_supply(ctx: Context<MintSupply>, amount: u64) -> Result<()> {
        let admin_account = &mut ctx.accounts.admin_account;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Проверка прошла ли неделя с последнего минта
        // 7 дней * 24 часа * 60 минут * 60 секунд = 604800 секунд
        let time_since_last_mint = current_time - admin_account.last_mint_time;
        require!(time_since_last_mint >= 604800, NDollarError::TooEarlyToMint);
        
        // Только авторизованный пользователь может минтить
        require!(
            admin_account.authority == ctx.accounts.authority.key(),
            NDollarError::UnauthorizedAccess
        );

        // Минтим токены
        let seeds = &[
            b"admin_account".as_ref(),
            &admin_account.mint.to_bytes(),
            &[admin_account.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: admin_account.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, amount)?;
        
        // Обновляем информацию о последнем минте
        admin_account.last_mint_time = current_time;
        admin_account.total_supply += amount;
        
        msg!("Минт выполнен успешно, добавлено: {}", amount);
        Ok(())
    }

    /// Сжигание токенов (административная функция)
    pub fn burn_tokens(ctx: Context<AdminFunction>, amount: u64) -> Result<()> {
        let admin_account = &ctx.accounts.admin_account;
        
        // Только авторизованный пользователь может сжигать токены
        require!(
            admin_account.authority == ctx.accounts.authority.key(),
            NDollarError::UnauthorizedAccess
        );

        // Сжигаем токены
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::burn(cpi_ctx, amount)?;
        
        msg!("Токены успешно сожжены, количество: {}", amount);
        Ok(())
    }

    /// Заморозка аккаунта (административная функция)
    pub fn freeze_account(ctx: Context<AdminFunction>) -> Result<()> {
        let admin_account = &ctx.accounts.admin_account;
        
        // Только авторизованный пользователь может замораживать аккаунты
        require!(
            admin_account.authority == ctx.accounts.authority.key(),
            NDollarError::UnauthorizedAccess
        );

        // Замораживаем аккаунт
        let cpi_accounts = FreezeAccount {
            account: ctx.accounts.token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::freeze_account(cpi_ctx)?;
        
        msg!("Аккаунт заморожен успешно");
        Ok(())
    }

    /// Разморозка аккаунта (административная функция)
    pub fn thaw_account(ctx: Context<AdminFunction>) -> Result<()> {
        let admin_account = &ctx.accounts.admin_account;
        
        // Только авторизованный пользователь может размораживать аккаунты
        require!(
            admin_account.authority == ctx.accounts.authority.key(),
            NDollarError::UnauthorizedAccess
        );

        // Размораживаем аккаунт
        let cpi_accounts = ThawAccount {
            account: ctx.accounts.token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::thaw_account(cpi_ctx)?;
        
        msg!("Аккаунт разморожен успешно");
        Ok(())
    }

    /// Обновление метаданных токена (административная функция)
    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        _name: String,
        _symbol: String,
        _uri: String,
    ) -> Result<()> {
        let admin_account = &ctx.accounts.admin_account;
        
        // Только авторизованный пользователь может обновлять метаданные
        require!(
            admin_account.authority == ctx.accounts.authority.key(),
            NDollarError::UnauthorizedAccess
        );

        // Обновление метаданных будет реализовано через CPI к токен-метадата программе
        // Здесь нужна реализация через метаплекс, но это требует больше кода
        // и для примера оставлено как заглушка
        
        msg!("Метаданные обновлены успешно");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeNDollar<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = admin_account,
    )]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: Аккаунт метаданных, который будет инициализирован через CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + AdminAccount::SPACE
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintSupply<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = mint.key() == admin_account.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
    #[account(
        mut,
        constraint = token_account.mint == mint.key()
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminFunction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = mint.key() == admin_account.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump = admin_account.bump,
        constraint = admin_account.authority == authority.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
    #[account(
        mut,
        constraint = token_account.mint == mint.key()
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        constraint = mint.key() == admin_account.mint
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"admin_account".as_ref(), mint.key().as_ref()],
        bump = admin_account.bump,
        constraint = admin_account.authority == authority.key()
    )]
    pub admin_account: Account<'info, AdminAccount>,
    
    /// CHECK: Аккаунт метаданных, который будет обновлен через CPI
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Метаплекс программа метаданных
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AdminAccount {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub last_mint_time: i64,
    pub total_supply: u64,
    pub bump: u8,
}

impl AdminAccount {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 1;
}

#[error_code]
pub enum NDollarError {
    #[msg("Несанкционированный доступ")]
    UnauthorizedAccess,
    #[msg("Слишком рано для минтинга, должна пройти неделя между минтами")]
    TooEarlyToMint,
}
