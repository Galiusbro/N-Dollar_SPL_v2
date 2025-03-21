use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
// use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::program::invoke_signed;

#[derive(Clone)]
pub struct LiquidityManager;

// impl anchor_lang::Id for LiquidityManager {
//     fn id() -> Pubkey {
//         crate::id()
//     }
// }

/// Функция для CPI вызова инструкции обновления состояния ликвидности после минта
pub fn update_after_mint<'info>(
    liquidity_manager_program: AccountInfo<'info>,
    liquidity_manager: AccountInfo<'info>,
    pool_ndollar_account: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    ndollar_amount: u64,
    accounts: Vec<AccountInfo<'info>>,
    signers_seeds: &[&[&[u8]]],
) -> Result<()> {
    // Вычисление дискриминатора инструкции
    let sighash = anchor_lang::solana_program::hash::hash("global:update_after_mint".as_bytes());
    let discriminator = sighash.to_bytes()[..8].to_vec();
    
    // Подготовка данных инструкции
    let mut data = discriminator;
    data.extend_from_slice(&ndollar_amount.to_le_bytes());
    
    // Создание инструкции
    let instruction = Instruction {
        program_id: liquidity_manager_program.key(),
        accounts: vec![
            AccountMeta::new(liquidity_manager.key(), false),
            AccountMeta::new(pool_ndollar_account.key(), false),
            AccountMeta::new_readonly(mint.key(), false),
        ],
        data,
    };
    
    // Вызов инструкции с подписью
    invoke_signed(
        &instruction,
        &accounts,
        signers_seeds,
    ).map_err(Into::into)
}