use self::create_offer::CreateOfferArgs;

pub mod cancel_offer;
pub mod create_offer;
pub mod match_offers;
mod packun;

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
}
