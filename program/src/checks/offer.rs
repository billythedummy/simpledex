use solana_program::pubkey::Pubkey;

use crate::{error::SimpleDexError, state::Offer};

macro_rules! is_pubkey_field {
    ($fn_name: ident, $field: ident, $err: expr) => {
        pub fn $fn_name(actual: &Pubkey, offer: &Offer) -> Result<(), SimpleDexError> {
            match *actual == offer.$field {
                true => Ok(()),
                false => Err($err),
            }
        }
    };
}

is_pubkey_field!(is_owner, owner, SimpleDexError::IncorrectOwner);
is_pubkey_field!(is_refund_to, refund_to, SimpleDexError::IncorrectRefundTo);
is_pubkey_field!(
    is_refund_rent_to,
    refund_rent_to,
    SimpleDexError::IncorredRefundRentTo
);
is_pubkey_field!(is_credit_to, credit_to, SimpleDexError::IncorrectMint);
