use solana_program::{account_info::AccountInfo, program_error::ProgramError};

pub fn is_signer(a: &AccountInfo) -> Result<(), ProgramError> {
    match a.is_signer {
        true => Ok(()),
        false => Err(ProgramError::MissingRequiredSignature),
    }
}
