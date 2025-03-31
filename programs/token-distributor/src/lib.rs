// token_distributor.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

// !!! УКАЖИТЕ ЗДЕСЬ РЕАЛЬНЫЙ ID ВАШЕЙ ПРОГРАММЫ РАСПРЕДЕЛИТЕЛЯ !!!
declare_id!("2Hy1wGdC5iqceaTnZC1qJeuoM4s6yEKHbYcjMMpbKYqp");

#[program]
pub mod token_distributor {
    use super::*;

    // total_supply больше не нужен как аргумент инструкции,
    // так как мы можем взять его из distributor_token_account.amount
    pub fn distribute_tokens(ctx: Context<DistributeTokens>) -> Result<()> {
        msg!("Distributing tokens...");

        // Получаем total_supply из баланса аккаунта дистрибьютора
        // Перезагружаем аккаунт на всякий случай, чтобы получить свежий баланс
        ctx.accounts.distributor_token_account.reload()?;
        let total_supply = ctx.accounts.distributor_token_account.amount;

        require!(total_supply > 0, ErrorCode::ZeroSupply);
        msg!("Total supply to distribute: {}", total_supply);


        // ... (Расчеты bonding_curve_amount и user_amount как раньше) ...
        let total_supply_u128 = total_supply as u128;
        let bonding_curve_amount_u128 = total_supply_u128
            .checked_mul(30)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::CalculationOverflow)?;
        let bonding_curve_amount = bonding_curve_amount_u128 as u64;
        let user_amount = total_supply
            .checked_sub(bonding_curve_amount)
            .ok_or(ErrorCode::CalculationOverflow)?;

        msg!(
            "Calculated distribution - Bonding Curve: {}, User: {}",
            bonding_curve_amount,
            user_amount
        );


        // Проверка баланса больше не нужна здесь, так как мы его только что прочитали
        // require!(
        //     ctx.accounts.distributor_token_account.amount == total_supply,
        //     ErrorCode::InsufficientDistributorBalance // Можно переименовать ошибку или убрать проверку
        // );

        // Находим PDA и bump
        let (_distributor_pda, distributor_bump) = Pubkey::find_program_address(
            &[b"distributor".as_ref(), ctx.accounts.mint.key().as_ref()],
            ctx.program_id,
        );
        let mint_key = ctx.accounts.mint.key();
        let seeds = &[b"distributor".as_ref(), mint_key.as_ref(), &[distributor_bump]];
        let signer_seeds = &[&seeds[..]];

        // Перевод 30% в Bonding Curve Account
        if bonding_curve_amount > 0 {
             msg!("Transferring {} tokens to bonding curve account {}", bonding_curve_amount, ctx.accounts.bonding_curve_token_account.key());
             let cpi_accounts_bc = Transfer {
                 from: ctx.accounts.distributor_token_account.to_account_info(),
                 to: ctx.accounts.bonding_curve_token_account.to_account_info(),
                 authority: ctx.accounts.distributor_authority.to_account_info(),
             };
             let cpi_program_bc = ctx.accounts.token_program.to_account_info();
             let cpi_ctx_bc = CpiContext::new_with_signer(cpi_program_bc, cpi_accounts_bc, signer_seeds);
             token::transfer(cpi_ctx_bc, bonding_curve_amount)?;
        } else {
            msg!("Skipping transfer to bonding curve (amount is zero)");
        }

        // Перевод 70% (остаток) в User Account
        if user_amount > 0 {
            msg!("Transferring {} tokens to user account {}", user_amount, ctx.accounts.user_token_account.key());
            let cpi_accounts_user = Transfer {
                from: ctx.accounts.distributor_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.distributor_authority.to_account_info(),
            };
            let cpi_program_user = ctx.accounts.token_program.to_account_info();
            let cpi_ctx_user = CpiContext::new_with_signer(cpi_program_user, cpi_accounts_user, signer_seeds);
            token::transfer(cpi_ctx_user, user_amount)?;
        } else {
            msg!("Skipping transfer to user (amount is zero)");
        }

        // Optional: Close the distributor token account
        // ... (логика закрытия аккаунта, если нужна) ...
        // Убедитесь, что `destination` для ренты указан правильно (например, user_authority)

        msg!("Token distribution complete.");
        Ok(())
    }
}

#[derive(Accounts)]
// Убираем total_supply из instruction, так как берем его из аккаунта
pub struct DistributeTokens<'info> {
    pub mint: Account<'info, Mint>, // Нужен для PDA seeds

    /// CHECK: PDA derivation checked below. Authority for distributor_token_account.
    #[account(
        seeds = [b"distributor".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub distributor_authority: AccountInfo<'info>,

    #[account(
        mut, // Будет уменьшаться баланс
        associated_token::mint = mint,
        associated_token::authority = distributor_authority, // PDA owns this account
    )]
    pub distributor_token_account: Account<'info, TokenAccount>,

    // User Authority теперь должен быть Signer, т.к. он платит за ATA
    #[account(mut)] // Получает ренту при закрытии distributor_token_account
    pub user_authority: Signer<'info>,

    #[account(
        init_if_needed, // Создаем ATA пользователя, если его нет
        payer = user_authority, // Пользователь платит за свой ATA
        associated_token::mint = mint,
        associated_token::authority = user_authority, // User owns this account
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA derivation checked below (using bonding_curve module logic).
    #[account(
        seeds = [b"bonding_curve".as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = bonding_curve::ID // Убедитесь, что ID правильный
    )]
    pub bonding_curve_authority: AccountInfo<'info>,

    #[account(
        init_if_needed, // Создаем ATA баиндинг кривой, если его нет
        payer = user_authority, // Пользователь платит за этот ATA тоже? Или нужен другой плательщик?
        associated_token::mint = mint,
        associated_token::authority = bonding_curve_authority, // Bonding curve PDA owns this
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,

    // --- Необходимые программы ---
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>, // Нужен для init_if_needed
    pub associated_token_program: Program<'info, AssociatedToken>, // Нужен для init_if_needed
}

#[error_code]
pub enum ErrorCode {
    #[msg("Calculation overflow")]
    CalculationOverflow,
    #[msg("Total supply cannot be zero")]
    ZeroSupply,
    // Можно убрать или переименовать, если проверка не нужна
    // #[msg("Distributor token account has insufficient balance")]
    // InsufficientDistributorBalance,
}
