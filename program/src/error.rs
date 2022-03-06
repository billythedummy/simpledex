//! Error types

use core::num::TryFromIntError;

use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the simpledex program.
#[derive(Clone, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
#[error("")] // no point deriving display individually if we cant use it due to high cost. Use PrintProgramError instead.
pub enum SimpleDexError {
    // 0
    InternalError,
    PackunError,
    InvalidHoldingAccount,
    IncorrectMint,
    TokenAccountFrozen,
    // 5
    RefundingToOfferAccounts,
}

impl From<SimpleDexError> for ProgramError {
    fn from(e: SimpleDexError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SimpleDexError {
    fn type_of() -> &'static str {
        "SimpleDexError"
    }
}

impl PrintProgramError for SimpleDexError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            Self::InternalError => msg!("unknown"),
            Self::PackunError => msg!("(de)serialization error"),
            Self::InvalidHoldingAccount => msg!("provided account is not a valid holding account"),
            Self::IncorrectMint => msg!("given mint does not match token account's"),
            Self::TokenAccountFrozen => msg!("provided token account is frozen"),
            Self::RefundingToOfferAccounts => {
                msg!("attempting to refund tokens/lamports to offer accounts")
            }
        }
    }
}

impl From<TryFromIntError> for SimpleDexError {
    fn from(_e: TryFromIntError) -> Self {
        Self::InternalError
    }
}
