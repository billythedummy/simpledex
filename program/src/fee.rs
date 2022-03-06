use crate::{error::SimpleDexError, types::Ratio};

pub const TAKER_FEE_BPS: u64 = 10;

const BPS_BASE: u64 = 10_000;

const FEE_RATIO: Ratio = Ratio::new_unchecked(TAKER_FEE_BPS, BPS_BASE);

pub fn calc_fee(amt_transferred: u64) -> Result<u64, SimpleDexError> {
    FEE_RATIO.apply_floor(amt_transferred)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn always_has_enough_to_pay_next_fees(
            original_offering in 1..=u64::MAX,
            amt_transferred in 1..=u64::MAX,
            next_amt_transferred in 1..=u64::MAX,
        ) {
            // TODO: use prop_compose! to generate
            // amt_transferred <= original_offering
            // next_amt_transferred <= new_offering
            // instead
            if amt_transferred > original_offering {
                return Ok(());
            }
            let max_fee_levied = calc_fee(original_offering).unwrap();
            let original_balance = match max_fee_levied.checked_add(original_offering) {
                Some(b) => b,
                None => return Ok(()),
            };

            let fee_levied = calc_fee(amt_transferred).unwrap();

            let new_offering = original_offering - amt_transferred;
            let new_balance = original_balance - amt_transferred - fee_levied;

            if next_amt_transferred > new_offering {
                return Ok(());
            }

            let next_fee_levied = calc_fee(next_amt_transferred).unwrap();
            prop_assert!(next_amt_transferred + next_fee_levied <= new_balance);
        }
    }
}
