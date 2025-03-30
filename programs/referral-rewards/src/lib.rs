// referral_rewards.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer, CloseAccount},
};

// !!! ЗАМЕНИТЕ НА ВАШ УНИКАЛЬНЫЙ PROGRAM ID ДЛЯ РЕФЕРАЛЬНОЙ ПРОГРАММЫ !!!
declare_id!("8YxBBLHPCAMQWkxD8WD1X1XyuUZH5u58wB8R4Q31eGCb"); // Пример ID, замените!

const ADMIN_CONFIG_SEED: &[u8] = b"admin_config";
const REWARD_POOL_SEED: &[u8] = b"reward_pool";

#[program]
pub mod referral_rewards {
    use super::*;

    // Инициализация конфигурации администратора и пула наград
    pub fn initialize(ctx: Context<Initialize>, admin_key: Pubkey, reward_amount: u64) -> Result<()> {
        msg!("Initializing Referral Rewards Program...");

        let admin_config = &mut ctx.accounts.admin_config;
        admin_config.admin_key = admin_key;
        admin_config.reward_token_mint = ctx.accounts.reward_token_mint.key();
        admin_config.reward_amount = reward_amount; // Указывайте с учетом decimals токена!
        admin_config.is_active = true; // Программа активна
        admin_config.bump_config = ctx.bumps.admin_config;
        admin_config.bump_pool_auth = ctx.bumps.reward_pool_authority;

        msg!("Admin Key: {}", admin_key);
        msg!("Reward Mint: {}", admin_config.reward_token_mint);
        msg!("Reward Amount (raw): {}", reward_amount);
        msg!("Referral program initialized and active.");
        Ok(())
    }

    // Обновление администратора (только текущий админ)
    pub fn update_admin(ctx: Context<UpdateAdmin>, new_admin_key: Pubkey) -> Result<()> {
        msg!("Updating admin key...");
        require!(ctx.accounts.admin_config.is_active, ErrorCode::ProgramInactive);
        // Проверка не нужна, т.к. admin_signer == admin_config.admin_key проверяется в #[account]
        ctx.accounts.admin_config.admin_key = new_admin_key;
        msg!("Admin key updated to: {}", new_admin_key);
        Ok(())
    }

     // Обновление размера награды (только админ)
    pub fn update_reward_amount(ctx: Context<UpdateAdmin>, new_reward_amount: u64) -> Result<()> {
        msg!("Updating reward amount...");
        require!(ctx.accounts.admin_config.is_active, ErrorCode::ProgramInactive);
        // Проверка не нужна, т.к. admin_signer == admin_config.admin_key проверяется в #[account]
        ctx.accounts.admin_config.reward_amount = new_reward_amount; // Указывайте с учетом decimals
        msg!("Reward amount updated to (raw): {}", new_reward_amount);
        Ok(())
    }

    // Включение/Отключение программы (только админ)
     pub fn set_active_status(ctx: Context<UpdateAdmin>, is_active: bool) -> Result<()> {
        msg!("Setting active status to: {}", is_active);
         // Проверка не нужна, т.к. admin_signer == admin_config.admin_key проверяется в #[account]
        ctx.accounts.admin_config.is_active = is_active;
        msg!("Program active status updated.");
        Ok(())
    }


