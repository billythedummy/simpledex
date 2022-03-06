use std::io::Cursor;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    checks::{
        is_ata_program, is_not_frozen, is_not_pubkey, is_of_mint, is_offer_pda, is_signer,
        is_system_program, is_token_program, mint_account_checked, token_account_checked,
    },
    error::SimpleDexError,
    packun::SerializePacked,
    pda::try_find_offer_pda,
    state::{Holding, Offer},
};

use super::SimpleDexInstruction;

const CREATE_OFFER_ACCOUNTS_LEN: usize = 13;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreateOfferArgs {
    pub bump: u8,
    pub seed: u16,
    pub offering: u64,
    pub accept_at_least: u64,
}

pub fn process_create_offer(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateOfferArgs,
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;
    let pay_from = next_account_info(account_info_iter)?;
    let offer = next_account_info(account_info_iter)?;
    let holding = next_account_info(account_info_iter)?;
    let refund_to = next_account_info(account_info_iter)?;
    let credit_to = next_account_info(account_info_iter)?;
    let refund_rent_to = next_account_info(account_info_iter)?;
    let offer_mint = next_account_info(account_info_iter)?;
    let accept_mint = next_account_info(account_info_iter)?;
    let token_prog = next_account_info(account_info_iter)?;
    let ata_prog = next_account_info(account_info_iter)?;
    let sys_prog = next_account_info(account_info_iter)?;

    // Deser
    let refund_to_token_acc = token_account_checked(refund_to)?;
    let credit_to_token_acc = token_account_checked(credit_to)?;
    // This checks that the mints are initialized
    mint_account_checked(accept_mint)?;
    mint_account_checked(offer_mint)?;

    // Checks
    is_signer(payer)?;

    is_signer(owner)?;

    // rely on token program transfer to make sure pay_from is of the correct mint type

    is_offer_pda(offer, owner, offer_mint, accept_mint, args.seed, args.bump)?;

    // rely on ATA CPI safety check to make sure holding is offer's ATA

    is_of_mint(&refund_to_token_acc, offer_mint.key)?;
    is_not_frozen(&refund_to_token_acc)?;
    is_not_pubkey(
        refund_to,
        holding.key,
        SimpleDexError::RefundingToOfferAccounts,
    )?;

    is_of_mint(&credit_to_token_acc, accept_mint.key)?;
    is_not_frozen(&credit_to_token_acc)?;

    is_not_pubkey(
        refund_rent_to,
        offer.key,
        SimpleDexError::RefundingToOfferAccounts,
    )?;
    is_not_pubkey(
        refund_rent_to,
        holding.key,
        SimpleDexError::RefundingToOfferAccounts,
    )?;

    is_token_program(token_prog)?;
    is_ata_program(ata_prog)?;
    is_system_program(sys_prog)?;

    // Process
    Holding::create_to(holding, payer, offer, offer_mint, sys_prog, token_prog)?;
    let created_offer = Offer::create_to(
        offer,
        payer,
        sys_prog,
        args.offering,
        args.accept_at_least,
        args.seed,
        args.bump,
        owner.key,
        offer_mint.key,
        accept_mint.key,
        refund_to.key,
        credit_to.key,
        refund_rent_to.key,
    )?;
    Holding::receive_holding_tokens(holding, pay_from, offer, &created_offer)
}

#[allow(clippy::too_many_arguments)]
pub fn create_offer(
    payer: &Pubkey,
    owner: &Pubkey,
    pay_from: &Pubkey,
    refund_to: &Pubkey,
    credit_to: &Pubkey,
    refund_rent_to: &Pubkey,
    offer_mint: &Pubkey,
    accept_mint: &Pubkey,
    seed: u16,
    offering: u64,
    accept_at_least: u64,
) -> Result<Instruction, ProgramError> {
    let (offer, bump) = try_find_offer_pda(owner, offer_mint, accept_mint, seed)?;
    let holding = get_associated_token_address(&offer, offer_mint);

    let mut accounts = Vec::with_capacity(CREATE_OFFER_ACCOUNTS_LEN);
    accounts.push(AccountMeta::new(*payer, true));
    accounts.push(AccountMeta::new_readonly(*owner, true));
    accounts.push(AccountMeta::new(*pay_from, false));
    accounts.push(AccountMeta::new(offer, false));
    accounts.push(AccountMeta::new(holding, false));
    accounts.push(AccountMeta::new_readonly(*refund_to, false));
    accounts.push(AccountMeta::new_readonly(*credit_to, false));
    accounts.push(AccountMeta::new_readonly(*refund_rent_to, false));
    accounts.push(AccountMeta::new_readonly(*offer_mint, false));
    accounts.push(AccountMeta::new_readonly(*accept_mint, false));
    accounts.push(AccountMeta::new_readonly(spl_token::id(), false));
    accounts.push(AccountMeta::new_readonly(
        spl_associated_token_account::id(),
        false,
    ));
    accounts.push(AccountMeta::new_readonly(system_program::id(), false));

    let mut data = [0; SimpleDexInstruction::PACKED_LEN_CREATE_OFFER];
    let mut writer = Cursor::new(data.as_mut());
    SimpleDexInstruction::CreateOffer(CreateOfferArgs {
        bump,
        seed,
        offering,
        accept_at_least,
    })
    .write_bytes(&mut writer)?;

    Ok(Instruction {
        program_id: crate::id(),
        accounts,
        data: data.to_vec(),
    })
}
