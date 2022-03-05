//! Program account types

use core::cmp::Ordering;

use solana_program::{clock::Slot, pubkey::Pubkey};

use crate::{
    error::SimpleDexError,
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
            return Err(SimpleDexError::InternalError);
        }
        let a_can_fill_b = a.offering >= b.accept_at_least;
        let b_can_fill_a = b.offering >= a.accept_at_least;
        let (amt_a_gives, amt_b_gives) = match (a_can_fill_b, b_can_fill_a) {
            (true, true) => (a.offering, b.offering),
            (true, false) => (b.accept_at_least, b.offering),
            (false, true) => (a.offering, a.accept_at_least),
            (false, false) => return Err(SimpleDexError::InternalError),
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

    pub fn max_willing_to_pay_for(&self, to_accept: u64) -> Result<u64, SimpleDexError> {
        if to_accept >= self.accept_at_least {
            return Ok(self.offering);
        }
        let proportion = Ratio::new(to_accept, self.accept_at_least)?;
        proportion.apply_floor(self.offering)
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
