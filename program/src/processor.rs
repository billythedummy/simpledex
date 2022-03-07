//! Program processor

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    id,
    instructions::{
        cancel_offer::process_cancel, create_offer::process_create_offer,
        match_offers::process_match_offers, SimpleDexInstruction,
    },
    packun::DeserializePacked,
};

pub struct Processor {}

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        if program_id != &id() {
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut reader = input;
        let instruction = SimpleDexInstruction::read_bytes(&mut reader)?;
        match instruction {
            SimpleDexInstruction::CreateOffer(args) => process_create_offer(accounts, args),
            SimpleDexInstruction::CancelOffer => process_cancel(accounts),
            SimpleDexInstruction::MatchOffers => process_match_offers(accounts),
        }
    }
}
