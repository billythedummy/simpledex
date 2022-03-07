use std::io::Cursor;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    checks::{is_credit_to, is_refund_rent_to, is_refund_to, is_token_program},
    error::SimpleDexError,
    fee::calc_fee,
    packun::SerializePacked,
    pda::try_create_offer_pda,
    state::{HoldingAccount, Offer, OfferAccount},
    types::OfferSeq,
};

use super::SimpleDexInstruction;

pub fn process_match_offers(accounts: &[AccountInfo]) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let offering_a = next_account_info(account_info_iter)?;
    let holding_a = next_account_info(account_info_iter)?;
    let offering_b = next_account_info(account_info_iter)?;
    let holding_b = next_account_info(account_info_iter)?;
    let credit_to_a = next_account_info(account_info_iter)?;
    let refund_to_a = next_account_info(account_info_iter)?;
    let refund_rent_to_a = next_account_info(account_info_iter)?;
    let credit_to_b = next_account_info(account_info_iter)?;
    let refund_to_b = next_account_info(account_info_iter)?;
    let refund_rent_to_b = next_account_info(account_info_iter)?;
    let matcher_a = next_account_info(account_info_iter)?;
    let matcher_b = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;

    // Deser
    let offering_a_acc = OfferAccount::load_checked(offering_a)?;
    let holding_a_acc = HoldingAccount::load_checked(holding_a, &offering_a_acc)?;
    let offering_b_acc = OfferAccount::load_checked(offering_b)?;
    let holding_b_acc = HoldingAccount::load_checked(holding_b, &offering_b_acc)?;

    // Checks
    // rely on token program transfer checks to ensure mints match between the 2 offers
    // and for the matcher fee accounts

    is_credit_to(credit_to_a.key, &offering_a_acc.data)?;
    is_refund_to(refund_to_a.key, &offering_a_acc.data)?;
    is_refund_rent_to(refund_rent_to_a.key, &offering_a_acc.data)?;

    is_credit_to(credit_to_b.key, &offering_b_acc.data)?;
    is_refund_to(refund_to_b.key, &offering_b_acc.data)?;
    is_refund_rent_to(refund_rent_to_b.key, &offering_b_acc.data)?;

    is_token_program(token_prog)?;

    // Process
    let (amt_a_gives, amt_b_gives) = Offer::try_match(&offering_a_acc.data, &offering_b_acc.data)?;
    let receipt = Receipt::calc(
        amt_a_gives,
        amt_b_gives,
        &offering_a_acc.data,
        &offering_b_acc.data,
    )?;

    holding_a_acc.transfer(&offering_a_acc, credit_to_b, receipt.a_to_b)?;
    holding_a_acc.transfer(&offering_a_acc, matcher_a, receipt.a_to_matcher)?;
    holding_b_acc.transfer(&offering_b_acc, credit_to_a, receipt.b_to_a)?;
    holding_b_acc.transfer(&offering_b_acc, matcher_b, receipt.b_to_matcher)?;

    update_offer_accounts(
        offering_a_acc,
        holding_a_acc,
        amt_a_gives,
        refund_to_a,
        refund_rent_to_a,
    )?;
    update_offer_accounts(
        offering_b_acc,
        holding_b_acc,
        amt_b_gives,
        refund_to_b,
        refund_rent_to_b,
    )?;

    Ok(())
}

struct Receipt {
    a_to_b: u64,
    b_to_a: u64,
    a_to_matcher: u64,
    b_to_matcher: u64,
}

impl Receipt {
    fn calc(
        amt_a_gives: u64,
        amt_b_gives: u64,
        offering_a: &Offer,
        offering_b: &Offer,
    ) -> Result<Self, SimpleDexError> {
        let (mut a_to_matcher, mut b_to_matcher) = match offering_a.relationship_with(offering_b) {
            OfferSeq::Maker => (0, calc_fee(amt_b_gives)?),
            OfferSeq::Taker => (calc_fee(amt_a_gives)?, 0),
            OfferSeq::Neither => (calc_fee(amt_a_gives)? / 2, calc_fee(amt_b_gives)? / 2),
        };

        let excess_a =
            amt_a_gives.saturating_sub(offering_b.min_willing_to_receive_for(amt_b_gives)?);
        let excess_b =
            amt_b_gives.saturating_sub(offering_a.min_willing_to_receive_for(amt_a_gives)?);
        let bonus_a = excess_a / 2;
        let bonus_b = excess_b / 2;

        // overflow safety:
        // bonus_a in [0, amt_a_gives / 2]
        let a_to_b = amt_a_gives - bonus_a;
        let b_to_a = amt_b_gives - bonus_b;

        a_to_matcher = a_to_matcher
            .checked_add(bonus_a)
            .ok_or(SimpleDexError::InternalError)?;
        b_to_matcher = b_to_matcher
            .checked_add(bonus_b)
            .ok_or(SimpleDexError::InternalError)?;

        Ok(Self {
            a_to_b,
            b_to_a,
            a_to_matcher,
            b_to_matcher,
        })
    }
}

fn update_offer_accounts<'a, 'me>(
    mut offer_acc: OfferAccount<'a, 'me>,
    mut holding_acc: HoldingAccount<'a, 'me>,
    amount_given: u64,
    refund_to: &AccountInfo<'a>,
    refund_rent_to: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    offer_acc.data = offer_acc.data.update_offer_matched(amount_given)?;
    match offer_acc.data.is_closed() {
        true => {
            holding_acc = holding_acc.reload()?;
            holding_acc.close(&offer_acc, refund_to, refund_rent_to)?;
            offer_acc.close(refund_rent_to)?;
            Ok(())
        }
        false => offer_acc.save(),
    }
}

pub fn match_offers(
    offering_a: &Offer,
    offering_b: &Offer,
    matcher_a: &Pubkey,
    matcher_b: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let offering_a_pubkey = try_create_offer_pda(offering_a)?;
    let holding_a = get_associated_token_address(&offering_a_pubkey, &offering_a.offer_mint);
    let offering_b_pubkey = try_create_offer_pda(offering_b)?;
    let holding_b = get_associated_token_address(&offering_b_pubkey, &offering_b.offer_mint);

    let accounts = vec![
        AccountMeta::new(offering_a_pubkey, false),
        AccountMeta::new(holding_a, false),
        AccountMeta::new(offering_b_pubkey, false),
        AccountMeta::new(holding_b, false),
        AccountMeta::new(offering_a.credit_to, false),
        AccountMeta::new(offering_a.refund_to, false),
        AccountMeta::new(offering_a.refund_rent_to, false),
        AccountMeta::new(offering_b.credit_to, false),
        AccountMeta::new(offering_b.refund_to, false),
        AccountMeta::new(offering_b.refund_rent_to, false),
        AccountMeta::new(*matcher_a, false),
        AccountMeta::new(*matcher_b, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let mut data = [0; SimpleDexInstruction::PACKED_LEN_MATCH_OFFERS];
    let mut writer = Cursor::new(data.as_mut());
    SimpleDexInstruction::MatchOffers.write_bytes(&mut writer)?;

    Ok(Instruction {
        program_id: crate::id(),
        accounts,
        data: data.to_vec(),
    })
}
