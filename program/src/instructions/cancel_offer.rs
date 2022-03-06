use std::io::Cursor;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    checks::{is_owner, is_refund_rent_to, is_refund_to, is_signer, is_token_program},
    packun::SerializePacked,
    pda::try_create_offer_pda,
    state::{HoldingAccount, Offer, OfferAccount},
};

use super::SimpleDexInstruction;

const CANCEL_OFFER_ACCOUNTS_LEN: usize = 6;

pub fn process_cancel(accounts: &[AccountInfo]) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let owner = next_account_info(account_info_iter)?;
    let offer = next_account_info(account_info_iter)?;
    let holding = next_account_info(account_info_iter)?;
    let refund_to = next_account_info(account_info_iter)?;
    let refund_rent_to = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;

    // Deser
    let offer_acc = OfferAccount::load_checked(offer)?;
    let holding_acc = HoldingAccount::load_checked(holding, &offer_acc)?;

    // Checks
    is_signer(owner)?;
    is_owner(owner, &offer_acc.data)?;
    is_refund_to(refund_to, &offer_acc.data)?;
    is_refund_rent_to(refund_rent_to, &offer_acc.data)?;
    is_token_program(token_prog)?;

    // Process
    holding_acc.close(&offer_acc, refund_to, refund_rent_to)?;
    offer_acc.close(refund_rent_to)?;
    Ok(())
}

pub fn cancel_offer(offer: &Offer) -> Result<Instruction, ProgramError> {
    let offer_pubkey = try_create_offer_pda(offer)?;
    let holding = get_associated_token_address(&offer_pubkey, &offer.offer_mint);

    let mut accounts = Vec::with_capacity(CANCEL_OFFER_ACCOUNTS_LEN);
    accounts.push(AccountMeta::new_readonly(offer.owner, true));
    accounts.push(AccountMeta::new(offer_pubkey, false));
    accounts.push(AccountMeta::new(holding, false));
    accounts.push(AccountMeta::new(offer.refund_to, false));
    accounts.push(AccountMeta::new(offer.refund_rent_to, false));
    accounts.push(AccountMeta::new_readonly(spl_token::id(), false));

    let mut data = [0; SimpleDexInstruction::PACKED_LEN_CANCEL_OFFER];
    let mut writer = Cursor::new(data.as_mut());
    SimpleDexInstruction::CancelOffer.write_bytes(&mut writer)?;

    Ok(Instruction {
        program_id: crate::id(),
        accounts,
        data: data.to_vec(),
    })
}
