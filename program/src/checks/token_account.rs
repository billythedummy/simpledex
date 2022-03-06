use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::error::SimpleDexError;

pub fn token_account_checked(actual: &AccountInfo) -> Result<Account, ProgramError> {
    if actual.owner != &spl_token::ID {
        return Err(ProgramError::IllegalOwner);
    }
    // spl token IsInitialized trait checks token acc is initialized
    Account::unpack(*actual.data.borrow())
}

pub fn is_of_mint(token_account: &Account, expected_mint: &Pubkey) -> Result<(), SimpleDexError> {
    match &token_account.mint == expected_mint {
        true => Ok(()),
        false => Err(SimpleDexError::IncorrectMint),
    }
}

pub fn is_not_frozen(token_account: &Account) -> Result<(), SimpleDexError> {
    match token_account.is_frozen() {
        true => Err(SimpleDexError::TokenAccountFrozen),
        false => Ok(()),
    }
}
