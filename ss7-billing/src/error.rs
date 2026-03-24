use thiserror::Error;

#[derive(Error, Debug)]
pub enum BillingError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: f64, required: f64 },

    #[error("Duplicate account: {0}")]
    DuplicateAccount(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Invalid MSISDN format: {0}")]
    InvalidMsisdn(String),

    #[error("Account suspended: {0}")]
    AccountSuspended(String),
}

pub type Result<T> = std::result::Result<T, BillingError>;
