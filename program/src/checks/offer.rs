use solana_program::account_info::AccountInfo;

use crate::{error::SimpleDexError, state::Offer};

macro_rules! is_field {
    ($fn_name: ident, $field: ident, $err: expr) => {
        pub fn $fn_name(actual: &AccountInfo, offer: &Offer) -> Result<(), SimpleDexError> {
            match *actual.key == offer.$field {
                true => Ok(()),
                false => Err($err),
            }
        }
    };
}

is_field!(is_owner, owner, SimpleDexError::IncorrectOwner);
is_field!(is_refund_to, refund_to, SimpleDexError::IncorrectRefundTo);
is_field!(
    is_refund_rent_to,
    refund_rent_to,
    SimpleDexError::IncorredRefundRentTo
);
