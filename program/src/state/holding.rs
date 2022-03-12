//! An offer account's holding ATA.

use solana_program::{
    account_info::AccountInfo,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
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
        rent_sysvar: &AccountInfo<'a>,
    ) -> Result<Self, ProgramError> {
        let create_ata_ix =
            create_associated_token_account(payer.key, offer_acc.key, offer_mint.key);
        invoke(
            &create_ata_ix,
            &[
                payer.to_owned(),
                new_holding_account.to_owned(),
                offer_acc.to_owned(),
                offer_mint.to_owned(),
                sys_prog.to_owned(),
                token_prog.to_owned(),
                rent_sysvar.to_owned(),
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
            .ok_or(SimpleDexError::NumericalError)?;
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
        offer: &OfferAccount<'a, 'me>,
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
        offer: &OfferAccount<'a, 'me>,
        refund_to: &AccountInfo<'a>,
        refund_rent_to: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        self.transfer(offer, refund_to, self.data.amount)?;
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
        )?;
        Ok(())
    }

    pub fn reload(mut self) -> Result<Self, ProgramError> {
        self.data = token_account_checked(self.account_info)?;
        Ok(self)
    }
}
