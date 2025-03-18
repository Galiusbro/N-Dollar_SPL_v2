use anchor_lang::prelude::*;

/// Коды ошибок программы
#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
    
    #[msg("Program is already authorized")]
    ProgramAlreadyAuthorized,
    
    #[msg("Too many authorized programs")]
    TooManyAuthorizedPrograms,
    
    #[msg("Program is not authorized")]
    ProgramNotAuthorized,
    
    #[msg("Fee is too high, max is 10% (1000 basis points)")]
    FeeTooHigh,
    
    #[msg("Admin Config is already upgraded")]
    AlreadyUpgraded,
} 