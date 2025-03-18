use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::GenesisError;

/// Проверяет, содержит ли строка только допустимые символы
pub fn validate_string(s: &str) -> bool {
    s.chars().all(|c| {
        c.is_alphanumeric() || c.is_whitespace() || VALID_SPECIAL_CHARS.contains(c)
    })
}

/// Проверяет имя токена на допустимость
pub fn validate_token_name(name: &str) -> Result<()> {
    require!(name.len() >= MIN_NAME_LENGTH, GenesisError::NameTooShort);
    require!(name.len() <= MAX_NAME_LENGTH, GenesisError::NameTooLong);
    require!(validate_string(name), GenesisError::InvalidCharacters);
    Ok(())
}

/// Проверяет символ токена на допустимость
pub fn validate_token_symbol(symbol: &str) -> Result<()> {
    require!(symbol.len() >= MIN_SYMBOL_LENGTH, GenesisError::SymbolTooShort);
    require!(symbol.len() <= MAX_SYMBOL_LENGTH, GenesisError::SymbolTooLong);
    require!(validate_string(symbol), GenesisError::InvalidCharacters);
    Ok(())
}

/// Рассчитывает количество токенов для начального предложения
pub fn calculate_initial_supply() -> u64 {
    MAX_SUPPLY / INITIAL_SUPPLY_PERCENTAGE
}

/// Создает seeds для PDA аккаунта CoinData
pub fn get_coin_data_seeds<'a>(mint: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
    [b"coin_data".as_ref(), mint.as_ref(), std::slice::from_ref(bump)]
}