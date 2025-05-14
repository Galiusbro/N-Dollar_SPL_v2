// token_distributor.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

// ID этой программы (Token Distributor)
declare_id!("2Hy1wGdC5iqceaTnZC1qJeuoM4s6yEKHbYcjMMpbKYqp");

// Объявляем ID реферальной программы для использования в constraints
mod referral_program {
    use anchor_lang::declare_id;
    // !!! ID РЕФЕРАЛЬНОЙ ПРОГРАММЫ (УБЕДИТЕСЬ, ЧТО ОН ВЕРНЫЙ) !!!
    declare_id!("DMQh8Evpe3y4DzAWxx1rhLuGpnZGDvFSPLJvD9deQQfX");
}

#[program]
pub mod token_distributor {
    use super::*;

    pub fn distribute_tokens(ctx: Context<DistributeTokens>) -> Result<()> {
        msg!("Distributing tokens...");

        ctx.accounts.distributor_token_account.reload()?;
        let total_supply = ctx.accounts.distributor_token_account.amount;

        require!(total_supply > 0, ErrorCode::ZeroSupply);
        msg!("Total supply to distribute: {}", total_supply);

        // Расчеты: 10% казне реферальной программы, 40% кривой, 50% AI агенту
        let total_supply_u128 = total_supply as u128;

        let referral_treasury_amount_u128 = total_supply_u128
            .checked_mul(10) // 10% for referral treasury (viral distribution)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::CalculationOverflow)?;
        let referral_treasury_amount = referral_treasury_amount_u128 as u64;

        let bonding_curve_amount_u128 = total_supply_u128
            .checked_mul(40) // 40% for bonding curve
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::CalculationOverflow)?;
        let bonding_curve_amount = bonding_curve_amount_u128 as u64;

        // Оставшаяся часть (50%) идет AI агенту
        let ai_agent_amount = total_supply
            .checked_sub(referral_treasury_amount)
            .ok_or(ErrorCode::CalculationOverflow)?
            .checked_sub(bonding_curve_amount)
            .ok_or(ErrorCode::CalculationOverflow)?;

        msg!(
            "Calculated distribution - Referral Treasury: {}, Bonding Curve: {}, AI Agent: {}",
            referral_treasury_amount,
            bonding_curve_amount,
            ai_agent_amount
        );

        // Находим PDA и bump для distributor_authority (ЭТОЙ программы)
        let (_distributor_pda, distributor_bump) = Pubkey::find_program_address(
            &[b"distributor".as_ref(), ctx.accounts.mint.key().as_ref()],
            ctx.program_id, // ID ЭТОЙ программы
        );
        let mint_key = ctx.accounts.mint.key();
        let distributor_seeds = &[b"distributor".as_ref(), mint_key.as_ref(), &[distributor_bump]];
        let distributor_signer_seeds = &[&distributor_seeds[..]];

         // --- Переводы ---

        // Перевод 10% на аккаунт казны РЕФЕРАЛЬНОЙ ПРОГРАММЫ
        if referral_treasury_amount > 0 {
             msg!("Transferring {} tokens to REFERRAL treasury account {}", referral_treasury_amount, ctx.accounts.referral_treasury_token_account.key());
             let cpi_accounts_treasury = Transfer {
                 from: ctx.accounts.distributor_token_account.to_account_info(),
                 to: ctx.accounts.referral_treasury_token_account.to_account_info(), // <-- Направляем в казну рефералки
                 authority: ctx.accounts.distributor_authority.to_account_info(),      // <-- Подписывает PDA дистрибьютора
             };
             let cpi_program_treasury = ctx.accounts.token_program.to_account_info();
             token::transfer(
                 CpiContext::new_with_signer(cpi_program_treasury, cpi_accounts_treasury, distributor_signer_seeds),
                 referral_treasury_amount
             )?;
        } else {
            msg!("Skipping transfer to referral treasury (amount is zero)");
        }

