use anchor_lang::error_code;
use solana_program_test::BanksClientError;
use std::error::Error;
use std::fmt;

#[error_code]
pub enum CommonError {
    #[msg("unknown error")]
    Unknown,
}

impl From<BanksClientError> for CommonError {
    fn from(error: BanksClientError) -> Self {
        match error {
            _ => CommonError::Unknown, // You might want a default mapping
        }
    }
}

#[derive(Debug)]
pub struct CustomError {
    error_code: CommonError,
    message: String,
}

impl CustomError {
    pub fn code<E>(error_code: CommonError, s: String) -> Self
    where
        E: Error,
    {
        Self {
            error_code,
            message: s,
        }
    }
    pub fn new<E: Error>(error_code: CommonError, error: E) -> Self {
        Self {
            error_code,
            message: error.to_string(),
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Error Code: {}] {}", self.error_code, self.message)
    }
}

impl From<std::io::Error> for CustomError {
    fn from(error: std::io::Error) -> Self {
        CustomError::new(CommonError::Unknown, error)
    }
}

impl Error for CustomError {}
