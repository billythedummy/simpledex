//! An offer account's holding ATA.

use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    instruction::{close_account, transfer},
    state::Account as TokenAccount,
};

use crate::{
    account::Account, checks::token_account_checked, error::SimpleDexError, fee::calc_fee,
};

use super::{Offer, OfferAccount};

pub type HoldingAccount<'a, 'me> = Account<'a, 'me, TokenAccount>;

impl<'a, 'me> HoldingAccount<'a, 'me> {
    pub fn create_to(
        new_holding_account: &'me AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        offer_acc: &AccountInfo<'a>,
        offer_mint: &AccountInfo<'a>,
        sys_prog: &AccountInfo<'a>,
        token_prog: &AccountInfo<'a>,
    ) -> Result<Self, ProgramError> {
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
        )?;
        Ok(Self {
            account_info: new_holding_account,
            data: token_account_checked(new_holding_account)?,
        })
    }

    pub fn load_checked(
        holding_account: &'me AccountInfo<'a>,
        offer_account: &OfferAccount,
    ) -> Result<Self, ProgramError> {
        let data = token_account_checked(holding_account)?;
        let res = Self {
            account_info: holding_account,
            data,
        };
        res.is_ata_of(offer_account)?;
        Ok(res)
    }

    fn is_ata_of(&self, offer: &OfferAccount) -> Result<(), SimpleDexError> {
        let expected = get_associated_token_address(offer.account_info.key, &offer.data.offer_mint);
        match expected == *self.account_info.key {
            true => Ok(()),
            false => Err(SimpleDexError::InvalidHoldingAccount),
        }
    }

    pub fn receive_holding_tokens(
        &self,
        owner: &AccountInfo<'a>,
        pay_from: &AccountInfo<'a>,
        offer: &Offer,
    ) -> Result<(), ProgramError> {
        let amt = offer
            .offering
            .checked_add(calc_fee(offer.offering)?)
            .ok_or(SimpleDexError::InternalError)?;
        let ix = transfer(
            &spl_token::id(),
            pay_from.key,
            self.account_info.key,
            owner.key,
            &[],
            amt,
        )?;
        invoke(
            &ix,
            &[
                pay_from.to_owned(),
                self.account_info.to_owned(),
                owner.to_owned(),
            ],
        )
    }

    pub fn transfer(
        &self,
        offer: &Account<'a, 'me, Offer>,
        to: &AccountInfo<'a>,
        amt: u64,
    ) -> Result<(), ProgramError> {
        let ix = transfer(
            &spl_token::id(),
            self.account_info.key,
            to.key,
            offer.account_info.key,
            &[],
            amt,
        )?;
        invoke_signed(
            &ix,
            &[
                self.account_info.to_owned(),
                to.to_owned(),
                offer.account_info.to_owned(),
            ],
            &[offer_pda_seeds!(offer.data)],
        )
    }

    pub fn close(
        self,
        offer: &Account<'a, 'me, Offer>,
        refund_to: &AccountInfo<'a>,
        refund_rent_to: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let balance = self.data.amount;
        if balance > 0 {
            self.transfer(offer, refund_to, balance)?;
        }
        let ix = close_account(
            &spl_token::id(),
            self.account_info.key,
            refund_rent_to.key,
            offer.account_info.key,
            &[],
        )?;
        invoke_signed(
            &ix,
            &[
                self.account_info.to_owned(),
                refund_rent_to.to_owned(),
                offer.account_info.to_owned(),
            ],
            &[offer_pda_seeds!(offer.data)],
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