        // Перевод 40% на аккаунт кривой
        if bonding_curve_amount > 0 {
             msg!("Transferring {} tokens to bonding curve account {}", bonding_curve_amount, ctx.accounts.bonding_curve_token_account.key());
             let cpi_accounts_bc = Transfer {
                 from: ctx.accounts.distributor_token_account.to_account_info(),
                 to: ctx.accounts.bonding_curve_token_account.to_account_info(),
                 authority: ctx.accounts.distributor_authority.to_account_info(), // distributor PDA
             };
             let cpi_program_bc = ctx.accounts.token_program.to_account_info();
             token::transfer(
                 CpiContext::new_with_signer(cpi_program_bc, cpi_accounts_bc, distributor_signer_seeds),
                 bonding_curve_amount
            )?;
        } else {
            msg!("Skipping transfer to bonding curve (amount is zero)");
        }

        // Перевод 50% (остаток) на аккаунт AI агента
        if ai_agent_amount > 0 {
             msg!("Transferring {} tokens to AI Agent account {}", ai_agent_amount, ctx.accounts.ai_agent_token_account.key());
             let cpi_accounts_ai = Transfer {
                 from: ctx.accounts.distributor_token_account.to_account_info(),
                 to: ctx.accounts.ai_agent_token_account.to_account_info(), // <-- Направляем AI агенту
                 authority: ctx.accounts.distributor_authority.to_account_info(),      // <-- Подписывает PDA дистрибьютора
             };
             let cpi_program_ai = ctx.accounts.token_program.to_account_info();
             token::transfer(
                 CpiContext::new_with_signer(cpi_program_ai, cpi_accounts_ai, distributor_signer_seeds),
                 ai_agent_amount
             )?;
        } else {
            msg!("Skipping transfer to AI Agent (amount is zero)");
        }

        msg!("Token distribution complete.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct DistributeTokens<'info> {
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA этой программы (distributor), авторитет для distributor_token_account.
    #[account(
        seeds = [b"distributor".as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub distributor_authority: AccountInfo<'info>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = distributor_authority)]
    pub distributor_token_account: Account<'info, TokenAccount>,

    /// The user who will own the user_token_account if created.
    pub user_authority: Signer<'info>,

    /// Account paying for the rent of new associated token accounts.
    #[account(mut)]
    pub rent_payer: Signer<'info>, // Pays for ATA creation

    #[account(
        init_if_needed,
        payer = rent_payer, // Rent payer
        associated_token::mint = mint,
        associated_token::authority = user_authority // The actual owner
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA для авторитета аккаунта кривой (bonding curve).
    pub bonding_curve_authority: AccountInfo<'info>,

    #[account(
        init_if_needed,
        // payer = user_authority,
        payer = rent_payer, // Payer changed
        associated_token::mint = mint,
        associated_token::authority = bonding_curve_authority,
    )]
    pub bonding_curve_token_account: Account<'info, TokenAccount>,

    // --- Аккаунты для казны реферальной программы --- 

    /// CHECK: PDA казны, принадлежащий реферальной программе.
    #[account(
        seeds = [b"referral_treasury".as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = referral_program::ID // Проверяем, что PDA выведен с ID реферальной программы
    )]
    pub referral_treasury_authority: AccountInfo<'info>,

    #[account(
        init_if_needed, // Создаем ATA для казны реферальной программы, если его нет
        // payer = user_authority,
        payer = rent_payer, // Payer changed
        associated_token::mint = mint,
        associated_token::authority = referral_treasury_authority, // Владелец ATA - PDA реферальной программы
    )]
    pub referral_treasury_token_account: Account<'info, TokenAccount>,

    // --- Аккаунты для AI Agent ---

    /// CHECK: Публичный ключ кошелька AI Agent (передается извне).
    pub ai_agent_authority: AccountInfo<'info>,

    #[account(
        init_if_needed, // Создаем ATA для AI Agent, если его нет
        // payer = user_authority,
        payer = rent_payer, // Payer changed
        associated_token::mint = mint,
        associated_token::authority = ai_agent_authority, // Владелец ATA - AI Agent
    )]
    pub ai_agent_token_account: Account<'info, TokenAccount>,

    // --- Необходимые программы --- 
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // Передаем саму реферальную программу, чтобы использовать ее ID в seeds::program
    // Используем AccountInfo, так как нам нужен только key/ID
    /// CHECK: Проверяем адрес ниже
    #[account(executable, address = referral_program::ID)] // Проверяем, что это исполняемый аккаунт с нужным ID
    pub referral_program: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Calculation overflow")]
    CalculationOverflow,
    #[msg("Total supply cannot be zero")]
    ZeroSupply,
}
