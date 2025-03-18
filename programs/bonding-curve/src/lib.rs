use anchor_lang::prelude::*;

// Объявление ID программы
declare_id!("HgiiaxwngpLK7jS3hC5EYXz8JkgSpMcA1xdaRc7tCqTL");

// Публичные модули
pub mod math_lib;
pub mod errors;
pub mod state;
pub mod utils;
pub mod instructions;
pub mod program;

// Реэкспорт основного модуля программы
pub use program::bonding_curve;
// Реэкспорт структуры BondingCurve для использования внешними программами

// Определяем тип программы для использования внешними программами
pub type BondingCurve = state::BondingCurve;