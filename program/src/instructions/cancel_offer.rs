use std::io::Cursor;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    checks::{is_owner, is_refund_rent_to, is_refund_to, is_signer, is_token_program},
    packun::SerializePacked,
    pda::try_create_offer_pda,
    state::{HoldingAccount, Offer, OfferAccount},
};

use super::SimpleDexInstruction;

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
    is_owner(owner.key, &offer_acc.data)?;
    is_refund_to(refund_to.key, &offer_acc.data)?;
    is_refund_rent_to(refund_rent_to.key, &offer_acc.data)?;
    is_token_program(token_prog)?;

    // Process
    let canceled_offer = offer_acc.account_info.key;
    let offer_mint = offer_acc.data.offer_mint;
    let offering = offer_acc.data.offering;
    let accept_mint = offer_acc.data.accept_mint;
    let accept_at_least = offer_acc.data.accept_at_least;

    holding_acc.close(&offer_acc, refund_to, refund_rent_to)?;
    offer_acc.close(refund_rent_to)?;

    log_success(
        canceled_offer,
        &offer_mint,
        offering,
        &accept_mint,
        accept_at_least,
    );
    Ok(())
}

fn log_success(
    canceled_offer: &Pubkey,
    offer_mint: &Pubkey,
    offering: u64,
    accept_mint: &Pubkey,
    accept_at_least: u64,
) {
    msg!(
        "CANCEL:{},{},{},{},{}",
        canceled_offer.to_string(),
        offer_mint.to_string(),
        offering,
        accept_mint.to_string(),
        accept_at_least
    );
}

pub fn cancel_offer(offer: &Offer) -> Result<Instruction, ProgramError> {
    let offer_pubkey = try_create_offer_pda(offer)?;
    let holding = get_associated_token_address(&offer_pubkey, &offer.offer_mint);

    let accounts = vec![
        AccountMeta::new_readonly(offer.owner, true),
        AccountMeta::new(offer_pubkey, false),
        AccountMeta::new(holding, false),
        AccountMeta::new(offer.refund_to, false),
        AccountMeta::new(offer.refund_rent_to, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let mut data = [0; SimpleDexInstruction::PACKED_LEN_CANCEL_OFFER];
    let mut writer = Cursor::new(data.as_mut());
    SimpleDexInstruction::CancelOffer.write_bytes(&mut writer)?;

    Ok(Instruction {
        program_id: crate::id(),
        accounts,
        data: data.to_vec(),
    })
}
