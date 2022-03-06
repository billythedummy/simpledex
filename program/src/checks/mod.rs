mod account_meta;
mod mint;
mod offer;
mod pda;
mod program_id;
mod token_account;

use std::error::Error;

pub use account_meta::*;
pub use mint::*;
pub use offer::*;
pub use pda::*;
pub use program_id::*;
pub use token_account::*;

use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

fn is_pubkey_matching<E: Error>(actual: &AccountInfo, expected: &Pubkey, err: E) -> Result<(), E> {
    match actual.key == expected {
        true => Ok(()),
        false => Err(err),
    }
}

pub fn is_not_pubkey<E: Error>(actual: &AccountInfo, not: &Pubkey, err: E) -> Result<(), E> {
    match actual.key == not {
        true => Err(err),
        false => Ok(()),
    }
}