    // Распределение награды рефереру и рефералу (вызывается бэкендом)
    pub fn distribute_referral_reward(ctx: Context<DistributeReferralReward>) -> Result<()> {
        msg!("Distributing referral reward...");
        let config = &ctx.accounts.admin_config;

        // 1. Проверка активности программы и авторизации бэкенда
        require!(config.is_active, ErrorCode::ProgramInactive);
        // Авторизация проверяется в constraint 'has_one = admin_key @ ErrorCode::Unauthorized'

        // 2. Проверка корректности аккаунтов
        require!(ctx.accounts.referrer_token_account.mint == config.reward_token_mint, ErrorCode::InvalidMint);
        require!(ctx.accounts.referee_token_account.mint == config.reward_token_mint, ErrorCode::InvalidMint);
        require!(ctx.accounts.referrer_token_account.owner != ctx.accounts.referee_token_account.owner, ErrorCode::SameReferrerReferee);

        // 3. Проверка баланса пула наград
        let reward_amount = config.reward_amount;
        let total_reward_needed = reward_amount.checked_mul(2).ok_or(ErrorCode::CalculationOverflow)?;
        require!(ctx.accounts.reward_pool_token_account.amount >= total_reward_needed, ErrorCode::InsufficientPoolBalance);

        msg!(
            "Reward amount per user (raw): {}. Total needed: {}",
            reward_amount,
            total_reward_needed
        );

        // 4. Подготовка PDA signer seeds для пула наград
        let seeds = &[
            REWARD_POOL_SEED.as_ref(),
            &[config.bump_pool_auth] // Используем сохраненный bump
        ];
        let signer_seeds = &[&seeds[..]];

        // 5. Перевод награды Рефереру
        msg!("Transferring {} to referrer {}", reward_amount, ctx.accounts.referrer_token_account.key());
        let cpi_accounts_referrer = Transfer {
            from: ctx.accounts.reward_pool_token_account.to_account_info(),
            to: ctx.accounts.referrer_token_account.to_account_info(),
            authority: ctx.accounts.reward_pool_authority.to_account_info(),
        };
        let cpi_program_referrer = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_referrer = CpiContext::new_with_signer(cpi_program_referrer, cpi_accounts_referrer, signer_seeds);
        token::transfer(cpi_ctx_referrer, reward_amount)?;
        msg!("Transfer to referrer successful.");

        // 6. Перевод награды Рефералу
        msg!("Transferring {} to referee {}", reward_amount, ctx.accounts.referee_token_account.key());
         let cpi_accounts_referee = Transfer {
            from: ctx.accounts.reward_pool_token_account.to_account_info(), // Снова из пула
            to: ctx.accounts.referee_token_account.to_account_info(),
            authority: ctx.accounts.reward_pool_authority.to_account_info(), // Тот же PDA авторизует
        };
        let cpi_program_referee = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_referee = CpiContext::new_with_signer(cpi_program_referee, cpi_accounts_referee, signer_seeds);
        token::transfer(cpi_ctx_referee, reward_amount)?;
        msg!("Transfer to referee successful.");

        msg!("Referral reward distribution complete.");
        Ok(())
    }

    // Функция для пополнения пула наград (вызывается админом/владельцем)
    pub fn deposit_rewards(ctx: Context<DepositRewards>, amount: u64) -> Result<()> {
        msg!("Depositing {} tokens into reward pool...", amount);
        require!(ctx.accounts.admin_config.is_active, ErrorCode::ProgramInactive);
        require!(amount > 0, ErrorCode::ZeroAmount);

        // Проверка, что токен соответствует
         require!(ctx.accounts.source_token_account.mint == ctx.accounts.admin_config.reward_token_mint, ErrorCode::InvalidMint);
         require!(ctx.accounts.reward_pool_token_account.mint == ctx.accounts.admin_config.reward_token_mint, ErrorCode::InvalidMint);


        let cpi_accounts = Transfer {
            from: ctx.accounts.source_token_account.to_account_info(),
            to: ctx.accounts.reward_pool_token_account.to_account_info(),
            authority: ctx.accounts.depositor_authority.to_account_info(), // Подпись того, кто пополняет
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        msg!("Successfully deposited {} tokens.", amount);
        Ok(())
    }

     // Функция для вывода средств из пула (только админ, для безопасности/обновления)
    pub fn withdraw_rewards(ctx: Context<WithdrawRewards>, amount: u64) -> Result<()> {
        msg!("Withdrawing {} tokens from reward pool...", amount);
        // Админ проверяется через constraint has_one
        require!(amount > 0, ErrorCode::ZeroAmount);
        require!(ctx.accounts.reward_pool_token_account.amount >= amount, ErrorCode::InsufficientPoolBalance);

        // Подготовка PDA signer seeds для пула наград
        let config = &ctx.accounts.admin_config;
        let seeds = &[
            REWARD_POOL_SEED.as_ref(),
            &[config.bump_pool_auth]
        ];
        let signer_seeds = &[&seeds[..]];

         let cpi_accounts = Transfer {
            from: ctx.accounts.reward_pool_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.reward_pool_authority.to_account_info(), // PDA авторизует
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, amount)?;

        msg!("Successfully withdrew {} tokens to {}", amount, ctx.accounts.destination_token_account.key());
        Ok(())
    }

     // Функция для закрытия пула и возврата ренты (только админ)
    // ОСТОРОЖНО: Использовать только если пул больше не нужен
    pub fn close_pool(ctx: Context<ClosePool>) -> Result<()> {
        msg!("Closing reward pool...");
        // Админ проверяется через constraint has_one

        let config = &ctx.accounts.admin_config;
        let seeds = &[
            REWARD_POOL_SEED.as_ref(),
            &[config.bump_pool_auth]
        ];
        let signer_seeds = &[&seeds[..]];

        // Закрываем ATA пула, ренту возвращаем админу
        let ca_accounts = CloseAccount {
            account: ctx.accounts.reward_pool_token_account.to_account_info(),
            destination: ctx.accounts.admin_key.to_account_info(), // Возвращаем ренту админу
            authority: ctx.accounts.reward_pool_authority.to_account_info(), // PDA авторизует
        };
        let ca_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            ca_accounts,
            signer_seeds
        );
        token::close_account(ca_ctx)?;
        msg!("Reward pool token account closed.");

        // Можно также закрыть admin_config, если нужно, но обычно его оставляют
        // let admin_config_account = ctx.accounts.admin_config.to_account_info();
        // admin_config_account.close(ctx.accounts.admin_key.to_account_info())?;
        // msg!("Admin config account closed.");


        Ok(())
    }

}

