use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use super::is_pubkey_matching;

fn is_program(actual: &AccountInfo, expected: &Pubkey) -> Result<(), ProgramError> {
    is_pubkey_matching(actual, expected, ProgramError::IncorrectProgramId)
}

pub fn is_system_program(actual: &AccountInfo) -> Result<(), ProgramError> {
    is_program(actual, &solana_program::system_program::id())
}

pub fn is_token_program(actual: &AccountInfo) -> Result<(), ProgramError> {
    is_program(actual, &spl_token::id())
}

pub fn is_ata_program(actual: &AccountInfo) -> Result<(), ProgramError> {
    is_program(actual, &spl_associated_token_account::id())
}
