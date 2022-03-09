use crate::{error::SimpleDexError, types::Ratio};

pub const MATCHER_EXCESS_BONUS_DIVISOR: u64 = 2;

pub const TAKER_FEE_BPS: u64 = 10;

const BPS_BASE: u64 = 10_000;

const FEE_RATIO: Ratio = Ratio::new_unchecked(TAKER_FEE_BPS, BPS_BASE);

pub fn calc_fee(amount_given: u64) -> Result<u64, SimpleDexError> {
    FEE_RATIO.apply_floor(amount_given)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    prop_compose! {
        fn first_fee_sequence()
            (original_offering in 1..=u64::MAX,)
            (original_offering in Just(original_offering), amount_given in 1..=original_offering)
            -> (u64, u64, u64, u64) {
                let max_fee_levied = calc_fee(original_offering).unwrap();
                let original_balance = match max_fee_levied.checked_add(original_offering) {
                    Some(b) => b,
                    // original_balance overflowed
                    // just give a generic one
                    None => return (2, 2, 1, 1),
                };
                let fee_levied = calc_fee(amount_given).unwrap();
                let new_offering = original_offering - amount_given;
                let new_balance = original_balance - amount_given - fee_levied;
                (original_offering, amount_given, new_offering, new_balance)
            }
    }

    prop_compose! {
        fn new_offering_and_next_amount_given()
            (t in first_fee_sequence())
            (t in Just(t), next_amount_given in 1..=t.2)
        -> (u64, u64) {
            (t.3, next_amount_given)
        }
    }

    proptest! {
        #[test]
        fn test_always_has_enough_to_pay_next_fees(
            (new_balance, next_amount_given) in new_offering_and_next_amount_given()
        ) {
            let next_fee_levied = calc_fee(next_amount_given).unwrap();
            prop_assert!(next_amount_given + next_fee_levied <= new_balance);
        }
    }
}
