//! Custom types

use core::convert::TryInto;
use spl_math::precise_number::PreciseNumber;

use crate::error::SimpleDexError;

fn try_into_precise_number(n: u64) -> Result<PreciseNumber, SimpleDexError> {
    PreciseNumber::new(n as u128).ok_or(SimpleDexError::InternalError)
}

fn try_from_precise_number(p: PreciseNumber) -> Result<u64, SimpleDexError> {
    let expanded = p.to_imprecise().ok_or(SimpleDexError::InternalError)?;
    Ok(expanded.try_into()?)
}

pub struct Ratio {
    num: u64,
    denom: u64,
}

impl Ratio {
    pub fn new(num: u64, denom: u64) -> Result<Self, SimpleDexError> {
        if denom == 0 {
            return Err(SimpleDexError::InternalError);
        }
        Ok(Self::new_unchecked(num, denom))
    }

    pub const fn new_unchecked(num: u64, denom: u64) -> Self {
        Self { num, denom }
    }

    pub fn apply_floor(&self, token_amt: u64) -> Result<u64, SimpleDexError> {
        try_from_precise_number(self.apply(token_amt)?)
    }

    pub fn apply_ceil(&self, token_amt: u64) -> Result<u64, SimpleDexError> {
        let res = self
            .apply(token_amt)?
            .ceiling()
            .ok_or(SimpleDexError::InternalError)?;
        try_from_precise_number(res)
    }

    fn apply(&self, token_amt: u64) -> Result<PreciseNumber, SimpleDexError> {
        let num_precise = try_into_precise_number(self.num)?;
        let denom_precise = try_into_precise_number(self.denom)?;
        let token_amt_precise = try_into_precise_number(token_amt)?;
        num_precise
            .checked_mul(&token_amt_precise)
            .and_then(|u| u.checked_div(&denom_precise))
            .ok_or(SimpleDexError::InternalError)
    }
}

pub enum OfferSeq {
    Maker,
    Taker,
    Neither,
}
