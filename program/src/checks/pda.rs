use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::error::SimpleDexError;

use super::is_pubkey_matching;

pub fn is_offer_pda(
    actual: &AccountInfo,
    owner: &AccountInfo,
    offer_mint: &AccountInfo,
    accept_mint: &AccountInfo,
    seed: u16,
    bump: u8,
) -> Result<(), ProgramError> {
    let (found_pubkey, found_bump) = {
        let owner = owner.key;
        let offer_mint = offer_mint.key;
        let accept_mint = accept_mint.key;
        Pubkey::try_find_program_address(
            offer_pda_seeds!(owner, offer_mint, accept_mint, seed),
            &crate::id(),
        )
        .ok_or(SimpleDexError::InternalError)
    }?;
    is_pubkey_matching(actual, &found_pubkey, SimpleDexError::InvalidHoldingAccount)?;
    match bump == found_bump {
        true => Ok(()),
        false => Err(SimpleDexError::InvalidHoldingAccount.into()),
    }
}
