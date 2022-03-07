//! Program account types

use core::cmp::Ordering;
use std::io::Cursor;

use solana_program::{
    account_info::AccountInfo,
    clock::{Clock, Slot},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_memory::sol_memset,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    account::Account,
    error::SimpleDexError,
    packun::{DeserializePacked, SerializePacked},
    pda::try_create_offer_pda,
    types::{OfferSeq, Ratio},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Offer {
    pub slot: Slot,
    pub offering: u64,
    pub accept_at_least: u64,
    pub seed: u16,
    pub bump: u8,
    pub owner: Pubkey,
    pub offer_mint: Pubkey,
    pub accept_mint: Pubkey,
    pub refund_to: Pubkey,
    pub credit_to: Pubkey,
    pub refund_rent_to: Pubkey,
}

impl Offer {
    pub fn try_match(a: &Self, b: &Self) -> Result<(u64, u64), SimpleDexError> {
        if !Self::is_match(a, b) {
            return Err(SimpleDexError::OffersDontMatch);
        }
        let a_can_fill_b = a.offering >= b.accept_at_least;
        let b_can_fill_a = b.offering >= a.accept_at_least;
        let (amt_a_gives, amt_b_gives) = match (a_can_fill_b, b_can_fill_a) {
            (true, true) => (a.offering, b.offering),
            (true, false) => (b.accept_at_least, b.offering),
            (false, true) => (a.offering, a.accept_at_least),
            (false, false) => return Err(SimpleDexError::OffersDontMatch),
        };
        Ok((amt_a_gives, amt_b_gives))
    }

    fn is_match(a: &Self, b: &Self) -> bool {
        // bid >= ask
        // (a.offering / a.accept_at_least) >= (b.accept_at_least / b.offering)
        // since all vals positive,
        // = a.offering * b.offering >= a.accept_at_least * b.accept_at_least
        a.offering as u128 * b.offering as u128
            >= a.accept_at_least as u128 * b.accept_at_least as u128
    }

    pub fn is_closed(&self) -> bool {
        self.offering == 0 || self.accept_at_least == 0
    }

    pub fn relationship_with(&self, other: &Self) -> OfferSeq {
        match self.slot.cmp(&other.slot) {
            Ordering::Equal => OfferSeq::Neither,
            Ordering::Less => OfferSeq::Maker,
            Ordering::Greater => OfferSeq::Taker,
        }
    }

    pub fn min_willing_to_receive_for(&self, to_pay: u64) -> Result<u64, SimpleDexError> {
        // shouldnt be greater than
        if to_pay >= self.offering {
            return Ok(self.accept_at_least);
        }
        let proportion = Ratio::new(to_pay, self.offering)?;
        proportion.apply_ceil(self.accept_at_least)
    }

    pub fn update_offer_matched(mut self, amount_given: u64) -> Result<Self, SimpleDexError> {
        let accept_over_offer = Ratio::new(self.accept_at_least, self.offering)?;
        let new_offering = self
            .offering
            .checked_sub(amount_given)
            .ok_or(SimpleDexError::InternalError)?;
        // round towards higher price
        let new_accept_at_least = accept_over_offer.apply_ceil(new_offering)?;
        self.offering = new_offering;
        self.accept_at_least = new_accept_at_least;
        Ok(self)
    }
}

pub type OfferAccount<'a, 'me> = Account<'a, 'me, Offer>;

impl<'a, 'me> OfferAccount<'a, 'me> {
    /// Create and save offer account to storage
    #[allow(clippy::too_many_arguments)]
    pub fn create_to(
        new_offer_account: &'me AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        offering: u64,
        accept_at_least: u64,
        seed: u16,
        bump: u8,
        owner: &Pubkey,
        offer_mint: &Pubkey,
        accept_mint: &Pubkey,
        refund_to: &Pubkey,
        credit_to: &Pubkey,
        refund_rent_to: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let clock = Clock::get()?;
        let res = Offer {
            slot: clock.slot,
            offering,
            accept_at_least,
            seed,
            bump,
            owner: owner.to_owned(),
            offer_mint: offer_mint.to_owned(),
            accept_mint: accept_mint.to_owned(),
            refund_to: refund_to.to_owned(),
            credit_to: credit_to.to_owned(),
            refund_rent_to: refund_rent_to.to_owned(),
        };
        create_pda_account(
            Offer::LEN,
            payer,
            system_program,
            new_offer_account,
            offer_pda_seeds!(res),
        )?;
        Offer::pack(res, &mut new_offer_account.data.borrow_mut())?;
        Ok(Self {
            account_info: new_offer_account,
            data: res,
        })
    }

    pub fn load_checked(offer_account: &'me AccountInfo<'a>) -> Result<Self, ProgramError> {
        let data = Offer::unpack(*offer_account.data.borrow())?;
        let expected_pda = try_create_offer_pda(&data)?;
        if &expected_pda != offer_account.key {
            return Err(SimpleDexError::IncorrectOfferAccount.into());
        }
        Ok(Self {
            account_info: offer_account,
            data,
        })
    }

    pub fn close(self, refund_rent_to: &AccountInfo<'a>) -> Result<(), SimpleDexError> {
        let refund_rent_to_starting_lamports = refund_rent_to.lamports();
        **refund_rent_to.lamports.borrow_mut() = refund_rent_to_starting_lamports
            .checked_add(self.account_info.lamports())
            .ok_or(SimpleDexError::InternalError)?;

        **self.account_info.lamports.borrow_mut() = 0;
        let mut data = self.account_info.data.borrow_mut();
        let data_len = data.len();
        sol_memset(*data, 0, data_len);
        Ok(())
    }

    pub fn save(self) -> Result<(), ProgramError> {
        Offer::pack(self.data, &mut self.account_info.data.borrow_mut())
    }
}

fn create_pda_account<'a>(
    space: usize,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> ProgramResult {
    let owner = &crate::id();
    let rent = Rent::get()?;
    if new_pda_account.lamports() > 0 {
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(new_pda_account.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[
                    payer.clone(),
                    new_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(new_pda_account.key, owner),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )
    } else {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )
    }
}

impl IsInitialized for Offer {
    fn is_initialized(&self) -> bool {
        self.slot != 0
    }
}

impl Sealed for Offer {}

// TODO: this should be derived
impl Pack for Offer {
    const LEN: usize = 221;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        // unwrap safety: length should be checked in pack() already,
        // cursor is just into a byte slice, should have no IO errors
        let mut writer = Cursor::new(dst);
        self.slot.write_bytes(&mut writer).unwrap();
        self.offering.write_bytes(&mut writer).unwrap();
        self.accept_at_least.write_bytes(&mut writer).unwrap();
        self.seed.write_bytes(&mut writer).unwrap();
        self.bump.write_bytes(&mut writer).unwrap();
        self.owner.write_bytes(&mut writer).unwrap();
        self.offer_mint.write_bytes(&mut writer).unwrap();
        self.accept_mint.write_bytes(&mut writer).unwrap();
        self.refund_to.write_bytes(&mut writer).unwrap();
        self.credit_to.write_bytes(&mut writer).unwrap();
        self.refund_rent_to.write_bytes(&mut writer).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut reader = src;
        Ok(Self {
            slot: Slot::read_bytes(&mut reader)?,
            offering: u64::read_bytes(&mut reader)?,
            accept_at_least: u64::read_bytes(&mut reader)?,
            seed: u16::read_bytes(&mut reader)?,
            bump: u8::read_bytes(&mut reader)?,
            owner: Pubkey::read_bytes(&mut reader)?,
            offer_mint: Pubkey::read_bytes(&mut reader)?,
            accept_mint: Pubkey::read_bytes(&mut reader)?,
            refund_to: Pubkey::read_bytes(&mut reader)?,
            credit_to: Pubkey::read_bytes(&mut reader)?,
            refund_rent_to: Pubkey::read_bytes(&mut reader)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_packing() {
        assert_eq!(224, std::mem::size_of::<Offer>());
    }
}
