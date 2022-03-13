use std::io::Cursor;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::SysvarId,
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
    state::{HoldingAccount, OfferAccount},
};

use super::SimpleDexInstruction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreateOfferArgs {
    pub bump: u8,
    pub seed: u16,
    pub offering: u64,
    pub accept_at_least: u64,
}

pub fn process_create_offer(
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
    // TODO: remove once spl-ATA 1.0.5 drops
    let rent = next_account_info(account_info_iter)?;

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
    // rely on ATA CPI safety check to make sure rent is indeed rent sysvar

    // Process
    let created_holding = HoldingAccount::create_to(
        holding, payer, offer, offer_mint, sys_prog, token_prog, rent,
    )?;
    let created_offer = OfferAccount::create_to(
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
    created_holding.receive_holding_tokens(owner, pay_from, &created_offer.data)?;
    log_success(
        offer.key,
        offer_mint.key,
        args.offering,
        accept_mint.key,
        args.accept_at_least,
    );
    Ok(())
}

fn log_success(
    created_offer: &Pubkey,
    offer_mint: &Pubkey,
    offering: u64,
    accept_mint: &Pubkey,
    accept_at_least: u64,
) {
    // Comparison:
    // concat_string! prog size 212120 bytes
    // format str prog size 209208 bytes
    //
    // concat_string! BPF instructions executed 100638. compute units 139454
    // format str BPF instructions exec 65614. compute units 96931
    // with no logs at all, BPF instructions exec 29140. compute units 63314
    msg!(
        "CREATE:{},{},{},{},{}",
        created_offer.to_string(),
        offer_mint.to_string(),
        offering,
        accept_mint.to_string(),
        accept_at_least
    );
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

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(*pay_from, false),
        AccountMeta::new(offer, false),
        AccountMeta::new(holding, false),
        AccountMeta::new_readonly(*refund_to, false),
        AccountMeta::new_readonly(*credit_to, false),
        AccountMeta::new_readonly(*refund_rent_to, false),
        AccountMeta::new_readonly(*offer_mint, false),
        AccountMeta::new_readonly(*accept_mint, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        // TODO: remove once spl-ATA 1.0.5 drops
        AccountMeta::new_readonly(Rent::id(), false),
    ];

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