// --- Accounts Structs ---

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = AdminConfig::LEN,
        seeds = [ADMIN_CONFIG_SEED], // Фиксированный сид для единственного конфига
        bump
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// PDA, который будет владеть пулом токенов для наград
    #[account(
        seeds = [REWARD_POOL_SEED], // Фиксированный сид для единственного PDA
        bump
    )]
    /// CHECK: This is the authority PDA for the reward pool ATA. Its derivation is checked by seeds.
    pub reward_pool_authority: AccountInfo<'info>,

    /// Mint токена, который используется для наград
    pub reward_token_mint: Account<'info, Mint>,

    /// ATA для хранения пула наград, создается и принадлежит PDA `reward_pool_authority`
    #[account(
        init,
        payer = payer,
        associated_token::mint = reward_token_mint,
        associated_token::authority = reward_pool_authority // PDA будет владельцем
    )]
    pub reward_pool_token_account: Account<'info, TokenAccount>,

    /// Аккаунт, который платит за создание аккаунтов (обычно деплоер/инициатор)
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>, // Needed for init
}

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED],
        bump = admin_config.bump_config,
        has_one = admin_key @ ErrorCode::Unauthorized // Только текущий админ может вызывать
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Текущий администратор, подписывающий транзакцию
    pub admin_key: Signer<'info>,
}


