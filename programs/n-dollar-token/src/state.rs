use anchor_lang::prelude::*;

#[account]
pub struct AdminAccount {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub last_mint_time: i64,
    pub total_supply: u64,
    pub bump: u8,
    // Поле для хранения дополнительных авторизованных ключей
    pub authorized_signers: [Option<Pubkey>; 3],  // Массив дополнительных подписантов
    // Поля для защиты от атак на время
    pub last_block_time: i64,  // Последний блок, когда был выполнен минт
    pub last_block_height: u64, // Высота последнего блока
    pub min_required_signers: u8, // Минимальное количество подписантов для чувствительных операций
}

impl AdminAccount {
    pub const SPACE: usize = 32 + 32 + 8 + 8 + 1 + ((32 + 1) * 3) + 8 + 8 + 1;
}
