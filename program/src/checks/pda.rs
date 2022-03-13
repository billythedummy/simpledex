use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{error::SimpleDexError, pda::try_find_offer_pda};

use super::is_pubkey_matching;

pub fn is_offer_pda(
    actual: &AccountInfo,
    owner: &AccountInfo,
    offer_mint: &AccountInfo,
    accept_mint: &AccountInfo,
    seed: u16,
    bump: u8,
) -> Result<(), ProgramError> {
    let (found_pubkey, found_bump) =
        try_find_offer_pda(owner.key, offer_mint.key, accept_mint.key, seed)?;
    is_pubkey_matching(actual, &found_pubkey, SimpleDexError::InvalidOfferAccount)?;
    match bump == found_bump {
        true => Ok(()),
        false => Err(SimpleDexError::InvalidOfferBump.into()),
    }
}
