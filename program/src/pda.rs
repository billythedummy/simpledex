#![macro_use]

use solana_program::pubkey::{Pubkey, PubkeyError};

use crate::error::SimpleDexError;

macro_rules! pda_seed {
    (@pubkey $pubkey_seed: expr) => {
        $pubkey_seed.as_ref()
    };
    (@u_16 $u16_seed: expr) => {
        &$u16_seed.to_le_bytes()
    };
    (@u_8 $u8_seed: expr) => {
        &[$u8_seed]
    };
}

macro_rules! offer_pda_seeds {
    ($offer: expr) => (
        offer_pda_seeds!(
            $offer.owner,
            $offer.offer_mint,
            $offer.accept_mint,
            $offer.seed,
            $offer.bump,
        )
    );
    ($owner: expr, $offer_mint: expr, $accept_mint: expr, $seed: expr $(,) ?) => (
        &[
            pda_seed!(@pubkey $owner),
            pda_seed!(@pubkey $offer_mint),
            pda_seed!(@pubkey $accept_mint),
            pda_seed!(@u_16 $seed),
        ]
    );
    ($owner: expr, $offer_mint: expr, $accept_mint: expr, $seed: expr, $bump: expr $(,) ?) => (
        &[
            pda_seed!(@pubkey $owner),
            pda_seed!(@pubkey $offer_mint),
            pda_seed!(@pubkey $accept_mint),
            pda_seed!(@u_16 $seed),
            pda_seed!(@u_8 $bump),
        ]
    );
}

pub fn try_find_offer_pda(
    owner: &Pubkey,
    offer_mint: &Pubkey,
    accept_mint: &Pubkey,
    seed: u16,
) -> Result<(Pubkey, u8), SimpleDexError> {
    Pubkey::try_find_program_address(
        offer_pda_seeds!(owner, offer_mint, accept_mint, seed),
        &crate::ID,
    )
    .ok_or(SimpleDexError::InternalError)
}

pub fn try_create_offer_pda(
    owner: &Pubkey,
    offer_mint: &Pubkey,
    accept_mint: &Pubkey,
    seed: u16,
    bump: u8,
) -> Result<Pubkey, PubkeyError> {
    Pubkey::create_program_address(
        offer_pda_seeds!(owner, offer_mint, accept_mint, seed, bump),
        &crate::ID,
    )
}