#[derive(Accounts)]
pub struct DistributeReferralReward<'info> {
    #[account(
        seeds = [ADMIN_CONFIG_SEED],
        bump = admin_config.bump_config,
        // Проверяем, что вызывающий (admin_key) является админом И что минт награды совпадает
        has_one = admin_key @ ErrorCode::Unauthorized,
        has_one = reward_token_mint @ ErrorCode::InvalidMint,
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Ключ бэкенда/администратора, который вызывает эту функцию
    pub admin_key: Signer<'info>,

    /// Mint токена для награды (проверяется через has_one)
    pub reward_token_mint: Account<'info, Mint>,

     /// PDA авторитет пула наград
    #[account(
        seeds = [REWARD_POOL_SEED],
        bump = admin_config.bump_pool_auth
    )]
    /// CHECK: PDA derivation checked by seeds.
    pub reward_pool_authority: AccountInfo<'info>,

    /// ATA аккаунт с пулом наград
    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = reward_pool_authority
    )]
    pub reward_pool_token_account: Account<'info, TokenAccount>,

    /// ATA аккаунт реферера (получатель)
    #[account(
        mut,
        // Не проверяем authority здесь, т.к. это просто получатель
        // Проверка на mint делается в коде функции для большей ясности
         constraint = referrer_token_account.mint == reward_token_mint.key() @ ErrorCode::InvalidMint,
    )]
    pub referrer_token_account: Account<'info, TokenAccount>,

    /// ATA аккаунт реферала (получатель)
    #[account(
        mut,
        constraint = referee_token_account.mint == reward_token_mint.key() @ ErrorCode::InvalidMint,
        // Дополнительно убедимся, что это не один и тот же аккаунт (хотя проверка owner ниже надежнее)
        // constraint = referee_token_account.key() != referrer_token_account.key() @ ErrorCode::SameReferrerRefereeTokenAccount
    )]
    pub referee_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct DepositRewards<'info> {
     #[account(
        seeds = [ADMIN_CONFIG_SEED],
        bump = admin_config.bump_config,
        has_one = reward_token_mint @ ErrorCode::InvalidMint, // Проверяем минт
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Mint токена для награды
    pub reward_token_mint: Account<'info, Mint>,

    /// PDA авторитет пула наград
    #[account(
        seeds = [REWARD_POOL_SEED],
        bump = admin_config.bump_pool_auth
    )]
     /// CHECK: PDA derivation checked by seeds.
    pub reward_pool_authority: AccountInfo<'info>,

     /// ATA аккаунт с пулом наград (куда пополняем)
    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = reward_pool_authority
    )]
    pub reward_pool_token_account: Account<'info, TokenAccount>,

     /// ATA аккаунт, откуда берутся токены для пополнения
    #[account(mut,
        constraint = source_token_account.mint == reward_token_mint.key() @ ErrorCode::InvalidMint,
        constraint = source_token_account.owner == depositor_authority.key() @ ErrorCode::Unauthorized, // Убедимся, что аккаунт принадлежит пополняющему
    )]
    pub source_token_account: Account<'info, TokenAccount>,

    /// Аккаунт, который инициирует пополнение (должен владеть source_token_account)
    #[account(mut)]
    pub depositor_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawRewards<'info> {
    #[account(
        seeds = [ADMIN_CONFIG_SEED],
        bump = admin_config.bump_config,
        has_one = admin_key @ ErrorCode::Unauthorized, // Только админ может вывести
        has_one = reward_token_mint @ ErrorCode::InvalidMint,
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Подпись админа
    pub admin_key: Signer<'info>,

     /// Mint токена для награды
    pub reward_token_mint: Account<'info, Mint>,

     /// PDA авторитет пула наград
    #[account(
        seeds = [REWARD_POOL_SEED],
        bump = admin_config.bump_pool_auth
    )]
     /// CHECK: PDA derivation checked by seeds.
    pub reward_pool_authority: AccountInfo<'info>,

     /// ATA аккаунт с пулом наград (откуда выводим)
    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = reward_pool_authority
    )]
    pub reward_pool_token_account: Account<'info, TokenAccount>,

    /// ATA аккаунт, куда выводятся токены (должен принадлежать админу или кому он укажет)
    #[account(mut,
        constraint = destination_token_account.mint == reward_token_mint.key() @ ErrorCode::InvalidMint,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

     pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClosePool<'info> {
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED],
        bump = admin_config.bump_config,
        has_one = admin_key @ ErrorCode::Unauthorized, // Только админ
        close = admin_key // Ренту от admin_config вернем админу
    )]
    pub admin_config: Account<'info, AdminConfig>,

    /// Подпись админа (получит ренту)
    #[account(mut)]
    pub admin_key: Signer<'info>,

     /// PDA авторитет пула наград
    #[account(
        seeds = [REWARD_POOL_SEED],
        bump = admin_config.bump_pool_auth
    )]
     /// CHECK: PDA derivation checked by seeds.
    pub reward_pool_authority: AccountInfo<'info>,

     /// ATA аккаунт с пулом наград (закрывается)
    #[account(
        mut,
        associated_token::mint = admin_config.reward_token_mint, // Используем минт из конфига
        associated_token::authority = reward_pool_authority,
        // Не используем close = здесь, т.к. закрываем через CPI
    )]
    pub reward_pool_token_account: Account<'info, TokenAccount>,

     pub token_program: Program<'info, Token>,
     pub system_program: Program<'info, System>,
}


// --- State Account ---

#[account]
#[derive(Default)]
pub struct AdminConfig {
    pub admin_key: Pubkey,        // Ключ бэкенда/администратора
    pub reward_token_mint: Pubkey,// Mint токена для наград
    pub reward_amount: u64,       // Сумма награды (с учетом decimals!)
    pub is_active: bool,          // Активна ли программа рефералов
    pub bump_config: u8,          // Bump для PDA этого конфига
    pub bump_pool_auth: u8,       // Bump для PDA авторитета пула наград
}

impl AdminConfig {
    // Рассчитываем место под аккаунт
    // 32 (admin_key) + 32 (mint) + 8 (amount) + 1 (is_active) + 1 (bump_config) + 1 (bump_pool_auth) + 8 (discriminator)
    const LEN: usize = 32 + 32 + 8 + 1 + 1 + 1 + 8;
}


// --- Error Codes ---

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized: Only the admin can perform this action.")]
    Unauthorized,
    #[msg("Calculation overflow.")]
    CalculationOverflow,
    #[msg("Insufficient balance in the reward pool.")]
    InsufficientPoolBalance,
    #[msg("Referrer and referee cannot be the same person.")]
    SameReferrerReferee,
    // #[msg("Referrer and referee token accounts cannot be the same.")]
    // SameReferrerRefereeTokenAccount, // Обычно проверка owner достаточна
    #[msg("Invalid token mint used.")]
    InvalidMint,
    #[msg("The referral program is currently inactive.")]
    ProgramInactive,
    #[msg("Amount cannot be zero.")]
    ZeroAmount,
}