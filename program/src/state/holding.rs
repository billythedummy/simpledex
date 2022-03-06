//! An offer account's holding ATA.

use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer;

use crate::{error::SimpleDexError, fee::calc_fee};

use super::Offer;

pub struct Holding;

impl Holding {
    pub fn create_to<'a>(
        payer: &AccountInfo<'a>,
        new_holding_account: &AccountInfo<'a>,
        offer_acc: &AccountInfo<'a>,
        offer_mint: &AccountInfo<'a>,
        sys_prog: &AccountInfo<'a>,
        token_prog: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let ix = create_ata_instruction(payer.key, offer_acc.key, offer_mint.key);
        invoke(
            &ix,
            &[
                payer.to_owned(),
                new_holding_account.to_owned(),
                offer_acc.to_owned(),
                offer_mint.to_owned(),
                sys_prog.to_owned(),
                token_prog.to_owned(),
            ],
        )
    }

    pub fn transfer_holding_tokens<'a>(
        offering: u64,
        pay_from: &AccountInfo<'a>,
        holding: &AccountInfo<'a>,
        offer_acc: &AccountInfo<'a>,
        offer: &Offer,
    ) -> Result<(), ProgramError> {
        let amt = offering
            .checked_add(calc_fee(offering)?)
            .ok_or(SimpleDexError::InternalError)?;
        let ix = transfer(
            &spl_token::id(),
            pay_from.key,
            holding.key,
            offer_acc.key,
            &[],
            amt,
        )?;
        invoke_signed(
            &ix,
            &[
                pay_from.to_owned(),
                holding.to_owned(),
                offer_acc.to_owned(),
            ],
            &[offer_pda_seeds!(offer)],
        )
    }
}

// TODO: ATA prog on mainnet still needs to pass in rent as sysvar
// and 1.0.4 was yanked, so I can't import it
// This will probably only work when mainnet ATA prog is updated
fn create_ata_instruction(payer: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Instruction {
    let associated_account_address = get_associated_token_address(owner, mint);
    Instruction {
        program_id: spl_associated_token_account::id(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: vec![], // Create instruction is an empty array
    }
}
