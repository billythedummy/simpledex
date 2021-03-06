use self::create_offer::CreateOfferArgs;

pub mod cancel_offer;
pub mod create_offer;
pub mod match_offers;
mod packun;

pub use cancel_offer::cancel_offer;
pub use create_offer::create_offer;
pub use match_offers::match_offers;
use solana_program::msg;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleDexInstruction {
    CreateOffer(CreateOfferArgs),
    CancelOffer,
    MatchOffers,
}

// unfortunate, can't impl Pack for variable sized enums
impl SimpleDexInstruction {
    pub const PACKED_LEN_CREATE_OFFER: usize = 20; // 1 + 19
    pub const PACKED_LEN_CANCEL_OFFER: usize = 1;
    pub const PACKED_LEN_MATCH_OFFERS: usize = 1;

    pub fn log_invocation(&self) {
        match self {
            Self::CreateOffer(_) => msg!("CreateOffer"),
            Self::CancelOffer => msg!("CancelOffer"),
            Self::MatchOffers => msg!("MatchOffers"),
        }
    }
}
