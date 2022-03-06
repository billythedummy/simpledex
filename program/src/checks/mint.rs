use solana_program::{account_info::AccountInfo, program_error::ProgramError, program_pack::Pack};
use spl_token::state::Mint;

pub fn mint_account_checked(actual: &AccountInfo) -> Result<Mint, ProgramError> {
    if actual.owner != &spl_token::id() {
        return Err(ProgramError::IllegalOwner);
    }
    // spl token IsInitialized trait checks mint is initialized
    Mint::unpack(*actual.data.borrow())
}
